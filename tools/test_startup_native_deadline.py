"""Native acceptance clocks with fake processes/checker returns only.

No real signer child, private profile access, signature generation or network.
Existing signer-controller regressions retain the durable attempt/HOLD checks.
"""
from contextlib import contextmanager
import importlib.util
import json
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest import mock

HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location("native_deadline_backend_helpers",HERE/"test_startup_signer_backend.py")
b=importlib.util.module_from_spec(spec);spec.loader.exec_module(b);o=b.o


@contextmanager
def fake_runner(output=b'{}',*,pins_elapsed=0,spawn_elapsed=0,parse_elapsed=0,cleanup_elapsed=0):
    """Scope global pipe/process mocks to invoke_signer, never host replay."""
    tick=[0.0];registered={};reads=iter((output,b''))
    stdin=mock.Mock(closed=False);stdout=mock.Mock(closed=False)
    stdin.fileno.return_value=101;stdout.fileno.return_value=102
    process=mock.Mock(stdin=stdin,stdout=stdout);process.poll.return_value=0;process.wait.return_value=0
    selector=mock.Mock()
    selector.register.side_effect=lambda stream,event:registered.update({stream:event})
    selector.unregister.side_effect=lambda stream:registered.pop(stream)
    selector.get_map.side_effect=lambda:registered
    selector.select.side_effect=lambda wait:[(SimpleNamespace(fileobj=stream),event) for stream,event in list(registered.items())]
    def advance(value):tick[0]+=value
    selector.close.side_effect=lambda:advance(cleanup_elapsed)
    def pins():advance(pins_elapsed);return HERE.parent
    def parse(raw):advance(parse_elapsed);return json.loads(raw)
    j=SimpleNamespace(pins=pins,NODE=Path("/pinned/node"),parse=parse)
    def spawn(*args,**kwargs):advance(spawn_elapsed);return process
    with mock.patch.object(o.subprocess,"Popen",side_effect=spawn) as popen,mock.patch.object(o.selectors,"DefaultSelector",return_value=selector),mock.patch.object(o.os,"set_blocking"),mock.patch.object(o.os,"write",side_effect=lambda fd,raw:len(raw)),mock.patch.object(o.os,"read",side_effect=lambda fd,size:next(reads)),mock.patch.object(o.time,"monotonic",side_effect=lambda:tick[0]):
        yield SimpleNamespace(j=j,process=process,selector=selector,popen=popen,tick=tick)


class NativeDeadlineTests(unittest.TestCase):
    def test_signer_entry_launch_and_remaining_wait_share_twenty_seconds(self):
        with fake_runner(pins_elapsed=20) as f:
            with self.assertRaisesRegex(o.Stop,"^signer_attempt_unresolved$"):o.invoke_signer(f.j,{})
            f.popen.assert_not_called();f.selector.close.assert_called_once()
        with fake_runner(spawn_elapsed=21) as f:
            f.process.poll.return_value=None
            with self.assertRaisesRegex(o.Stop,"^signer_attempt_unresolved$"):o.invoke_signer(f.j,{})
            f.process.kill.assert_called_once();f.process.wait.assert_called_once_with()
        with fake_runner(pins_elapsed=3,spawn_elapsed=4,parse_elapsed=12,cleanup_elapsed=.5) as f:
            self.assertEqual(o.invoke_signer(f.j,{}),{})
            f.process.wait.assert_called_once_with(timeout=13)
            self.assertEqual(f.tick[0],19.5)

    def test_signer_parse_and_cleanup_cannot_accept_late_complete_response(self):
        for parse_elapsed,cleanup_elapsed in ((20,0),(0,20),(19,.999)):
            with self.subTest(parse=parse_elapsed,cleanup=cleanup_elapsed),fake_runner(parse_elapsed=parse_elapsed,cleanup_elapsed=cleanup_elapsed) as f:
                if parse_elapsed+cleanup_elapsed<20:self.assertEqual(o.invoke_signer(f.j,{}),{})
                else:
                    with self.assertRaisesRegex(o.Stop,"^signer_attempt_unresolved$"):o.invoke_signer(f.j,{})
                f.process.wait.assert_called_once_with(timeout=20)
                f.selector.close.assert_called_once()

    def test_checker_payload_and_return_are_inside_acceptance_deadline(self):
        phase={"id":"phase","feeCapLamports":"1"}
        prepared={"reviewed":{},"lifetime":{},"feeQuoteLamports":"1"}
        signed={"wireBase64":"public","wireSha256":"a"*64,"signatureHex":"b"*128}
        for elapsed in (19.999,20,21):
            tick=[0.0];report={"reason":"PENDING_OR_UNKNOWN_NO_REPLACEMENT"}
            def checked(request):
                self.assertEqual(len(request[0]["trace"]),2);tick[0]=elapsed;return [report]
            j=SimpleNamespace(_node_request=mock.Mock(side_effect=checked))
            with self.subTest(elapsed=elapsed),mock.patch.object(o.time,"monotonic",side_effect=lambda:tick[0]):
                if elapsed<20:self.assertIs(o.check_signed(j,{},phase,prepared,signed),report)
                else:
                    with self.assertRaisesRegex(o.Stop,"^signed_checker_deadline$"):o.check_signed(j,{},phase,prepared,signed)
            j._node_request.assert_called_once()
        tick=[0.0];real_encode=o.encode
        def slow_trace(value):tick[0]=20;return real_encode(value)
        j=SimpleNamespace(_node_request=mock.Mock())
        with mock.patch.object(o.time,"monotonic",side_effect=lambda:tick[0]),mock.patch.object(o,"encode",side_effect=slow_trace):
            with self.assertRaisesRegex(o.Stop,"^signed_checker_deadline$"):o.check_signed(j,{},phase,prepared,signed)
        j._node_request.assert_not_called()

if __name__=="__main__":unittest.main()
