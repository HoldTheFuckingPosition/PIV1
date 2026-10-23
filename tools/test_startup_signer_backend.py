"""Future signer-controller tests: public historical wires and injected backend.

No private file, real signer child, key derivation, signing or actual transport.
Full-host ELF/message/lifetime seams remain synthetic, as in retained tests.
"""
import base64
import copy
import importlib.util
import json
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest import mock

HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location("signer_full_manifest_helpers",HERE/"test_startup_manifest.py")
t=importlib.util.module_from_spec(spec);spec.loader.exec_module(t);o=t.o


class SignerBackendTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls): t.StartupManifestTests.setUpClass()

    def setUp(self):
        self.full=t.StartupManifestTests(methodName="test_actual_full_949_create_reopen_and_record_bound")
        self.full.setUp();self.addCleanup(self.full.doCleanups);self.host=self.full.host
        patch=mock.patch.object(o,"helpers",return_value=(self.host.r,self.host.c,self.host.s,self.host.j));patch.start();self.addCleanup(patch.stop)
        review,life,unsigned=t.h.phase_from_fixture(self.host.c,1);self.pid=self.host.manifest["phases"][0]["id"]
        self.host.prepared={"id":self.pid,"wireBase64":unsigned,"lifetime":life,"feeQuoteLamports":"10000"}
        patch=mock.patch.object(o,"phase_template",return_value=review);patch.start();self.addCleanup(patch.stop)
        self.host.create();self.host.call("PREPARE",[self.host.prepared]);self.invocations=[]
        self.data={"ids":[self.pid],"signerProfilePath":"/approved-disposable-testnet/profile.json"}

    def auth(self):
        a=json.loads(self.host.auth()["authorization_raw"]);a.update(schema="piv1-launcher-release-authorization-v2",signerBackendSha256=o.SIGNER_BACKEND_SHA,signerProfileSha256="e"*64)
        raw=o.encode(a);return {"authorization_raw":raw,"authorization_sha256":o.sha(raw)}

    def records(self):return [json.loads(p.read_bytes()) for p in sorted(o.journal_path(self.host.base).glob("*.json"))]

    def response(self,request):
        return {"schema":"piv1-startup-signing-response-v1","requestId":request["requestId"],"packets":[{"id":self.pid,"wireBase64":self.host.tx["signedWireBase64"]}]}

    def backend(self,payload):
        self.invocations.append(payload);chain=self.records()
        self.assertEqual(chain[-1]["kind"],"SIGN_ATTEMPT")
        self.assertEqual(chain[-1]["data"]["request"],payload["request"])
        self.assertNotIn(self.data["signerProfilePath"],json.dumps(chain))
        self.assertEqual(o.replay(self.host.j,chain)["transportMode"],"OFFLINE_INJECTED_TEST")
        return self.response(payload["request"])

    def sign(self,backend=None,**kw):return self.host.call("SIGN",self.data,live_sign=True,_signer=backend or self.backend,**(kw or self.auth()))

    def test_atomic_request_attempt_precedes_callback_and_reuses_import(self):
        before=self.records();result=self.sign();chain=self.records()
        self.assertEqual([r["kind"] for r in chain[len(before):]],["SIGN_ATTEMPT","IMPORT"])
        self.assertEqual(len(self.invocations),1)
        self.assertIsNone(result["pendingSignerAttempt"]);self.assertIsNone(result["pendingSigningRequest"])
        self.assertFalse(result["signerInvoked"]);self.assertFalse(result["signerControlProven"]);self.assertFalse(result["liveReady"])
        self.assertEqual(result["signerInvocationStatus"],"OFFLINE_INJECTED_ONLY")
        self.assertEqual(result["proposedFeeLedger"]["actualFeeLamports"],"0")
        self.assertEqual(self.host.calls,[])
        with self.assertRaisesRegex(o.Stop,"transport_mode_changed"):
            self.host.call("SEND",{"id":self.pid},live=True,**self.auth())
        with self.assertRaisesRegex(o.Stop,"live_transport_not_enabled"):
            self.host.call("SEND",{"id":self.pid},live_sign=True,**self.auth())
        self.assertEqual(self.host.calls,[])

    def test_sign_merges_callback_revalidation_into_fresh_import_commit(self):
        stages=[];original=o.replay
        def replay(j,records):
            stages.append(records[-1]["kind"])
            return original(j,records)
        def backend(payload):
            self.assertEqual(self.records()[-1]["kind"],"SIGN_ATTEMPT")
            self.assertEqual(stages,["PREPARE","SIGN_ATTEMPT","SIGN_ATTEMPT"])
            return self.response(payload["request"])
        with mock.patch.object(o,"replay",side_effect=replay):result=self.sign(backend)
        self.assertEqual(stages,["PREPARE","SIGN_ATTEMPT","SIGN_ATTEMPT","IMPORT","IMPORT"])
        self.assertIsNone(result["pendingSignerAttempt"])
        self.assertEqual([r["kind"] for r in self.records()][-2:],["SIGN_ATTEMPT","IMPORT"])

    def test_import_commit_retains_exact_post_callback_attempt_comparison(self):
        original=o.Journal.append
        def append(operator,kind,data,**kwargs):
            if kind=="IMPORT":
                # Explicit in-memory mutation seam: the alternative live mode
                # is valid in replay, but differs from the attempted callback.
                changed=copy.deepcopy(operator.chain[-1])
                self.assertEqual(changed["kind"],"SIGN_ATTEMPT")
                changed["data"]["mode"]="LOCAL_TESTNET_BACKEND"
                operator.chain[-1]=changed
            return original(operator,kind,data,**kwargs)
        with mock.patch.object(o.Journal,"append",new=append):
            with self.assertRaisesRegex(o.Stop,"signer_attempt_changed"):self.sign()
        self.host.refresh_tail();result=self.host.call("INSPECT")
        self.assertEqual(self.records()[-1]["kind"],"SIGN_ATTEMPT")
        self.assertEqual(result["pendingSignerAttempt"]["mode"],"OFFLINE_INJECTED_TEST")
        with self.assertRaisesRegex(o.Stop,"signer_attempt_unresolved"):self.sign()

    def test_flag_receipt_profile_and_phase_guards_precede_invocation(self):
        callback=mock.Mock(side_effect=AssertionError("must not invoke"));before=self.records()
        with self.assertRaisesRegex(o.Stop,"live_sign_not_enabled"):
            self.host.call("SIGN",self.data,_signer=callback,**self.auth())
        with self.assertRaisesRegex(o.Stop,"signer_authorization_v2_required"):
            self.host.call("SIGN",self.data,live_sign=True,_signer=callback,**self.host.auth())
        for changes,reason in (({"signerBackendSha256":"0"*64},"signer_authorization_binding"),({"operatorSourceSha256":"0"*64},"launcher_authorization_binding"),({"expiresUnixTimestamp":"0"},"launcher_authorization_expired")):
            auth=json.loads(self.auth()["authorization_raw"]);auth.update(changes);raw=o.encode(auth)
            with self.assertRaisesRegex(o.Stop,reason):
                self.host.call("SIGN",self.data,live_sign=True,_signer=callback,authorization_raw=raw,authorization_sha256=o.sha(raw))
        bad={**self.data,"signerProfilePath":str(HERE/"not-a-private-profile.json")}
        with self.assertRaisesRegex(o.Stop,"signer_profile_in_repository"):
            self.host.call("SIGN",bad,live_sign=True,_signer=callback,**self.auth())
        with self.assertRaisesRegex(o.Stop,"handoff_batch"):
            self.host.call("SIGN",{**self.data,"ids":[self.pid,self.pid]},live_sign=True,_signer=callback,**self.auth())
        self.assertEqual(self.records(),before);callback.assert_not_called()

    def test_ambiguous_attempt_blocks_retry_and_only_exact_response_resolves(self):
        def ambiguous(payload):self.invocations.append(payload);raise TimeoutError("private path must never escape")
        with self.assertRaisesRegex(o.Stop,"^signer_attempt_unresolved$"):self.sign(ambiguous)
        self.host.refresh_tail();result=self.host.call("INSPECT")
        self.assertEqual(result["status"],"SIGNER_ATTEMPT_UNRESOLVED_ORIGINAL_RESPONSE_ONLY")
        self.assertEqual(result["pendingSignerAttempt"]["mode"],"OFFLINE_INJECTED_TEST")
        before=self.records();callback=mock.Mock()
        for action,data in (("SIGN",self.data),("PREPARE",[self.host.prepared]),("HANDOFF",[self.pid]),("BIND_SEAL",{}),("SEND",{"id":self.pid})):
            with self.assertRaisesRegex(o.Stop,"signer_attempt_unresolved"):
                self.host.call(action,data,live_sign=True,_signer=callback,**self.auth())
        callback.assert_not_called();self.assertEqual(self.records(),before)
        response=self.response(result["pendingSigningRequest"])
        with self.assertRaisesRegex(o.Stop,"signing_request_binding"):
            self.host.call("IMPORT",{**response,"requestId":"0"*64},**self.auth())
        changed=json.loads(self.auth()["authorization_raw"]);changed["signerProfileSha256"]="f"*64;raw=o.encode(changed)
        with self.assertRaisesRegex(o.Stop,"signing_authorization_changed"):
            self.host.call("IMPORT",response,authorization_raw=raw,authorization_sha256=o.sha(raw))
        resolved=self.host.call("IMPORT",response,**self.auth());self.assertIsNone(resolved["pendingSignerAttempt"])
        self.assertEqual(len(self.invocations),1)
        with self.assertRaisesRegex(o.Stop,"transport_mode_changed"):
            self.host.call("SEND",{"id":self.pid},live=True,**self.auth())

    def test_invalid_second_signature_and_expired_return_preserve_attempt(self):
        def bad_signature(payload):
            response=self.backend(payload);wire=bytearray(base64.b64decode(response["packets"][0]["wireBase64"]));wire[65]^=1
            response["packets"][0]["wireBase64"]=base64.b64encode(wire).decode();return response
        with self.assertRaisesRegex(o.Stop,"signature_invalid"):self.sign(bad_signature)
        self.host.refresh_tail();result=self.host.call("INSPECT");self.assertIsNotNone(result["pendingSignerAttempt"])
        response=self.response(result["pendingSigningRequest"])
        with mock.patch.object(o.time,"time",return_value=18446744073709551616):
            with self.assertRaisesRegex(o.Stop,"launcher_authorization_expired"):self.host.call("IMPORT",response,**self.auth())
            self.assertIsNotNone(self.host.call("INSPECT")["pendingSignerAttempt"])
        self.host.call("IMPORT",response,**self.auth())

    def test_expiry_after_callback_and_live_intent_uncertainty_are_honest(self):
        clock=[1]
        def expiry(payload):self.invocations.append(payload);clock[0]=18446744073709551616;return self.response(payload["request"])
        with mock.patch.object(o.time,"time",side_effect=lambda:clock[0]):
            with self.assertRaisesRegex(o.Stop,"launcher_authorization_expired"):self.sign(expiry)
        self.host.refresh_tail();self.assertEqual(self.records()[-1]["kind"],"SIGN_ATTEMPT")
        # Pure replay of a real-mode intent states uncertainty, not proof that
        # a child ran. No real child is invoked by this synthetic construction.
        chain=self.records();chain[-1]["data"]["mode"]="LOCAL_TESTNET_BACKEND"
        state=o.replay(self.host.j,chain);self.assertEqual(state["signerAttempt"]["mode"],"LOCAL_TESTNET_BACKEND")
        self.assertIsNone(state["transportMode"])

    def test_replay_pins_request_profile_mode_and_unresolved_transition(self):
        def lost(payload):raise TimeoutError()
        with self.assertRaises(o.Stop):self.sign(lost)
        chain=self.records()
        for key,value in (("signerProfileSha256","0"*64),("signerBackendSha256","0"*64),("mode","INVALID")):
            bad=copy.deepcopy(chain);bad[-1]["data"][key]=value
            with self.assertRaises(ValueError):o.replay(self.host.j,bad)
        bad=copy.deepcopy(chain);bad[-1]["data"]["request"]["packets"][0]["preparedSha256"]="0"*64
        with self.assertRaisesRegex(o.Stop,"signer_attempt_request_binding"):o.replay(self.host.j,bad)
        bad=copy.deepcopy(chain);bad.append({"kind":"PREPARE","data":[self.host.prepared]})
        with self.assertRaisesRegex(o.Stop,"signer_attempt_unresolved"):o.replay(self.host.j,bad)
        self.assertEqual(o.MAX_RECORDS,8192)


