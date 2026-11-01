"""Focused fake-process tests; no RPC, native signer, private input or replay."""
import json
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest import mock

from tools import startup_signer_diagnostics as diagnostic
from tools import startup_two_operation as base
from tools.test_startup_native_deadline import fake_runner, o

class SignerDiagnosticsTests(unittest.TestCase):
    def setUp(self):
        self.events = []
        self.rows = []
        store = SimpleNamespace(capture=lambda f: f(), write=self.write)
        self.owner = SimpleNamespace(transport_leaf=o, signer_diagnostics=diagnostic, store=store, phase=241)
        self.addCleanup(mock.patch.stopall)
        mock.patch('socket.socket', side_effect=AssertionError('network forbidden')).start()
        mock.patch('socket.create_connection', side_effect=AssertionError('network forbidden')).start()
        mock.patch.object(o, 'signer_backend_source', return_value=Path('/unused-public-backend')).start()

    def write(self, name, raw):
        self.events.append('diagnostic')
        self.assertEqual(name, 'phase-0241-native-error.json')
        self.rows.append(json.loads(raw))

    def capture(self, raw, complete):
        self.events.append(('raw', complete))

    def gate(self, runner, failure):
        class Gate:
            pins = staticmethod(runner.j.pins)
            parse = staticmethod(runner.j.parse)

            @property
            def NODE(self):
                raise failure
        return Gate()

    def test_pre_native_refusal_is_recorded_before_same_generic_hold(self):
        with fake_runner() as f:
            with self.assertRaisesRegex(o.Stop, '^signer_attempt_unresolved$'):
                try:
                    self.call_with_gate(f, base.Stop('fresh_sample_aged_HOLD'))
                except o.Stop:
                    self.events.append('HOLD')
                    raise
            f.popen.assert_not_called()
        self.assertEqual(self.events, [('raw', False), 'diagnostic', 'HOLD'])
        self.assertEqual(self.rows[0]['exceptionType'], 'Stop')
        self.assertEqual(self.rows[0]['errorCode'], 'fresh_sample_aged_HOLD')
        site = self.rows[0]['wrapperCallsite']
        self.assertEqual(site['file'], 'tools/startup_release_operator.py')
        self.assertEqual(site['function'], 'invoke_signer')
        self.assertIs(type(site['line']), int)

    def call_with_gate(self, runner, failure):
        return self.call_for_adapter(self.gate(runner, failure))

    def call_for_adapter(self, adapter):
        return diagnostic.invoke(o, adapter, {}, raw_capture=self.capture,
                                 admission_stop=base.Stop, store=self.owner.store,
                                 phase=241, encode=base.encode)

    def test_complete_output_does_not_hide_wait_or_parse_failure(self):
        for step in ('wait', 'parse'):
            with self.subTest(step=step), fake_runner() as f:
                if step == 'wait':
                    f.process.wait.return_value = 1
                else:
                    f.j.parse = mock.Mock(side_effect=ValueError('CANARY_PRIVATE_MESSAGE'))
                with self.assertRaisesRegex(o.Stop, '^signer_attempt_unresolved$'):
                    self.call_for_adapter(f.j)
                f.popen.assert_called_once()
                self.assertIn(('raw', True), self.events)
                row = self.rows[-1]
                self.assertEqual(row['errorCode'], 'signer_backend_refused' if step == 'wait' else 'UNCLASSIFIED_SIGNER_FAILURE')
                self.assertNotIn('CANARY', json.dumps(row))

    def test_unknown_types_and_hostile_args_are_never_formatted(self):
        class Hostile:
            def __str__(self): raise AssertionError('must not format')
            def __repr__(self): raise AssertionError('must not repr')
            def __eq__(self, other): raise AssertionError('must not compare')
        class Unknown(ValueError):
            def __getattribute__(self, name): raise AssertionError('must not inspect custom attributes')
            def __str__(self): raise AssertionError('must not format')
        for failure in (base.Stop(Hostile()), Unknown('CANARY_PRIVATE_PATH')):
            with self.subTest(kind=type(failure)), fake_runner() as f:
                with self.assertRaises(o.Stop):
                    self.call_with_gate(f, failure)
                self.assertEqual(self.rows[-1]['errorCode'], 'UNCLASSIFIED_SIGNER_FAILURE')
                self.assertNotIn('CANARY', json.dumps(self.rows[-1]))

    def test_stale_outer_context_is_not_attributed_to_new_invocation(self):
        with fake_runner() as old:
            try:
                self.call_with_gate(old, base.Stop('fresh_sample_aged_HOLD'))
            except o.Stop:
                with fake_runner(cleanup_elapsed=20) as new:
                    with self.assertRaises(o.Stop):
                        self.call_for_adapter(new.j)
        self.assertEqual(self.rows[-1]['errorCode'], 'CAUSE_UNAVAILABLE')
        self.assertIsNone(self.rows[-1]['wrapperCallsite']['line'])

    def test_diagnostic_write_failure_propagates_without_retry(self):
        self.owner.store.write = mock.Mock(side_effect=OSError('CANARY_IO_FAILURE'))
        with fake_runner() as f:
            with self.assertRaises(OSError):
                self.call_with_gate(f, base.Stop('fresh_sample_aged_HOLD'))
            f.popen.assert_not_called()
        self.owner.store.write.assert_called_once()

    def test_success_does_not_write_a_diagnostic(self):
        with fake_runner() as f:
            self.assertEqual(self.call_for_adapter(f.j), {})
            f.popen.assert_called_once()
        self.assertEqual(self.rows, [])
        self.assertEqual(self.events, [('raw', True)])


if __name__ == '__main__':
    unittest.main()
