"""Changed-path refusal tests; no RPC, native process or private input."""
import unittest

from tools.startup_write_freshness_diagnostics import require_freshness


class Stop(Exception):
    pass


class WriteFreshnessDiagnosticsTests(unittest.TestCase):
    def test_same_exact_thresholds_and_separate_fixed_predicates(self):
        for limit in (1.0, 2.0):
            for request_ok, receive_ok in ((True, False), (False, True), (False, False)):
                rows = []
                with self.subTest(limit=limit, request_ok=request_ok, receive_ok=receive_ok):
                    with self.assertRaisesRegex(Stop, '^fresh_sample_aged_HOLD$'):
                        require_freshness(limit if request_ok else limit + .001,
                                          1.0 if receive_ok else 1.001,
                                          limit, record=rows.append, stop=Stop)
                    self.assertEqual(rows, [{'schema': 'piv1-write-freshness-refusal-v1',
                                             'requestStartWithinLimit': request_ok,
                                             'receiveAgeWithinOneSecond': receive_ok}])

    def test_success_and_exact_equalities_have_no_callback(self):
        def forbidden(_):
            self.fail('success must not write or refresh')
        for limit in (1.0, 2.0):
            require_freshness(limit, 1.0, limit, record=forbidden, stop=Stop)
            require_freshness(0.1, 0.1, limit, record=forbidden, stop=Stop)

    def test_failed_durability_propagates_once_without_admission(self):
        calls = []
        def fail(row):
            calls.append(row)
            raise OSError('fake durability failure')
        with self.assertRaises(OSError):
            require_freshness(2.001, .1, 2.0, record=fail, stop=Stop)
        self.assertEqual(len(calls), 1)


if __name__ == '__main__':
    unittest.main()
