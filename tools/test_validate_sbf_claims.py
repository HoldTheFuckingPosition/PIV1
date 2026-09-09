#!/usr/bin/python3
"""Small stdlib refusal tests. They never invoke Cargo or the runtime."""
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import tarfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('runner', Path(__file__).with_name('validate_sbf_claims.py'))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class Refusals(unittest.TestCase):
    def test_environment_has_no_home_wrapper_credentials_or_network_mode(self):
        env = runner.environment(Path('/tmp/piv1-sbf-claims-test'))
        self.assertNotIn('HOME', env)
        self.assertNotIn('RUSTC_WRAPPER', env)
        self.assertEqual(env['CARGO_NET_OFFLINE'], 'true')
        self.assertEqual(env['PATH'], '/usr/bin:/bin')
        self.assertEqual(env['CC'], '/usr/bin/cc')

    def test_existing_output_is_never_reused(self):
        with tempfile.TemporaryDirectory(prefix='piv1-sbf-claims-refusal-') as directory:
            path = Path(directory); (path / 'preserve').write_bytes(b'original')
            with self.assertRaisesRegex(RuntimeError, 'already exists'):
                runner.fresh_output(path)
            self.assertEqual((path / 'preserve').read_bytes(), b'original')

    def test_symlink_pin_or_binary_is_not_read(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / 'source'; source.write_bytes(b'public fixture')
            link = Path(directory) / 'link'; link.symlink_to(source)
            with self.assertRaisesRegex(RuntimeError, 'non-symlink'):
                runner.digest(link)

    def test_archive_checksum_or_extracted_source_change_refuses(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory); archive_dir = home / 'registry/cache' / runner.REGISTRY
            source = home / 'registry/src' / runner.REGISTRY / 'sample-1.0.0'
            archive_dir.mkdir(parents=True); source.mkdir(parents=True)
            archive = archive_dir / 'sample-1.0.0.crate'
            with tarfile.open(archive, 'w:gz') as tar:
                info = tarfile.TarInfo('sample-1.0.0/lib.rs'); info.size = 5
                tar.addfile(info, io.BytesIO(b'value'))
            package = {'name': 'sample', 'version': '1.0.0', 'checksum': runner.digest(archive)}
            (source / 'lib.rs').write_bytes(b'other')
            with patch.object(runner, 'CARGO_HOME', home):
                with self.assertRaisesRegex(RuntimeError, 'source changed'):
                    runner.verify_packages([package])
                package['checksum'] = '0' * 64
                with self.assertRaisesRegex(RuntimeError, 'archive changed'):
                    runner.verify_packages([package])

    def test_unreviewed_executable_is_rejected_before_command(self):
        class NoCommand:
            def command(self, *args):
                raise AssertionError('must not invoke any command')
        with self.assertRaisesRegex(RuntimeError, 'reviewed executable hash required'):
            runner.execute(NoCommand(), Path('/tmp/piv1-sbf-claims-build-test'), None)

    def test_failed_build_preserves_diagnostics_before_refusing(self):
        with tempfile.TemporaryDirectory() as directory:
            class FailedBuild:
                output = Path(directory)
                def command(self, label, argv):
                    assert '--no-run' in argv and '--locked' in argv and '--offline' in argv
                    assert argv[argv.index('--jobs') + 1] == '1'
                    (self.output / 'build.stdout').write_text(json.dumps({'reason': 'compiler-message',
                        'message': {'level': 'error', 'message': 'fixture error'}}) + '\n')
                    (self.output / 'build.stderr').write_text('error: fixture error\n')
                    return 101
            with self.assertRaisesRegex(RuntimeError, 'diagnostics require review'):
                runner.build(FailedBuild())
            evidence = json.loads((Path(directory) / 'diagnostics.json').read_text())
            self.assertEqual(evidence['cargo_exit'], 101)
            self.assertEqual(evidence['messages'][0]['message']['message'], 'fixture error')


if __name__ == '__main__':
    unittest.main()
