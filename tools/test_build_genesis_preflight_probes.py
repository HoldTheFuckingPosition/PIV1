#!/usr/bin/python3
"""Refusal/orchestration regressions. No compiler, artifact loader or network."""
import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import struct
import sys
import tarfile
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
SCRIPT = Path('/home/jerem/piv1/tools/build_genesis_preflight_probes.py')
spec = importlib.util.spec_from_file_location('probe_runner_tests', SCRIPT)
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class ProbeGuards(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix='piv1-genesis-probes-tests-'))
        self.addCleanup(shutil.rmtree, self.root)

    def fresh_output(self):
        # mkdtemp may use underscores, which the public output grammar excludes.
        path = Path(str(self.root).replace('_', 'a') + '-output')
        self.addCleanup(lambda: shutil.rmtree(path) if path.exists() and not path.is_symlink() else None)
        return path

    def test_helper_drift_refuses_before_import_execution(self):
        changed = self.root / 'helper.py'; changed.write_text('raise AssertionError("must not import")')
        with patch.object(runner, 'HELPER', changed), patch.object(runner.importlib.util, 'spec_from_file_location') as load:
            with self.assertRaisesRegex(RuntimeError, 'import refused'):
                runner.load_helper()
            load.assert_not_called()

    def test_no_execute_authority_and_existing_outputs_refuse_without_mutation(self):
        output = self.fresh_output()
        with patch.object(sys, 'argv', [str(SCRIPT), '--output', str(output), '--execute']), \
                patch.object(runner, 'load_helper') as load:
            with self.assertRaisesRegex(RuntimeError, 'reviewed runner/pin hashes'):
                runner.main()
            self.assertFalse(output.exists()); load.assert_not_called()
        output.mkdir(); marker = output / 'retained'; marker.write_bytes(b'unchanged')
        with patch.object(sys, 'argv', [str(SCRIPT), '--output', str(output)]):
            with self.assertRaisesRegex(RuntimeError, 'already exists'):
                runner.main()
        self.assertEqual(marker.read_bytes(), b'unchanged')

    def test_configuration_and_credentials_refuse_without_reading_payload(self):
        workspace = self.root / 'probes'; workspace.mkdir()
        cargo = self.root / 'cargo'; cargo.mkdir()
        helper = SimpleNamespace(configuration_absence=lambda: [], CARGO_HOME=cargo)
        for directory, name in [(workspace/'caller', '.cargo/config.toml'), (cargo, 'credentials.toml')]:
            file = directory / name; file.parent.mkdir(parents=True, exist_ok=True); file.write_text('marker')
            with patch.object(runner, 'WORKSPACE', workspace), patch.object(Path, 'open', side_effect=AssertionError('payload read')):
                with self.assertRaisesRegex(RuntimeError, 'configuration'):
                    runner.reject_configs(helper)
            file.unlink()

    def test_lock_version_or_checksum_change_is_rejected(self):
        workspace = self.root / 'probes'; workspace.mkdir()
        (self.root/'Cargo.lock').write_bytes((runner.ROOT/'Cargo.lock').read_bytes())
        original = (runner.WORKSPACE/'Cargo.lock').read_text()
        pins = json.loads(runner.PINS.read_text())
        for replacement in ('version = "999.0.0"', 'version = "0.8.11"'):
            (workspace/'Cargo.lock').write_text(original.replace('version = "0.8.12"', replacement, 1))
            with patch.object(runner, 'ROOT', self.root), patch.object(runner, 'WORKSPACE', workspace):
                with self.assertRaisesRegex(RuntimeError, 'registry version/checksum'):
                    runner.verify_lock(pins)
        mutated = original.replace('checksum = "', 'checksum = "0', 1)
        (workspace/'Cargo.lock').write_text(mutated)
        with patch.object(runner, 'ROOT', self.root), patch.object(runner, 'WORKSPACE', workspace):
            with self.assertRaisesRegex(RuntimeError, 'registry version/checksum'):
                runner.verify_lock(pins)

    def test_probe_source_inventory_includes_nested_crates_and_rejects_symlinks(self):
        workspace=self.root/'probes'; (workspace/'caller/src').mkdir(parents=True)
        (workspace/'caller/src/lib.rs').write_text('test')
        (self.root/'tools').mkdir();(self.root/'tools/test_build_genesis_preflight_probes.py').write_text('test')
        helper=runner.load_helper()
        with patch.object(runner,'WORKSPACE',workspace),patch.object(runner,'ROOT',self.root),patch.object(runner,'SCRIPT',self.root/'tools/test_build_genesis_preflight_probes.py'):
            sources=runner.probe_sources(helper)
            self.assertIn('probes/caller/src/lib.rs',sources)
            (workspace/'caller/src/other.rs').symlink_to(self.root/'absent')
            with self.assertRaisesRegex(RuntimeError,'symlink'):
                runner.probe_sources(helper)

    def simulated_run(self, *, stderr='', malformed_elf=False, audit_failure=False, package_failure=False):
        helper=runner.load_helper();output=self.fresh_output();pin=self.root/'pins.json'
        pin.write_text(json.dumps({'schema':1,'baseline_head':'reviewed','packages':[]}))
        state={'user':'jerem','branch':'integration/piv1-testnet','head':'head','refs':'refs','status':''}
        def command(run,name,argv,**options):
            run.commands.append({'name':name,'argv':list(map(str,argv)),'exit_code':0})
            (run.output/(name+'.stdout')).write_bytes(b'')
            (run.output/(name+'.stderr')).write_text(stderr if name=='build' else '')
            if name=='build':
                self.assertIn('--locked',argv);self.assertIn('--offline',argv);self.assertIn('--workspace',argv)
                for package in runner.PACKAGES:
                    artifact=output/'target'/helper.TARGET/'release'/(package.replace('-','_')+'.so')
                    artifact.parent.mkdir(parents=True,exist_ok=True);artifact.write_bytes(b'malformed')
                if audit_failure:(output/'unexpected-wallet.json').write_text('test marker only')
            return 0
        args=[str(SCRIPT),'--output',str(output),'--execute','--approved-runner-sha256',runner.digest(SCRIPT),
              '--approved-pins-sha256',runner.digest(pin)]
        with contextlib.ExitStack() as stack:
            stack.enter_context(patch.object(runner,'PINS',pin));stack.enter_context(patch.object(sys,'argv',args))
            stack.enter_context(patch.object(runner,'load_helper',return_value=helper))
            stack.enter_context(patch.object(runner,'verify_inputs',return_value={}))
            packages=stack.enter_context(patch.object(runner,'verify_packages',return_value=[]))
            if package_failure:packages.side_effect=tarfile.ReadError('invalid package archive')
            stack.enter_context(patch.object(helper,'git_state',return_value=state))
            stack.enter_context(patch.object(helper,'verify_baseline'))
            stack.enter_context(patch.object(helper,'preflight'))
            stack.enter_context(patch.object(runner,'verify_metadata'))
            stack.enter_context(patch.object(helper.Run,'command',command))
            if malformed_elf:stack.enter_context(patch.object(helper,'elf_summary',side_effect=struct.error('truncated ELF table')))
            stack.enter_context(contextlib.redirect_stdout(io.StringIO()))
            status=runner.main()
        result=json.loads((output/'result.json').read_text())
        self.assertEqual(status,1);self.assertEqual(result['status'],'FAIL')
        return result

    def test_zero_exit_stack_error_and_independent_audit_failure_are_retained(self):
        result=self.simulated_run(stderr='Error: Function f overflows the maximum allowed frame space.\n',audit_failure=True)
        self.assertEqual(result['compiler_exit_code'],0);self.assertEqual(len(result['diagnostic_errors']),1)
        self.assertIn('unexpected-wallet.json',result['output_audit_error'])
        self.assertIn('build.stderr',result['logs']);self.assertIn('preservation',result)

    def test_zero_exit_warning_is_not_accepted_as_clean_build(self):
        result=self.simulated_run(stderr='warning: unused variable\n')
        self.assertEqual(result['diagnostic_errors'],[]);self.assertEqual(len(result['diagnostic_warnings']),1)
        self.assertNotIn('artifacts',result)

    def test_malformed_elf_failure_is_durable_with_raw_logs(self):
        result=self.simulated_run(malformed_elf=True)
        self.assertEqual(result['error'],'truncated ELF table')
        self.assertIn('build.stderr',result['logs']);self.assertIn('preservation',result)

    def test_malformed_package_archive_never_enters_compiler(self):
        result=self.simulated_run(package_failure=True)
        self.assertFalse(result['target_build_started']);self.assertEqual(result['commands'],[])
        self.assertEqual(result['error'],'invalid package archive')


if __name__ == '__main__':
    unittest.main(verbosity=2)
