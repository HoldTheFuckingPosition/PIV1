#!/usr/bin/python3
"""Small runner boundary checks; no Bank build or execution."""
import errno
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

PATH = Path(__file__).with_name('validate_bank_smoke.py')
spec = importlib.util.spec_from_file_location('bank_runner', PATH)
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class RunnerBoundaries(unittest.TestCase):
    def test_environment_is_scrubbed_and_offline(self):
        with patch.dict(os.environ, {'HOME': '/must-not-inherit', 'TOKEN': 'fixture'}):
            env = runner.environment(Path('/tmp/piv1-bank-smoke-test'))
        self.assertNotIn('HOME', env)
        self.assertNotIn('TOKEN', env)
        self.assertEqual(env['CARGO_NET_OFFLINE'], 'true')
        self.assertEqual(env['CARGO_BUILD_JOBS'], '1')
        self.assertEqual(env['RAYON_NUM_THREADS'], '1')
        self.assertEqual(env['OPENSSL_CONFIG_DIR'], '/tmp/piv1-bank-smoke-test/openssl-config')

    def test_refuses_existing_output_without_removal(self):
        with tempfile.TemporaryDirectory(prefix='piv1-bank-smoke-test-') as name:
            path = Path(name)
            sentinel = path / 'keep'
            sentinel.write_text('unchanged')
            with self.assertRaisesRegex(RuntimeError, 'existing output'):
                runner.checked_output(path, fresh=True)
            self.assertEqual(sentinel.read_text(), 'unchanged')

    def test_refuses_nonlocal_or_unscoped_outputs(self):
        for path in [Path('/home/jerem/piv1/output'), Path('/tmp/unrelated'),
                     Path('/tmp/piv1-bank-smoke-a/../b')]:
            with self.subTest(path=path), self.assertRaises(RuntimeError):
                runner.checked_output(path, fresh=True)

    def test_refuses_output_symlink(self):
        with tempfile.TemporaryDirectory(prefix='piv1-bank-smoke-test-') as name:
            link = Path(name + '-link')
            link.symlink_to(name)
            try:
                with self.assertRaises(RuntimeError):
                    runner.checked_output(link)
            finally:
                link.unlink()

    def test_hash_refuses_symlink(self):
        with tempfile.TemporaryDirectory() as name:
            path = Path(name)
            (path/'data').write_text('public fixture')
            (path/'link').symlink_to(path/'data')
            with self.assertRaises(RuntimeError):
                runner.digest(path/'link')

    def test_tree_digest_detects_changed_and_extra_source(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            (root/'source').write_text('a')
            first = runner.tree_digest(root)
            (root/'source').write_text('b')
            self.assertNotEqual(first, runner.tree_digest(root))
            (root/'source').write_text('a')
            (root/'extra').write_text('c')
            self.assertNotEqual(first, runner.tree_digest(root))

    def test_receipts_are_exclusive(self):
        with tempfile.TemporaryDirectory() as name:
            path = Path(name)/'receipt.json'
            runner.write(path, {'status': 'first'})
            with self.assertRaises(FileExistsError):
                runner.write(path, {'status': 'second'})
            self.assertEqual(json.loads(path.read_text()), {'status': 'first'})

    def test_actual_child_filter_denies_socket_and_preserves_uid(self):
        code = '''import errno,os,socket
assert os.getuid()==1001
for family in [socket.AF_INET,socket.AF_INET6,socket.AF_UNIX]:
 try: socket.socket(family,socket.SOCK_STREAM)
 except OSError as error: assert error.errno==errno.EPERM
 else: raise AssertionError('socket unexpectedly allowed')
print('socket-denial-pass')
'''
        result = subprocess.run(['/usr/bin/python3', '-I', '-B', '-c', code],
            env={'PATH': '/usr/bin:/bin', 'LC_ALL': 'C'}, stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=10,
            preexec_fn=runner.child_limits)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, b'socket-denial-pass\n')

    def test_current_process_group_memory_observation(self):
        self.assertGreater(runner.process_memory(os.getpgrp()), 0)

    def test_actual_child_allows_local_pair_but_denies_other_domains(self):
        code = '''import ctypes,errno,socket
a,b=socket.socketpair(socket.AF_UNIX,socket.SOCK_SEQPACKET)
a.send(b'local compiler IPC')
assert b.recv(64)==b'local compiler IPC'
a.close();b.close()
libc=ctypes.CDLL(None,use_errno=True)
fds=(ctypes.c_int*2)()
for domain in [socket.AF_INET,socket.AF_INET6,1+(1<<32)]:
 result=libc.syscall(ctypes.c_long(53),ctypes.c_ulong(domain),ctypes.c_int(socket.SOCK_STREAM),ctypes.c_int(0),ctypes.byref(fds))
 assert result==-1 and ctypes.get_errno()==errno.EPERM
print('local-only-IPC-pass')
'''
        result = subprocess.run(['/usr/bin/python3', '-I', '-B', '-c', code],
            env={'PATH': '/usr/bin:/bin', 'LC_ALL': 'C'}, stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=10,
            preexec_fn=runner.child_limits)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, b'local-only-IPC-pass\n')

    def test_timeout_kills_child_and_retains_receipt(self):
        with tempfile.TemporaryDirectory(prefix='piv1-bank-smoke-test-') as name:
            output = Path(name)
            with self.assertRaisesRegex(RuntimeError, 'failed; retained'):
                runner.monitored(['/usr/bin/python3', '-I', '-B', '-c',
                                  'import time; time.sleep(20)'], output, 'timeout', 0)
            record = json.loads((output/'timeout.json').read_text())
            self.assertEqual(record['guard_stop'], 'wall timeout')
            self.assertNotEqual(record['exit'], 0)
            self.assertTrue((output/'timeout.stderr').is_file())

    def test_memory_guard_kills_child_and_retains_receipt(self):
        with tempfile.TemporaryDirectory(prefix='piv1-bank-smoke-test-') as name:
            output = Path(name)
            with patch.object(runner, 'process_memory', return_value=runner.MAX_MEMORY+1):
                with self.assertRaisesRegex(RuntimeError, 'failed; retained'):
                    runner.monitored(['/usr/bin/python3', '-I', '-B', '-c',
                                      'import time; time.sleep(20)'], output, 'memory', 10)
            record = json.loads((output/'memory.json').read_text())
            self.assertEqual(record['guard_stop'], 'sampled process-group RSS+swap exceeded limit')
            self.assertEqual(record['sampled_peak_group_rss_swap_bytes'], runner.MAX_MEMORY+1)


if __name__ == '__main__':
    unittest.main(verbosity=2)
