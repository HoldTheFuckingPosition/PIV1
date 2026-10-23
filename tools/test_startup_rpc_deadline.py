"""Absolute public-RPC deadline: fake processes/clocks, no transport child/network.

The worker literal runs only in-process with fake HTTP and standard streams.
Controller cases use historical signatures and retained synthetic host seams.
"""
import base64
from contextlib import contextmanager, ExitStack
import http.client
import importlib.util
import io
import json
import os
from pathlib import Path
import resource
import ssl
import subprocess
from types import SimpleNamespace
import unittest
from unittest import mock

HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location("deadline_lifetime_helpers",HERE/"test_startup_lifetime_capture.py")
t=importlib.util.module_from_spec(spec);spec.loader.exec_module(t);o=t.o


class Stream:
    def __init__(self,number):self.number=number;self.closed=False
    def fileno(self):return self.number
    def close(self):self.closed=True


@contextmanager
def fake_child(frame=b'',*,drip=False,exitcode=0,stall_exit=False,output_bound=None,group_kill_error=False):
    """Only the parent's process/pipe boundary is fake; no child is started."""
    tick=[0.0];events=[];registered={};chunks=iter((frame,b''));killed=[False]
    process=SimpleNamespace(stdin=Stream(101),stdout=Stream(102),pid=424242,returncode=None)
    def wait(timeout=None):
        events.append(("wait",timeout))
        if timeout is not None and stall_exit:
            tick[0]+=timeout;raise subprocess.TimeoutExpired("fake-public-child",timeout)
        process.returncode=-9 if killed[0] else exitcode
        return process.returncode
    process.wait=mock.Mock(side_effect=wait)
    def kill(pid,signal):
        events.append(("kill",pid,signal))
        if group_kill_error:raise OSError("injected group signal failure")
        killed[0]=True
    def fallback():events.append(("fallback-kill",));killed[0]=True
    process.kill=mock.Mock(side_effect=fallback)
    selector=mock.Mock()
    selector.register.side_effect=lambda stream,event:registered.update({stream:event})
    selector.unregister.side_effect=lambda stream:registered.pop(stream)
    selector.get_map.side_effect=lambda:registered
    def select(timeout):
        tick[0]+=timeout
        return [(SimpleNamespace(fileobj=stream),event) for stream,event in list(registered.items())]
    selector.select.side_effect=select
    with ExitStack() as stack:
        popen=stack.enter_context(mock.patch.object(o.subprocess,"Popen",return_value=process))
        stack.enter_context(mock.patch.object(o.selectors,"DefaultSelector",return_value=selector))
        stack.enter_context(mock.patch.object(o.os,"set_blocking"))
        stack.enter_context(mock.patch.object(o.os,"write",side_effect=lambda fd,raw:len(raw)))
        stack.enter_context(mock.patch.object(o.os,"read",side_effect=(lambda fd,size:b'{' if drip else next(chunks))))
        stack.enter_context(mock.patch.object(o.os,"killpg",side_effect=kill))
        stack.enter_context(mock.patch.object(o.time,"monotonic",side_effect=lambda:tick[0]))
        stack.enter_context(mock.patch.dict(os.environ,{key:"" for key in ("HTTP_PROXY","HTTPS_PROXY","ALL_PROXY","http_proxy","https_proxy","all_proxy")}))
        stack.enter_context(mock.patch.dict(os.environ,{key:"" for key in o.RPC_TLS_ENV}))
        if output_bound is not None:stack.enter_context(mock.patch.object(o,"RPC_OUTPUT_BOUND",output_bound))
        yield SimpleNamespace(process=process,popen=popen,selector=selector,events=events,tick=tick,killed=killed)


class RpcDeadlineTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        t.LifetimeCaptureTests.setUpClass();cls.c=t.t.StartupManifestTests.c

    def response(self,body=b'[]',**changes):
        frame={"schema":"piv1-startup-rpc-child-v1","kind":"RESPONSE","status":200,"headers":[["content-type","application/json"]],"bodyBase64":base64.b64encode(body).decode()}
        frame.update(changes);return o.encode(frame)

    def test_fixed_isolated_process_contract_and_complete_success(self):
        with fake_child(self.response(b'public-body')) as child:
            self.assertEqual(o.public_rpc_transport(self.c,b'[]'),(200,[["content-type","application/json"]],b'public-body'))
        argv=child.popen.call_args.args[0]
        self.assertEqual(argv,[str(Path(o.sys.executable).resolve()),"-I","-S","-c",o.RPC_TRANSPORT_SCRIPT])
        options=child.popen.call_args.kwargs
        self.assertEqual(options["env"],{"PIV1_PUBLIC_CHECK_REPO":str(HERE.parent)})
        self.assertTrue(options["start_new_session"]);self.assertTrue(options["close_fds"])
        self.assertIs(options["stderr"],subprocess.DEVNULL);self.assertNotIn("shell",options)
        self.assertFalse(child.killed[0]);self.assertEqual(child.process.returncode,0)
        self.assertTrue(child.process.stdin.closed and child.process.stdout.closed)

    def test_drip_and_exit_stall_hit_absolute_deadline_kill_group_and_reap(self):
        for options in ({"drip":True},{"frame":self.response(),"stall_exit":True}):
            with self.subTest(options=options),fake_child(**options) as child:
                with self.assertRaises(self.c.TransportFailure) as stopped:o.public_rpc_transport(self.c,b'[]')
                self.assertEqual(stopped.exception.partial,b'')
            self.assertTrue(child.killed[0]);self.assertEqual(child.process.returncode,-9)
            self.assertGreaterEqual(child.tick[0],30)
            kinds=[event[0] for event in child.events]
            self.assertLess(kinds.index("kill"),len(kinds)-1);self.assertEqual(kinds[-1],"wait")
            self.assertTrue(child.process.stdin.closed and child.process.stdout.closed)
        with fake_child(drip=True,group_kill_error=True) as child:
            with self.assertRaises(self.c.TransportFailure):o.public_rpc_transport(self.c,b'[]')
        self.assertEqual([event[0] for event in child.events],["kill","fallback-kill","wait"])
        child.process.kill.assert_called_once();self.assertEqual(child.process.returncode,-9)

    def test_strict_frames_output_bounds_and_only_complete_error_partial_is_retained(self):
        partial=b'actual-public-http-partial'
        error=o.encode({"schema":"piv1-startup-rpc-child-v1","kind":"TRANSPORT_FAILURE","partialBase64":base64.b64encode(partial).decode()})
        for frame,options,expected in ((error,{},partial),(error,{"exitcode":1},b''),(b'{"partialBase64":"uncompleted',{},b''),(self.response()+b' ',{},b''),(self.response(bodyBase64='***'),{},b''),(self.response(headers=[["a",1]]),{},b''),(self.response(extra=True),{},b''),(self.response(),{"output_bound":8},b'')):
            with self.subTest(frame=frame[:80],options=options),fake_child(frame,**options) as child:
                with self.assertRaises(self.c.TransportFailure) as stopped:o.public_rpc_transport(self.c,b'[]')
                self.assertEqual(stopped.exception.partial,expected)
            self.assertIsNotNone(child.process.returncode)
            self.assertTrue(child.process.stdin.closed and child.process.stdout.closed)

    def test_parent_proxy_and_source_pins_refuse_before_spawn(self):
        with mock.patch.object(o.subprocess,"Popen",side_effect=AssertionError("must not spawn")) as popen:
            with mock.patch.dict(os.environ,{"HTTPS_PROXY":"http://unsupported.invalid"}):
                with self.assertRaisesRegex(o.Stop,"proxy_environment_unsupported"):o.public_rpc_transport(self.c,b'[]')
            for key in o.RPC_TLS_ENV:
                with self.subTest(environment=key),mock.patch.dict(os.environ,{name:"" for name in self.c.PROXIES}),mock.patch.dict(os.environ,{key:"unsupported-public-config"}):
                    with self.assertRaisesRegex(o.Stop,"rpc_tls_environment_unsupported"):o.public_rpc_transport(self.c,b'[]')
            with mock.patch.dict(os.environ,{key:"" for key in self.c.PROXIES}),mock.patch.object(Path,"read_bytes",return_value=b'changed collector'):
                with self.assertRaisesRegex(o.Stop,"rpc_collector_source_pin"):o.public_rpc_transport(self.c,b'[]')
        popen.assert_not_called()
        with mock.patch.object(Path,"read_bytes",return_value=b'changed collector'),mock.patch.object(http.client,"HTTPSConnection",side_effect=AssertionError("must not open HTTP")) as https,mock.patch.object(resource,"setrlimit"),mock.patch.dict(os.environ,{"PIV1_PUBLIC_CHECK_REPO":str(HERE.parent)}):
            with self.assertRaisesRegex(ValueError,"collector_source_pin"):
                exec(compile(o.RPC_TRANSPORT_SCRIPT,"<fixed-rpc-worker>","exec"),{"__name__":"worker_source_refusal_only"})
        https.assert_not_called()
        pinned=(HERE/"collect_startup_public_evidence.py").read_bytes()
        with fake_child(self.response()),mock.patch.object(Path,"read_bytes",side_effect=[pinned,b'changed after child']):
            with self.assertRaises(self.c.TransportFailure):o.public_rpc_transport(self.c,b'[]')

    def test_exact_worker_literal_uses_pinned_tls_and_matches_parent_frames(self):
        for failed in (False,True):
            with self.subTest(failed=failed):
                response=mock.Mock(status=200,length=None)
                response.getheaders.return_value=[("content-type","application/json")]
                response.read.side_effect=http.client.IncompleteRead(b'public-partial') if failed else [b'public-body',b'']
                connection=mock.Mock();connection.getresponse.return_value=response;output=io.BytesIO()
                with mock.patch.object(o.sys,"stdin",SimpleNamespace(buffer=io.BytesIO(b'[]'))),mock.patch.object(o.sys,"stdout",SimpleNamespace(buffer=output)),mock.patch.object(http.client,"HTTPSConnection",return_value=connection) as https,mock.patch.object(ssl,"create_default_context",return_value="fake-TLS-context"),mock.patch.object(resource,"setrlimit"),mock.patch.dict(os.environ,{**{key:"" for key in self.c.PROXIES},"PIV1_PUBLIC_CHECK_REPO":str(HERE.parent)}):
                    exec(compile(o.RPC_TRANSPORT_SCRIPT,"<fixed-rpc-worker>","exec"),{"__name__":"worker_fake_http_only"})
                https.assert_called_once_with("api.testnet.solana.com",443,timeout=30,context="fake-TLS-context")
                self.assertEqual(connection.request.call_args.args,("POST","/"));connection.close.assert_called_once()
                frame=output.getvalue();self.assertEqual(o.encode(json.loads(frame)),frame)
                with fake_child(frame):
                    if failed:
                        with self.assertRaises(self.c.TransportFailure) as stopped:o.public_rpc_transport(self.c,b'[]')
                        self.assertEqual(stopped.exception.partial,b'public-partial')
                    else:self.assertEqual(o.public_rpc_transport(self.c,b'[]')[2],b'public-body')

    def test_production_rpc_routing_keeps_injected_transport_and_collectors_separate(self):
        c=self.c;raw=b'[]';injected=mock.Mock(return_value=(200,[],b'[]'))
        with mock.patch.object(o,"public_rpc_transport",return_value=(200,[],b'[]')) as production:
            self.assertEqual(o.bounded_transport(c,None)(raw),(200,[],b'[]'));production.assert_called_once_with(c,raw)
            self.assertEqual(o.bounded_transport(c,injected)(raw),(200,[],b'[]'));injected.assert_called_once_with(raw)
            self.assertEqual(production.call_count,1)
        # The unchanged native collector still uses its own tls_post, never
        # the operator wrapper or an injected mode supplied on its behalf.
        with mock.patch.object(o,"public_rpc_transport",side_effect=AssertionError("collector mode must not route here")),mock.patch.object(c,"tls_post",side_effect=RuntimeError("fake native collector stop")) as native:
            report=c.collect({"schema":c.SCHEMA,"wireBase64":t.t.h.FIXTURE["transactions"][1]["signedWireBase64"],"expectedWireSha256":t.t.h.FIXTURE["transactions"][1]["signedWireSha256"],"expectedMessageSha256":t.t.h.FIXTURE["transactions"][1]["messageSha256"],"signature":t.t.h.FIXTURE["transactions"][1]["signature"],"expectedFeeLamports":t.t.h.FIXTURE["transactions"][1]["feeLamports"],"accountKeys":list(t.t.h.FIXTURE["fundPoststate"]["accounts"]),"minimumContextSlot":"1","includeExpiry":False},Path(self._temporary_directory())/"collection")
            native.assert_called_once();self.assertEqual(report["transportMode"],"FIXED_ORIGIN_TLS")

    def _temporary_directory(self):
        temporary=__import__('tempfile').TemporaryDirectory();self.addCleanup(temporary.cleanup);return temporary.name

    def test_post_intent_deadline_retains_one_unknown_and_blocks_resend_or_reprepare(self):
        e=t.Environment(self);e.capture();e.signed();attempts=[]
        def transport(raw):
            if any(q["method"]=="sendTransaction" for q in json.loads(raw)):
                self.assertEqual(e.records()[-1]["kind"],"INTENT")
                with fake_child(drip=True) as child:
                    attempts.append(child);return o.public_rpc_transport(e.host.c,raw)
            return e.transport(raw)
        result=e.host.call("SEND",{"id":e.pid},_transport=transport,**e.host.auth())
        records=e.records();self.assertEqual([r["kind"] for r in records][-2:],["INTENT","UNKNOWN"])
        self.assertEqual(sum(r["kind"]=="INTENT" for r in records),1);self.assertEqual(len(attempts),1)
        self.assertTrue(attempts[0].killed[0]);self.assertEqual(attempts[0].process.returncode,-9)
        intent=next(r["data"] for r in records if r["kind"]=="INTENT");evidence=Path(intent["evidencePath"])
        self.assertEqual(len(list(evidence.glob('*.failure.json'))),1)
        self.assertEqual(list(evidence.glob('*.partial-response.bin')),[])
        self.assertEqual(result["status"],"UNKNOWN_OR_SUBMITTED_RECOVERY_ONLY")
        with self.assertRaisesRegex(o.Stop,"intent_exists_recovery_only"):e.host.call("SEND",{"id":e.pid},_transport=transport,**e.host.auth())
        with self.assertRaisesRegex(o.Stop,"unresolved_intent_blocks_prepare"):e.capture()
        self.assertEqual(len(attempts),1);self.assertEqual(e.host.call("INSPECT")["tail"],result["tail"])

    def test_quote_and_preflight_deadlines_stop_before_any_intent_or_send(self):
        e=t.Environment(self);before=e.records()
        def timeout(raw):
            with fake_child(drip=True):return o.public_rpc_transport(e.host.c,raw)
        with self.assertRaises(e.host.c.TransportFailure):
            e.host.call("PREPARE",{"captureLifetime":True,"packets":[{"id":e.pid,"wireBase64":e.host.prepared["wireBase64"]}]},_transport=timeout,**e.host.auth())
        self.assertEqual(e.records(),before);e.capture();e.signed()
        with self.assertRaises(e.host.c.TransportFailure):e.host.call("SEND",{"id":e.pid},_transport=timeout,**e.host.auth())
        self.assertFalse(any(r["kind"]=="INTENT" for r in e.records()));self.assertFalse(e.sent)


if __name__=="__main__":unittest.main()