class SignerRunnerTests(unittest.TestCase):
    def test_fixed_process_contract_and_bounded_failure_discard(self):
        # Real runner logic, fake process/selector/OS pipe calls only.
        j=SimpleNamespace(pins=lambda:HERE.parent,NODE=Path("/pinned/node"),parse=lambda raw:json.loads(raw))
        for output,exitcode,timeout,success in ((b'{"schema":"public-response"}',0,False,True),(b'partial-private-output',1,False,False),(b'x'*262145,0,False,False),(b'{}',0,True,False)):
            stdin=mock.Mock(closed=False);stdout=mock.Mock(closed=False)
            stdin.fileno.return_value=101;stdout.fileno.return_value=102
            process=mock.Mock(stdin=stdin,stdout=stdout);process.poll.return_value=None if timeout else exitcode;process.wait.return_value=exitcode
            selector=mock.Mock();registered={}
            selector.register.side_effect=lambda stream,event:registered.update({stream:event})
            selector.unregister.side_effect=lambda stream:registered.pop(stream)
            selector.get_map.side_effect=lambda:registered
            selector.select.side_effect=lambda wait:[(SimpleNamespace(fileobj=stream),event) for stream,event in list(registered.items())]
            reads=iter((output,b''));ticks=iter((0,21)) if timeout else None
            with mock.patch.object(o.subprocess,"Popen",return_value=process) as popen,mock.patch.object(o.selectors,"DefaultSelector",return_value=selector),mock.patch.object(o.os,"set_blocking"),mock.patch.object(o.os,"write",side_effect=lambda fd,raw:len(raw)),mock.patch.object(o.os,"read",side_effect=lambda fd,size:next(reads)),mock.patch.object(o.time,"monotonic",side_effect=(lambda:next(ticks)) if timeout else lambda:0):
                if success:self.assertEqual(o.invoke_signer(j,{"signerProfilePath":"/private/not-opened"}),{"schema":"public-response"})
                else:
                    with self.assertRaisesRegex(o.Stop,"^signer_attempt_unresolved$"):o.invoke_signer(j,{"signerProfilePath":"/private/not-opened"})
            argv=popen.call_args.args[0];self.assertEqual(argv,["/pinned/node",str(HERE/"startup_signer_backend.cjs"),"--allow-live-sign"])
            self.assertEqual(popen.call_args.kwargs["env"],{"PIV1_PUBLIC_CHECK_REPO":str(HERE.parent)})
            self.assertEqual(popen.call_args.kwargs["stderr"],o.subprocess.DEVNULL)
            self.assertIs(popen.call_args.kwargs["preexec_fn"],o.disable_signer_core_dumps)
            self.assertNotIn("shell",popen.call_args.kwargs)


if __name__=="__main__":unittest.main()
