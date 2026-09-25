#!/usr/bin/python3
"""Guard regressions use mocks only: no Cargo, SBF or network execution."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SCRIPT = Path('/home/jerem/piv1/tools/validate_genesis_preflight.py')
spec = importlib.util.spec_from_file_location('genesis_runner', SCRIPT)
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class Guards(unittest.TestCase):
    def test_helper_mismatch_refuses_import(self):
        with patch.object(runner, 'digest', return_value='0'*64), patch.object(runner.importlib.util, 'spec_from_file_location') as load:
            with self.assertRaisesRegex(RuntimeError, 'immutable helper'):
                runner.load_helper()
            load.assert_not_called()

    def test_sources_reject_symlink_before_read(self):
        with tempfile.TemporaryDirectory(prefix='piv1-t230-unit-') as temp:
            root = Path(temp); (root/'linked.rs').symlink_to('/does-not-exist')
            with patch.object(runner, 'WORKSPACE', root), patch.object(runner, 'digest') as read:
                with self.assertRaisesRegex(RuntimeError, 'symlink'):
                    runner.sources()
                read.assert_not_called()

    def test_sources_reject_suspicious_filename_before_read(self):
        with tempfile.TemporaryDirectory(prefix='piv1-t230-unit-') as temp:
            root=Path(temp); (root/'wallet.rs').write_text('not a wallet')
            with patch.object(runner, 'WORKSPACE', root), patch.object(runner, 'digest') as read:
                with self.assertRaisesRegex(RuntimeError, 'suspicious'):
                    runner.sources()
                read.assert_not_called()

    def test_fresh_path_refuses_existing_and_outside_tmp(self):
        for path in [Path('/home/piv1-genesis-runtime-test'),Path('/tmp/wrong-name')]:
            with self.assertRaisesRegex(RuntimeError, 'unexpected output'):
                runner.checked_output(path, fresh=True)
        with tempfile.TemporaryDirectory(prefix='piv1-genesis-runtime-unit-') as temp:
            with self.assertRaisesRegex(RuntimeError, 'already exists'):
                runner.checked_output(Path(temp),fresh=True)

    def test_output_symlink_refused(self):
        with tempfile.TemporaryDirectory(prefix='piv1-t230-unit-') as temp:
            path=Path(temp)/'link';path.symlink_to('/tmp')
            with self.assertRaisesRegex(RuntimeError,'symlink'):
                runner.audit_output(Path(temp))

    def test_public_package_name_does_not_allow_wallet_output(self):
        with tempfile.TemporaryDirectory(prefix='piv1-t230-unit-') as temp:
            root=Path(temp); directory=root/'target/solana-keypair-fixture';directory.mkdir(parents=True)
            (directory/'wallet.json').write_text('public test marker')
            with self.assertRaisesRegex(RuntimeError,'suspicious'):
                runner.audit_output(root)

    def test_registry_subset_is_checksum_sensitive(self):
        package={'name':'example','version':'1.0.0','source':'registry+x','checksum':'a'}
        changed={**package,'checksum':'b'}
        self.assertFalse(runner.registry({'package':[changed]}) <= runner.registry({'package':[package]}))

    def test_run_rejects_missing_exact_binary_hash_before_read(self):
        with patch.object(runner,'checked_output') as checked:
            for approved in [None,'','a'*63,'z'*64]:
                with self.assertRaisesRegex(RuntimeError,'reviewed executable'):
                    runner.execute(None,None,Path('/tmp/piv1-genesis-runtime-build'),approved)
            checked.assert_not_called()

    def test_run_rejects_failed_build_and_changed_binary(self):
        class Helper:
            regular=staticmethod(lambda path:path)
        with tempfile.TemporaryDirectory(prefix='piv1-genesis-runtime-unit-') as temp:
            output=Path(temp);record={'status':'FAIL','stage':'build'}
            (output/'result.json').write_text(json.dumps(record))
            with self.assertRaisesRegex(RuntimeError,'build did not pass'):
                runner.execute(Helper,None,output,'a'*64)
            binary=output/'target/runtime';binary.parent.mkdir();binary.write_bytes(b'public fixture')
            record.update(status='BUILD_PASS',runner_sha256='a'*64,pins_sha256='a'*64,
                          executable={'path':str(binary),'sha256':'b'*64})
            (output/'result.json').write_text(json.dumps(record))
            with patch.object(runner,'digest',return_value='a'*64):
                with self.assertRaisesRegex(RuntimeError,'executable drift'):
                    runner.execute(Helper,None,output,'a'*64)

    def test_changed_executable_after_run_is_rejected_even_on_test_failure(self):
        class Helper:
            regular=staticmethod(lambda path:path)
            write_json=staticmethod(lambda path,value:None)
        for exit_code in [0,1]:
            with tempfile.TemporaryDirectory(prefix='piv1-genesis-runtime-unit-') as temp:
                output=Path(temp);binary=output/'target/runtime';binary.parent.mkdir();binary.write_bytes(b'public fixture')
                record={'status':'BUILD_PASS','stage':'build','runner_sha256':'a'*64,'pins_sha256':'a'*64,
                        'executable':{'path':str(binary),'sha256':'a'*64}}
                (output/'result.json').write_text(json.dumps(record))
                class Run:
                    def __init__(self):self.output=output;self.executed=False
                    def command(self,*_):self.executed=True;return exit_code
                run=Run()
                def digest(path):return 'b'*64 if path==binary and run.executed else 'a'*64
                with patch.object(runner,'digest',side_effect=digest):
                    with self.assertRaisesRegex(RuntimeError,'executable changed during execution'):
                        runner.execute(Helper,run,output,'a'*64)

    def test_build_zero_exit_warning_rejected(self):
        class Helper:
            TOOLCHAIN=Path('/fixed/toolchain')
            write_json=staticmethod(lambda path,value:None)
        with tempfile.TemporaryDirectory(prefix='piv1-t230-unit-') as temp:
            class Run:
                output=Path(temp)
                def command(self,*_):return 0
            (Run.output/'build.stdout').write_text(json.dumps({'reason':'compiler-message','message':{'level':'warning'}})+'\n')
            (Run.output/'build.stderr').write_text('')
            with self.assertRaisesRegex(RuntimeError,'diagnostics'):
                runner.build(Helper,Run())


if __name__=='__main__':
    unittest.main()
