#!/usr/bin/python3
"""Stdlib runner regressions; synthetic data and mocked compilation only.

Run: /usr/bin/python3 -I tools/test_build_keyless_sbf.py
No compiler, SBF loader, key generator or real key material is used here.
"""

import contextlib
import io
import json
import os
from pathlib import Path
import shutil
import stat
import struct
import sys
import types
import unittest
from unittest.mock import patch
import uuid

SCRIPT = Path(__file__).with_name('build_keyless_sbf.py').resolve()
runner = types.ModuleType('keyless_runner_regression_subject')
runner.__file__ = str(SCRIPT)
exec(compile(SCRIPT.read_text(), str(SCRIPT), 'exec'), runner.__dict__)


def synthetic_elf(*, machine=247, entry=0x1000, text_flags=6,
                  segment_flags=5, segment_offset=0x80, symbol_value=0x1000, flags=0):
    """Parser fixture only, not a compiled or runtime-valid SBF program."""
    data = bytearray(0x300)
    data[:16] = b'\x7fELF\x02\x01\x01' + bytes(9)
    struct.pack_into('<HHIQQQIHHHHHH', data, 16, 3, machine, 1, entry, 64,
                     0x180, flags, 64, 56, 1, 64, 5, 2)
    struct.pack_into('<IIQQQQQQ', data, 64, 1, segment_flags, segment_offset,
                     0x1000, 0x1000, 16, 16, 8)
    names = b'\0.text\0.shstrtab\0.dynstr\0.dynsym\0'
    data[0x90:0x90 + len(names)] = names
    strings = b'\0entrypoint\0'
    data[0xc0:0xc0 + len(strings)] = strings
    struct.pack_into('<IBBHQQ', data, 0xe0 + 24, 1, 0x12, 0, 1, symbol_value, 8)
    rows = [(0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            (1, 1, text_flags, 0x1000, 0x80, 16, 0, 0, 8, 0),
            (7, 3, 0, 0, 0x90, len(names), 0, 0, 1, 0),
            (17, 3, 0, 0, 0xc0, len(strings), 0, 0, 1, 0),
            (25, 11, 0, 0, 0xe0, 48, 3, 1, 8, 24)]
    for index, row in enumerate(rows):
        struct.pack_into('<IIQQQQIIQQ', data, 0x180 + index * 64, *row)
    return data


class RunnerRegressions(unittest.TestCase):
    def setUp(self):
        self.root = Path('/tmp') / ('piv1-keyless-sbf-regression-' + uuid.uuid4().hex)
        self.root.mkdir(mode=0o700)
        self.addCleanup(shutil.rmtree, self.root)

    def file(self, relative, content=b'ordinary test marker\n'):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        return path

    def test_only_exact_locked_target_fingerprint_regular_file_is_allowed(self):
        path = self.file('target/sbpf-solana-solana/release/.fingerprint/'
                         'solana-sysvar-id-3587705b029f3e0b/lib-solana_sysvar_id.json')
        with patch.object(Path, 'open', side_effect=AssertionError('audit must not read payloads')):
            paths = runner.audit_outputs(self.root)
        self.assertIn(str(path.relative_to(self.root)), paths)

    def test_fingerprint_exception_near_misses_remain_sensitive(self):
        base = 'target/sbpf-solana-solana/release/.fingerprint/'
        cases = [
            'lib-solana_sysvar_id.json',
            base + 'solana-sysvar-id-3587705b029f3e0b/id.json',
            base + 'solana-sysvar-id-3587705b029f3e0b/other-solana_sysvar_id.json',
            base + 'solana-sysvar-id-3587705b029f3e0/lib-solana_sysvar_id.json',
            base + 'solana-sysvar-id-3587705b029f3e0z/lib-solana_sysvar_id.json',
            base + 'solana-sysvar-id-2.2.1-3587705b029f3e0b/lib-solana_sysvar_id.json',
            base + 'other-3587705b029f3e0b/lib-solana_sysvar_id.json',
            'target/release/.fingerprint/solana-sysvar-id-3587705b029f3e0b/lib-solana_sysvar_id.json',
            'target/sbpf-solana-solana/debug/.fingerprint/solana-sysvar-id-3587705b029f3e0b/lib-solana_sysvar_id.json',
        ]
        for index, relative in enumerate(cases):
            with self.subTest(relative=relative):
                separate = self.root / str(index)
                path = separate / relative
                path.parent.mkdir(parents=True)
                path.write_bytes(b'marker')
                with patch.object(Path, 'open', side_effect=AssertionError('sensitive payload read')):
                    with self.assertRaisesRegex(RuntimeError, 'sensitive output name'):
                        runner.audit_outputs(separate)

    def test_fingerprint_symlink_directory_and_fifo_are_not_exempt(self):
        relative = ('target/sbpf-solana-solana/release/.fingerprint/'
                    'solana-sysvar-id-3587705b029f3e0b/lib-solana_sysvar_id.json')
        for kind in ('symlink', 'directory', 'fifo'):
            with self.subTest(kind=kind):
                separate = self.root / kind
                path = separate / relative
                path.parent.mkdir(parents=True)
                if kind == 'symlink':
                    path.symlink_to(self.root / 'absent')
                elif kind == 'directory':
                    path.mkdir()
                else:
                    os.mkfifo(path, mode=0o600)
                with patch.object(Path, 'open', side_effect=AssertionError('nonregular payload read')):
                    with self.assertRaises(RuntimeError):
                        runner.audit_outputs(separate)

    def test_other_sensitive_names_and_symlinks_remain_rejected_without_reads(self):
        for name in ('id.json', 'test-keypair.json', 'credentials.json', 'wallet.json', 'x.pem'):
            with self.subTest(name=name):
                separate = self.root / name.replace('.', '-')
                separate.mkdir()
                (separate / name).write_bytes(b'marker only')
                with patch.object(Path, 'open', side_effect=AssertionError('sensitive payload read')):
                    with self.assertRaisesRegex(RuntimeError, 'sensitive output name'):
                        runner.audit_outputs(separate)

    def test_known_diagnostic_logs_require_regular_non_symlink_files(self):
        self.file('build.stdout', b'')
        (self.root / 'build.stderr').symlink_to(self.root / 'absent')
        with self.assertRaisesRegex(RuntimeError, 'diagnostic log is not a regular file'):
            runner.classify_diagnostics(self.root)

    def test_compiler_zero_does_not_hide_stack_target_or_call_overwrite_errors(self):
        lines = [
            'Error: Function f overflows the maximum allowed frame space. Estimated function frame size: 10688 bytes.',
            'Error: A function call in method f overwrites values in the frame.',
            'warning: stack frame too large',
            'LLVM ERROR: unsupported relocation',
            'error[E0425]: unresolved name',
        ]
        self.file('build.stdout', b'')
        self.file('build.stderr', ('\n'.join(lines) + '\nwarning: unused function\n').encode())
        self.assertEqual([match['text'] for match in runner.classify_diagnostics(self.root)], lines)

    def test_explicit_elf_machine_set_records_each_actual_machine(self):
        for machine in (247, 263):
            with self.subTest(machine=machine):
                path = self.file(str(machine) + '.elf', synthetic_elf(machine=machine))
                summary = runner.elf_summary(path)
                self.assertEqual(summary['machine'], machine)
                self.assertEqual(summary['flags'], 0)
                self.assertEqual(summary['executable_entry_segments'], [0])

    def test_wrong_machine_flags_entry_and_mapping_remain_rejected(self):
        cases = [{'machine': 62}, {'machine': 0}, {'flags': 1}, {'entry': 0x1010},
                 {'entry': 0x1001}, {'text_flags': 2}, {'segment_flags': 4},
                 {'segment_offset': 0x88}, {'symbol_value': 0x1008}]
        for index, arguments in enumerate(cases):
            with self.subTest(arguments=arguments):
                path = self.file(str(index) + '.elf', synthetic_elf(**arguments))
                with self.assertRaises(RuntimeError):
                    runner.elf_summary(path)

    def test_elf_table_and_section_bounds_remain_checked(self):
        for location in ('table', 'section'):
            with self.subTest(location=location):
                data = synthetic_elf()
                if location == 'table':
                    struct.pack_into('<Q', data, 40, 0x10000)
                else:
                    struct.pack_into('<Q', data, 0x180 + 64 + 32, 0x10000)
                path = self.file(location + '.elf', data)
                with self.assertRaisesRegex(RuntimeError, 'outside file'):
                    runner.elf_summary(path)

    def test_existing_directory_file_and_dangling_link_are_untouched(self):
        for kind in ('directory', 'file', 'symlink'):
            path = Path(str(self.root) + '-' + kind)
            if kind == 'directory':
                path.mkdir(); self.addCleanup(shutil.rmtree, path)
            elif kind == 'file':
                path.write_bytes(b'marker'); self.addCleanup(path.unlink)
            else:
                path.symlink_to(self.root / 'absent'); self.addCleanup(path.unlink)
            before = path.lstat()
            with patch.object(sys, 'argv', [str(SCRIPT), '--output', str(path)]):
                with self.assertRaisesRegex(RuntimeError, 'already exists'):
                    runner.main()
            self.assertEqual(path.lstat(), before)

    def test_wrong_review_hashes_refuse_before_output_creation(self):
        path = Path(str(self.root) + '-execute')
        for extra in ([], ['--approved-runner-sha256', runner.digest(SCRIPT),
                           '--approved-pins-sha256', '0' * 64]):
            with self.subTest(extra=extra):
                with patch.object(sys, 'argv', [str(SCRIPT), '--output', str(path), '--execute', *extra]):
                    with self.assertRaisesRegex(RuntimeError, 'hash required/mismatched'):
                        runner.main()
                self.assertFalse(os.path.lexists(path))

    def test_malformed_pin_data_has_durable_failure_without_compiler_calls(self):
        for index, content in enumerate((b'{', b'{"schema":2}', b'[]')):
            with self.subTest(content=content):
                output = Path(str(self.root) + '-bad-pin-' + str(index))
                self.addCleanup(lambda path=output: shutil.rmtree(path) if path.exists() else None)
                pin = self.file('bad-pin-' + str(index) + '.json', content)
                with patch.object(runner, 'PINS', pin), \
                        patch.object(sys, 'argv', [str(SCRIPT), '--output', str(output)]), \
                        patch.object(runner.Run, 'command') as command, \
                        contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(runner.main(), 1)
                    command.assert_not_called()
                result = json.loads((output / 'result.json').read_text())
                self.assertEqual(result['status'], 'FAIL')
                self.assertTrue(result['error'])
                self.assertIn('pin verification incomplete', result['preservation_error'])
                self.assertFalse(result['target_build_started'])

    def test_nonregular_and_symlink_pin_companions_refuse_before_reading(self):
        for kind in ('directory', 'symlink'):
            with self.subTest(kind=kind):
                output = Path(str(self.root) + '-bad-companion-' + kind)
                pin = self.root / ('pin-' + kind)
                if kind == 'directory':
                    pin.mkdir()
                else:
                    pin.symlink_to(self.root / 'absent')
                with patch.object(runner, 'PINS', pin), \
                        patch.object(sys, 'argv', [str(SCRIPT), '--output', str(output)]), \
                        patch.object(Path, 'open', side_effect=AssertionError('pin read')):
                    with self.assertRaisesRegex(RuntimeError, 'pin companion must be a regular file'):
                        runner.main()
                self.assertFalse(output.exists())

    def combined_failure(self, drift):
        output = Path(str(self.root) + '-combined')
        self.addCleanup(lambda: shutil.rmtree(output) if output.exists() else None)
        pin = self.file('pins.json', json.dumps({'schema': 1, 'source_files': {'source': 'hash'},
                                               'baseline_head': 'reviewed'}).encode())
        state = {'user': 'jerem', 'branch': 'integration/piv1-testnet', 'head': 'actual',
                 'refs': 'protected refs', 'status': ''}

        def mock_compiler(run, name, argv, **options):
            self.assertEqual(name, 'build')
            self.assertEqual(argv[0], str(runner.CARGO))
            self.assertIn('--locked', argv); self.assertIn('--offline', argv)
            self.assertNotIn('RUSTC_WRAPPER', run.env)
            run.commands.append({'name': name, 'argv': argv, 'exit_code': 0})
            (run.output / 'build.stdout').write_bytes(b'')
            (run.output / 'build.stderr').write_bytes(b'Error: A function call overwrites values in the frame.\n')
            (run.output / 'test-keypair.json').write_bytes(b'plain test marker, no key material')
            return 0

        argv = [str(SCRIPT), '--output', str(output), '--execute',
                '--approved-runner-sha256', runner.digest(SCRIPT),
                '--approved-pins-sha256', runner.digest(pin)]
        with contextlib.ExitStack() as stack:
            stack.enter_context(patch.object(sys, 'argv', argv))
            stack.enter_context(patch.object(runner, 'PINS', pin))
            stack.enter_context(patch.object(runner, 'verify_pins'))
            stack.enter_context(patch.object(runner, 'verify_baseline'))
            stack.enter_context(patch.object(runner, 'configuration_absence', return_value=[]))
            stack.enter_context(patch.object(runner, 'git_state', return_value=state))
            stack.enter_context(patch.object(runner, 'sources', return_value={'source': 'drift' if drift else 'hash'}))
            stack.enter_context(patch.object(runner, 'preflight'))
            stack.enter_context(patch.object(runner.Run, 'command', mock_compiler))
            stack.enter_context(contextlib.redirect_stdout(io.StringIO()))
            status = runner.main()
        self.assertEqual(status, 1)
        result = json.loads((output / 'result.json').read_text())
        self.assertEqual(result['status'], 'FAIL')
        self.assertEqual(result['compiler_exit_code'], 0)
        self.assertEqual(len(result['diagnostic_errors']), 1)
        self.assertIn('test-keypair.json', result['output_audit_error'])
        self.assertIn('build.stderr', result['logs'])
        self.assertEqual(result['logs']['build.stderr']['sha256'], runner.digest(output / 'build.stderr'))
        self.assertEqual(stat.S_IMODE(output.stat().st_mode), 0o700)
        if drift:
            self.assertIn('source drift', result['preservation_error'])
            self.assertNotIn('preservation', result)
        else:
            self.assertIn('protected Git refs unchanged', result['preservation'])
            self.assertNotIn('preservation_error', result)

    def test_preflight_audit_failure_never_enters_target_build(self):
        output = Path(str(self.root) + '-preflight-failure')
        self.addCleanup(lambda: shutil.rmtree(output) if output.exists() else None)
        pin = self.file('pins.json', json.dumps({'schema': 1, 'source_files': {'source': 'hash'},
                                               'baseline_head': 'reviewed'}).encode())
        state = {'user': 'jerem', 'branch': 'integration/piv1-testnet', 'head': 'actual',
                 'refs': 'protected refs', 'status': ''}
        argv = [str(SCRIPT), '--output', str(output), '--execute',
                '--approved-runner-sha256', runner.digest(SCRIPT),
                '--approved-pins-sha256', runner.digest(pin)]
        def bad_preflight(run, pins):
            (run.output / 'unexpected-wallet.json').write_bytes(b'plain marker only')
        with contextlib.ExitStack() as stack:
            stack.enter_context(patch.object(sys, 'argv', argv))
            stack.enter_context(patch.object(runner, 'PINS', pin))
            stack.enter_context(patch.object(runner, 'verify_pins'))
            stack.enter_context(patch.object(runner, 'verify_baseline'))
            stack.enter_context(patch.object(runner, 'configuration_absence', return_value=[]))
            stack.enter_context(patch.object(runner, 'git_state', return_value=state))
            stack.enter_context(patch.object(runner, 'sources', return_value={'source': 'hash'}))
            stack.enter_context(patch.object(runner, 'preflight', bad_preflight))
            command = stack.enter_context(patch.object(runner.Run, 'command'))
            stack.enter_context(contextlib.redirect_stdout(io.StringIO()))
            self.assertEqual(runner.main(), 1)
            command.assert_not_called()
        result = json.loads((output / 'result.json').read_text())
        self.assertFalse(result['target_build_started'])
        self.assertIn('unexpected-wallet.json', result['output_audit_error'])
        self.assertIn('protected Git refs unchanged', result['preservation'])

    def test_combined_compiler_and_audit_failure_retains_successful_preservation(self):
        self.combined_failure(False)

    def test_compiler_audit_and_preservation_failures_are_all_retained(self):
        self.combined_failure(True)


if __name__ == '__main__':
    unittest.main(verbosity=2)
