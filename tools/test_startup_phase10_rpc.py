"""Focused private-route OS seams: fake children/TLS and synthetic temp endpoints.

No network, actual child, native signature or real home-private input is used.
The inherited deadline fixture replaces the OS boundary, not the production
transport function; a complete raw response must survive a later refusal.
"""
import base64
from contextlib import contextmanager, ExitStack
import importlib.util
import io
import json
import os
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

HERE=Path(__file__).parent
def module(name,path):
    spec=importlib.util.spec_from_file_location(name,path); result=importlib.util.module_from_spec(spec); spec.loader.exec_module(result); return result
r=module("private_route_under_test",HERE/"startup_phase10_rpc.py")
d=module("private_route_deadline_fixture",HERE/"test_startup_rpc_deadline.py")
t=module("private_route_controller_fixture",HERE/"test_startup_phase10_exception.py")


@contextmanager
def fake_child(*args, **kwargs):
    """Reuse the deadline child while preserving real source/receipt file I/O."""
    real_read, real_write = os.read, os.write
    with d.fake_child(*args, **kwargs) as child:
        pipe_read, pipe_write = os.read, os.write
        with mock.patch.object(os, "read", side_effect=lambda fd, size: pipe_read(fd, size) if fd == 102 else real_read(fd, size)), \
             mock.patch.object(os, "write", side_effect=lambda fd, raw: pipe_write(fd, raw) if fd == 101 else real_write(fd, raw)):
            yield child


class Phase10RpcTests(unittest.TestCase):
    def setUp(self):
        self.core=d.o; self.c=self.core.helpers()[1]
        self.tmp=tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.directory=Path(self.tmp.name); os.chmod(self.directory,0o700)
        self.credential=self.directory/"synthetic-endpoint.url"
        self.raw=b"https://"+b"synthetic.solana-testnet.quiknode.pro/"+b"NOT_A_REAL_TOKEN_00000000/\n"
        fd=os.open(self.credential,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
        try: os.write(fd,self.raw)
        finally: os.close(fd)
        self.value=r.binding(r.sha(self.raw),r.sha((HERE/"startup_phase10_rpc.py").read_bytes()))
        self.keys=self.core.startup_manifest().required_keys({"kind":"UPLOAD_WRITE"})

    def frame(self,body):
        return r.encode({"schema":"piv1-phase10-private-rpc-worker-v1","kind":"RESPONSE","status":200,"headers":[],"bodyBase64":base64.b64encode(body).decode()})

    def test_native_worker_boundary_success_deadline_and_no_late_spawn(self):
        body=b'[{"jsonrpc":"2.0","id":1,"result":null}]'
        with fake_child(self.frame(body)) as child:
            self.assertEqual(r.transport(self.core,self.c,self.value,b'[]'),(200,[],body))
        options=child.popen.call_args.kwargs
        self.assertEqual(options["env"]["PIV1_PHASE10_ENDPOINT_SHA256"],r.sha(self.raw))
        self.assertTrue(options["start_new_session"]); self.assertTrue(child.process.stdin.closed and child.process.stdout.closed)
        for options in ({"drip":True},{"frame":self.frame(body),"stall_exit":True}):
            with self.subTest(options=options),fake_child(**options) as child:
                with self.assertRaises(self.c.TransportFailure): r.transport(self.core,self.c,self.value,b'[]')
            self.assertTrue(child.killed[0]); self.assertEqual(child.process.returncode,-9)
            self.assertEqual(child.events[-1][0],"wait")
        original=r.check_binding
        with fake_child(self.frame(body)) as child:
            def slow(value): original(value); child.tick[0]+=31
            with mock.patch.object(r,"check_binding",side_effect=slow),self.assertRaises(self.c.TransportFailure):
                r.transport(self.core,self.c,self.value,b'[]')
            child.popen.assert_not_called()

    def test_complete_raw_fsync_precedes_endpoint_drift_refusal(self):
        body=r.encode([{"jsonrpc":"2.0","id":1,"result":"public-genesis"}])
        store=self.c.Store(self.directory/"receipt"); self.addCleanup(store.close)
        write=store.write
        def drift(name,raw):
            result=write(name,raw)
            if name=="rpc-01.http.json": self.credential.write_bytes(self.raw+b" ")
            return result
        with mock.patch.object(r,"CREDENTIAL",self.credential),fake_child(self.frame(body)),mock.patch.object(store,"write",side_effect=drift):
            client=r.Rpc(self.core,self.c,self.core.helpers()[3],store,self.value,self.keys,r.MODE)
            with self.assertRaisesRegex(r.Stop,"credential_refused"): client.query([("getGenesisHash",[])])
        self.assertEqual((store.path/"rpc-01.response.bin").read_bytes(),body)
        self.assertEqual(json.loads((store.path/"rpc-01.http.json").read_bytes())["status"],200)
        self.assertFalse((store.path/"rpc-01.failure.json").exists())

    def test_worker_secret_safe_frames_and_owner_only_synthetic_credential(self):
        for body,expected in ((b'[]',"RESPONSE"),(self.raw.strip(),"FAILURE")):
            with self.subTest(expected=expected),ExitStack() as stack:
                response=mock.Mock(); response.getheaders.return_value=[]; response.read.side_effect=[body,b'']; response.length=0; response.status=200
                connection=mock.Mock(); connection.getresponse.return_value=response
                output=io.BytesIO()
                stack.enter_context(mock.patch.object(r,"CREDENTIAL",self.credential))
                stack.enter_context(mock.patch.dict(os.environ,{"PIV1_PHASE10_ENDPOINT_SHA256":r.sha(self.raw)}))
                stack.enter_context(mock.patch.object(r.sys,"stdin",mock.Mock(buffer=io.BytesIO(b'[]'))))
                stack.enter_context(mock.patch.object(r.sys,"stdout",mock.Mock(buffer=output)))
                stack.enter_context(mock.patch.object(r.http.client,"HTTPSConnection",return_value=connection))
                r.worker(); frame=json.loads(output.getvalue()); self.assertEqual(frame["kind"],expected)
                self.assertNotIn(self.raw.strip(),output.getvalue()); connection.close.assert_called_once()
        with mock.patch.object(r,"CREDENTIAL",self.credential):
            os.chmod(self.credential,0o644)
            with self.assertRaisesRegex(r.Stop,"credential_refused"): r.verify_credential(self.value)
            os.chmod(self.credential,0o600)
            link=self.directory/"symlink.url"; link.symlink_to(self.credential)
            with mock.patch.object(r,"CREDENTIAL",link),self.assertRaisesRegex(r.Stop,"credential_refused"): r.verify_credential(self.value)

    def test_native_mode_controller_factory_and_original_recovery_under_fake_os(self):
        fixture=t.Phase10ControllerTests("test_controller_single_send_exact_branches_recovery_and_retained_unknown")
        fixture.setUp(); self.addCleanup(fixture.doCleanups); fixture.private_package()
        e=t.e; fixture.value["rpcTransport"]["endpointCredentialSha256"]=r.sha(self.raw); fixture.make_authorization()
        original_route=e.rpc_module; original_core=e.successor_core; clock=[100.0]; expire_manifest=[False]
        def local_route(value):
            route=original_route(value); route.CREDENTIAL=self.credential; return route
        def native_core():
            core=original_core(); helpers=core.helpers(); core.helpers=lambda:helpers
            core.invoke_signer=lambda _j,payload:fixture.signer(payload)
            core.native_collection=mock.Mock(side_effect=AssertionError("official collection forbidden"))
            original_write=helpers[1].Store.write
            def write(store,name,raw):
                result=original_write(store,name,raw)
                if name=="manifest.json" and expire_manifest[0]: clock[0]+=core.COLLECTION_SECONDS+1
                return result
            helpers[1].Store.write=write
            return core
        pipes={}; requests=[]; processes=[]; fd_counter=[1000000]; registered={}
        class Pipe:
            def __init__(self): fd_counter[0]+=1; self.fd=fd_counter[0]; self.closed=False
            def fileno(self): return self.fd
            def close(self): self.closed=True
        def popen(*_args,**options):
            self.assertEqual(options["env"]["PIV1_PHASE10_ENDPOINT_SHA256"],r.sha(self.raw))
            process=SimpleNamespace(stdin=Pipe(),stdout=Pipe(),pid=424242,returncode=None,raw=bytearray(),sent=False)
            def wait(timeout=None): process.returncode=0; return 0
            process.wait=mock.Mock(side_effect=wait); process.kill=mock.Mock()
            pipes[process.stdin.fd]=process; pipes[process.stdout.fd]=process; processes.append(process); return process
        read,write=os.read,os.write
        def pipe_write(fd,raw):
            if fd not in pipes: return write(fd,raw)
            pipes[fd].raw.extend(raw); return len(raw)
        def pipe_read(fd,size):
            if fd not in pipes: return read(fd,size)
            process=pipes[fd]
            if process.sent: return b''
            process.sent=True; requests.extend(json.loads(process.raw)); status,_headers,body=fixture.transport(bytes(process.raw))
            return self.frame(body)
        selector=mock.Mock(); selector.register.side_effect=lambda stream,event:registered.update({stream:event})
        selector.unregister.side_effect=lambda stream:registered.pop(stream); selector.get_map.side_effect=lambda:registered
        selector.select.side_effect=lambda _timeout:[(SimpleNamespace(fileobj=stream),event) for stream,event in list(registered.items())]
        def run(action,tail=None):
            return e.successor(action,fixture.value,fixture.journal,authorization_raw=fixture.authorization_raw,
                authorization_sha256=fixture.authorization_pin,expected_tail=tail,live=True,live_sign=True,signer_profile_path="/synthetic-unread-profile.json")
        with ExitStack() as stack:
            stack.enter_context(mock.patch.object(e,"rpc_module",side_effect=local_route))
            stack.enter_context(mock.patch.object(e,"successor_core",side_effect=native_core))
            stack.enter_context(mock.patch.object(r.subprocess,"Popen",side_effect=popen))
            stack.enter_context(mock.patch.object(r.selectors,"DefaultSelector",return_value=selector))
            stack.enter_context(mock.patch.object(r.os,"read",side_effect=pipe_read)); stack.enter_context(mock.patch.object(r.os,"write",side_effect=pipe_write))
            stack.enter_context(mock.patch.object(r.os,"set_blocking")); stack.enter_context(mock.patch.object(r.os,"killpg"))
            stack.enter_context(mock.patch.dict(os.environ,{key:"" for key in (*self.c.PROXIES,*self.core.RPC_TLS_ENV)}))
            created=run("CREATE"); sent=run("CAPTURE_SIGN_SEND",created["tail"]); fixture.recovered=True
            expire_manifest[0]=True
            with mock.patch.object(e.time,"monotonic",side_effect=lambda:clock[0]),self.assertRaisesRegex(e.Stop,"collection_deadline_no_adoption"):
                run("RECOVER",sent["tail"])
            held=run("INSPECT",fixture.tail()); self.assertEqual(held["status"],"RECOVER_ATTEMPT")
            expire_manifest[0]=False; recovered=run("RECOVER",held["tail"])
            self.assertEqual(recovered["status"],"FINALIZED"); self.assertEqual(recovered["oldPossibleFeeLamports"],"5000")
        self.assertEqual(fixture.signs,1); self.assertEqual(fixture.sends,1)
        self.assertEqual(len(processes),21); self.assertTrue(all(p.returncode==0 and p.stdin.closed and p.stdout.closed for p in processes))
        self.assertTrue(all(len(q["params"][0])==5 for q in requests if q["method"]=="getMultipleAccounts"))
        records=[json.loads(p.read_bytes()) for p in sorted(Path(fixture.journal).glob("*.json"))]
        self.assertEqual(records[1]["data"]["mode"],r.MODE)
        for p in Path(fixture.journal+".evidence").glob("*-recovery/collection/manifest.json"):
            self.assertEqual(json.loads(p.read_bytes())["transportMode"],r.MODE)


if __name__=="__main__": unittest.main()
