"""Compound controller boundaries with real small journals and retained signatures.

The imported full-manifest fixture has synthetic ELF/account/Clock and first
message-template seams. Public transport and signer return host data and an
existing historical signature, never new signatures/network/private inputs.
Replay, quote/raw validation, append/fsync and request binding run normally.
The recovery test explicitly stubs an unresolved collector result only.
"""
import copy
import importlib.util
import json
from pathlib import Path
import unittest
from unittest import mock

HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location("compound_lifetime_helpers",HERE/"test_startup_lifetime_capture.py")
t=importlib.util.module_from_spec(spec);spec.loader.exec_module(t);o=t.o
A="PREPARE_SIGN_ATTEMPT"
B="IMPORT_SEND_ATTEMPT"


class CompoundActionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):t.LifetimeCaptureTests.setUpClass()

    def environment(self):return t.Environment(self)

    def auth(self,e):
        a=json.loads(e.host.auth()["authorization_raw"])
        a.update(schema="piv1-launcher-release-authorization-v2",signerBackendSha256=o.SIGNER_BACKEND_SHA,signerProfileSha256="e"*64)
        raw=o.encode(a);return {"authorization_raw":raw,"authorization_sha256":o.sha(raw)}

    def data(self,e):
        return {"packet":{"id":e.pid,"wireBase64":e.host.prepared["wireBase64"]},"signerProfilePath":"/not-opened/compound-profile.json"}

    def response(self,e,request):
        return {"schema":"piv1-startup-signing-response-v1","requestId":request["requestId"],"packets":[{"id":e.pid,"wireBase64":e.host.tx["signedWireBase64"]}]}

    def run_compound(self,e,*,backend=None,transport=None,**kw):
        def historical(payload):
            e.capturing=False
            self.assertEqual(e.records()[-1]["kind"],A)
            return self.response(e,payload["request"])
        e.capturing=True
        try:
            return e.host.call("CAPTURE_SIGN_SEND",self.data(e),live=True,live_sign=True,
                _signer=backend or historical,_transport=transport or e.transport,**(kw or self.auth(e)))
        finally:e.capturing=False

    def lose_response(self,e):
        def lost(payload):
            e.capturing=False
            raise TimeoutError("injected lost response")
        with self.assertRaisesRegex(o.Stop,"^signer_attempt_unresolved$"):self.run_compound(e,backend=lost)
        e.host.refresh_tail()

    def test_five_fresh_predispatch_replays_two_checkers_and_one_journal_open(self):
        e=self.environment();events=[];checks=[];quoted=[False];sent=[False];opens=[]
        replay=o.replay;check=o.check_signed;init=o.Journal.__init__
        def observed_replay(j,records):
            if quoted[0] and not sent[0]:events.append(records[-1]["kind"])
            return replay(j,records)
        def observed_check(*args,**kw):
            self.assertEqual(e.records()[-1]["kind"],A);checks.append("before-B")
            return check(*args,**kw)
        def opened(instance,*args,**kw):opens.append(True);init(instance,*args,**kw)
        def backend(payload):
            e.capturing=False;record=e.records()[-1];d=record["data"]
            self.assertEqual(record["kind"],A)
            self.assertEqual(events,[A,A])
            self.assertEqual(payload["request"],d["attempt"]["request"])
            self.assertEqual(payload["request"]["operatorTail"],record["previousDigest"])
            commitment=o.compound_prepared_digest(record["previousDigest"],d["entries"],d["quote"])
            self.assertEqual(payload["request"]["packets"][0]["preparedRecordDigest"],commitment)
            self.assertNotEqual(commitment,record["digest"])
            self.assertNotIn(self.data(e)["signerProfilePath"],json.dumps(e.records()))
            return self.response(e,payload["request"])
        def transport(raw):
            methods=[q["method"] for q in json.loads(raw)]
            if methods==["sendTransaction"]:
                self.assertEqual(events,[A,A,B,B,"INTENT"]);self.assertEqual(checks,["before-B"]*2)
                self.assertEqual(e.records()[-1]["kind"],"INTENT");sent[0]=True
            result=e.transport(raw)
            if e.capturing and "getLatestBlockhash" in methods:quoted[0]=True
            return result
        with mock.patch.object(o,"replay",side_effect=observed_replay),mock.patch.object(o,"check_signed",side_effect=observed_check),mock.patch.object(o.Journal,"__init__",new=opened):
            result=self.run_compound(e,backend=backend,transport=transport)
        self.assertEqual(len(opens),1);self.assertTrue(e.sent)
        self.assertEqual([r["kind"] for r in e.records()],["CREATE",A,B,"INTENT","SUBMITTED"])
        self.assertEqual(result["status"],"UNKNOWN_OR_SUBMITTED_RECOVERY_ONLY")
        self.assertFalse(result["liveReady"]);self.assertFalse(result["signerInvoked"])

    def test_flags_matching_modes_and_fresh_current_phase_precede_external_work(self):
        request={"schema":o.SCHEMA,"action":"CAPTURE_SIGN_SEND","expectedTail":None,"data":None}
        callback=lambda value:None
        with mock.patch.object(o,"helpers",side_effect=AssertionError("must not load")):
            for kwargs,reason in (({},"live_sign_not_enabled"),({"live_sign":True},"live_transport_not_enabled"),
                ({"live_sign":True,"live":True,"_signer":callback},"compound_transport_signer_mode"),
                ({"live_sign":True,"live":True,"_transport":callback},"compound_transport_signer_mode")):
                with self.subTest(kwargs=kwargs),self.assertRaisesRegex(o.Stop,reason):o.operate(request,"/unused",**kwargs)
        e=self.environment();bad=self.data(e);bad["packet"]["id"]=e.host.manifest["phases"][1]["id"]
        with self.assertRaisesRegex(o.Stop,"compound_current_phase_required"):
            e.host.call("CAPTURE_SIGN_SEND",bad,live=True,live_sign=True,_signer=callback,_transport=e.transport,**self.auth(e))
        self.assertEqual(e.calls,[])
        e.host.call("PREPARE",[e.host.prepared])
        with self.assertRaisesRegex(o.Stop,"compound_phase_already_prepared"):self.run_compound(e)
        e.host.call("HANDOFF",[e.pid],**self.auth(e))
        with self.assertRaisesRegex(o.Stop,"compound_current_phase_required"):self.run_compound(e)
        self.assertEqual(e.calls,[])

    def test_raw_tamper_before_each_commit_and_after_each_durability_is_freshly_refused(self):
        for target,when in ((A,"before"),(A,"after"),(B,"before"),(B,"after")):
            with self.subTest(target=target,when=when):
                e=self.environment();saved=[];callbacks=[];append=o.Journal.append;write=e.host.j.write_file
                def corrupt(quote):
                    path=Path(quote["path"])/"rpc-02.response.bin";raw=path.read_bytes();saved.append((path,raw));path.write_bytes(raw+b' ')
                def precommit(instance,kind,data,**kw):
                    if kind==target and when=="before":corrupt(data["quote"] if kind==A else instance.chain[-1]["data"]["quote"])
                    return append(instance,kind,data,**kw)
                def durable(fd,name,raw):
                    write(fd,name,raw);record=json.loads(raw)
                    if record.get("kind")==target and when=="after":
                        corrupt(record["data"]["quote"] if target==A else e.records()[-2]["data"]["quote"])
                def backend(payload):
                    e.capturing=False;callbacks.append(True);return self.response(e,payload["request"])
                try:
                    with mock.patch.object(o.Journal,"append",new=precommit),mock.patch.object(e.host.j,"write_file",side_effect=durable):
                        with self.assertRaisesRegex(o.Stop,"quote_artifact_hash"):self.run_compound(e,backend=backend)
                    expected=["CREATE"]+([A] if target==B or when=="after" else [])+([B] if target==B and when=="after" else [])
                    self.assertEqual([r["kind"] for r in e.records()],expected)
                    self.assertEqual(len(callbacks),int(target==B));self.assertFalse(e.sent)
                    self.assertFalse(any(q["method"]=="getMultipleAccounts" for batch in e.queries for q in batch))
                finally:
                    for path,raw in saved:path.write_bytes(raw)
                    e.full.doCleanups()

    def test_callback_source_change_and_precommit_content_rebinding_refuse(self):
        for fault in ("source","content"):
            with self.subTest(fault=fault):
                e=self.environment();append=o.Journal.append;read=Path.read_bytes;changed=[False]
                def source(path):
                    return b'injected changed operator source' if changed[0] and path==Path(o.__file__) else read(path)
                def backend(payload):
                    e.capturing=False;changed[0]=True;return self.response(e,payload["request"])
                def precommit(instance,kind,data,**kw):
                    if kind==A:data["attempt"]["request"]["packets"][0]["preparedRecordDigest"]="0"*64
                    return append(instance,kind,data,**kw)
                if fault=="source":
                    with mock.patch.object(Path,"read_bytes",new=source):
                        with self.assertRaisesRegex(o.Stop,"launcher_authorization_binding"):self.run_compound(e,backend=backend)
                else:
                    with mock.patch.object(o.Journal,"append",new=precommit):
                        with self.assertRaisesRegex(o.Stop,"signer_attempt_request_binding"):self.run_compound(e)
                self.assertEqual([r["kind"] for r in e.records()],["CREATE"]+([A] if fault=="source" else []))
                self.assertFalse(e.sent)
                e.full.doCleanups()

    def test_second_checker_failure_and_callback_raw_tamper_keep_original_attempt(self):
        for fault in ("checker","raw"):
            with self.subTest(fault=fault):
                e=self.environment();checks=[];saved=[];check=o.check_signed
                def checked(*args,**kw):
                    checks.append(True)
                    if fault=="checker" and len(checks)==2:raise o.Stop("injected_second_checker_refusal")
                    return check(*args,**kw)
                def backend(payload):
                    e.capturing=False
                    if fault=="raw":
                        path=Path(e.records()[-1]["data"]["quote"]["path"])/"rpc-02.response.bin"
                        raw=path.read_bytes();saved.append((path,raw));path.write_bytes(raw+b' ')
                    return self.response(e,payload["request"])
                try:
                    with mock.patch.object(o,"check_signed",side_effect=checked):
                        with self.assertRaisesRegex(o.Stop,"injected_second_checker_refusal" if fault=="checker" else "quote_artifact_hash"):
                            self.run_compound(e,backend=backend)
                    self.assertEqual([r["kind"] for r in e.records()],["CREATE",A]);self.assertFalse(e.sent)
                    self.assertEqual(len(checks),2)
                finally:
                    for path,raw in saved:path.write_bytes(raw)
                    e.full.doCleanups()

    def test_lost_reply_blocks_reentry_and_only_exact_manual_import_resolves(self):
        e=self.environment();self.lose_response(e);report=e.host.call("INSPECT");before=e.records()
        self.assertEqual(report["status"],"SIGNER_ATTEMPT_UNRESOLVED_ORIGINAL_RESPONSE_ONLY")
        for action,data in (("CAPTURE_SIGN_SEND",self.data(e)),("SIGN",{"ids":[e.pid],"signerProfilePath":"/not-opened/profile"}),("SEND",{"id":e.pid}),("PREPARE",[e.host.prepared])):
            with self.assertRaisesRegex(o.Stop,"signer_attempt_unresolved"):
                e.host.call(action,data,live=True,live_sign=True,_signer=lambda x:None,_transport=e.transport,**self.auth(e))
        response=self.response(e,report["pendingSigningRequest"])
        with self.assertRaisesRegex(o.Stop,"signing_request_binding"):e.host.call("IMPORT",{**response,"requestId":"0"*64},**self.auth(e))
        changed=json.loads(self.auth(e)["authorization_raw"]);changed["signerProfileSha256"]="f"*64;raw=o.encode(changed)
        with self.assertRaisesRegex(o.Stop,"signing_authorization_changed"):
            e.host.call("IMPORT",response,authorization_raw=raw,authorization_sha256=o.sha(raw))
        self.assertEqual(e.records(),before)
        resolved=e.host.call("IMPORT",response,**self.auth(e));self.assertIsNone(resolved["pendingSignerAttempt"])
        self.assertEqual(e.records()[-1]["kind"],"IMPORT");self.assertFalse(e.sent)
        with self.assertRaisesRegex(o.Stop,"compound_phase_already_prepared"):self.run_compound(e)

    def test_uncertain_compound_write_stops_before_next_external_boundary(self):
        for target in (A,B):
            with self.subTest(target=target):
                e=self.environment();write=e.host.j.write_file;callbacks=[]
                def uncertain(fd,name,raw):
                    write(fd,name,raw)
                    if json.loads(raw).get("kind")==target:raise OSError("injected post-fsync uncertainty")
                def backend(payload):
                    e.capturing=False;callbacks.append(True);return self.response(e,payload["request"])
                with mock.patch.object(e.host.j,"write_file",side_effect=uncertain):
                    with self.assertRaisesRegex(o.Stop,"^commit_uncertain_no_retry_or_adoption$"):self.run_compound(e,backend=backend)
                self.assertEqual([r["kind"] for r in e.records()],["CREATE",A]+([B] if target==B else []))
                self.assertEqual(len(callbacks),int(target==B));self.assertFalse(e.sent)
                self.assertFalse(any(q["method"]=="getMultipleAccounts" for batch in e.queries for q in batch))
                e.host.refresh_tail();report=e.host.call("INSPECT")
                self.assertEqual(report["pendingSignerAttempt"] is not None,target==A)
                with self.assertRaisesRegex(o.Stop,"signer_attempt_unresolved" if target==A else "compound_phase_already_prepared"):self.run_compound(e)
                e.full.doCleanups()

    def test_interrupted_callback_releases_lock_and_keeps_durable_attempt(self):
        e=self.environment()
        class Interrupted(BaseException):pass
        def interrupted(payload):
            e.capturing=False
            raise Interrupted("injected callback interruption")
        with self.assertRaises(Interrupted):self.run_compound(e,backend=interrupted)
        self.assertEqual([r["kind"] for r in e.records()],["CREATE",A])
        e.host.refresh_tail();report=e.host.call("INSPECT")
        self.assertEqual(report["status"],"SIGNER_ATTEMPT_UNRESOLVED_ORIGINAL_RESPONSE_ONLY")
        self.assertFalse(e.sent)

    def test_replay_compound_guards_and_quote_reservation_are_independent(self):
        e=self.environment();self.lose_response(e);records=e.records();state=o.replay(e.host.j,records)
        slots=o.evidence_plan(state["manifest"],e.ref)
        self.assertEqual(o.remaining_reservations(state["manifest"],records,slots),
            (sum(slots.values())-slots[("QUOTE",e.pid)],8*len(state["manifest"]["phases"])+5))
        response=self.response(e,state["handoff"]);imported=o.import_response(e.host.c,e.host.j,state,response,self.auth(e)["authorization_sha256"])
        # Actual append invokes fresh full replay; no forged virtual records.
        operator=o.Journal(e.host.j,e.host.base)
        try:
            attempt={"id":e.pid,"stage":"SEND","evidencePath":str(operator.path.with_name(operator.path.name+".evidence")/f"{len(operator.chain):08}-send"),"transportMode":"OFFLINE_INJECTED_TEST"}
            for change,reason in (({"stage":"RECOVER"},"compound_send_attempt_required"),({"evidencePath":"/unrelated/public-output"},"compound_send_path_binding"),({"id":"other_phase"},"compound_current_import_attempt")):
                with self.assertRaisesRegex(o.Stop,reason):operator.append(B,{"import":imported,"attempt":{**attempt,**change}})
            with self.assertRaisesRegex(o.Stop,"signer_attempt_unresolved"):operator.append(A,records[-1]["data"])
        finally:operator.close()
        self.assertEqual(e.records(),records)
        bad=copy.deepcopy(records);bad[-1]["data"]["entries"][0]["id"]=e.host.manifest["phases"][1]["id"]
        with self.assertRaisesRegex(o.Stop,"compound_current_phase_required"):o.replay(e.host.j,bad)
        bad=copy.deepcopy(records);bad[-1]["data"]["quote"]["operatorSourceSha256"]="0"*64
        with self.assertRaisesRegex(o.Stop,"quote_source_or_predecessor_binding"):o.replay(e.host.j,bad)

    def test_future_prepared_prefix_cannot_redirect_compound_request_or_import(self):
        e=self.environment();future=e.host.manifest["phases"][1]["id"]
        # Legacy replay permits a future PREPARE without the operate builder's
        # contiguous-batch precondition. Persist it through real Journal.append;
        # the retained first-message template remains the explicit fixture seam.
        operator=o.Journal(e.host.j,e.host.base)
        try:operator.append("PREPARE",[{**e.host.prepared,"id":future,"feeQuoteLamports":"5000","reviewed":e.review}])
        finally:operator.close()
        e.host.refresh_tail()
        self.lose_response(e);records=e.records();record=records[-1]
        prior=o.replay(e.host.j,records[:-1]);self.assertIn(future,prior["prepared"])
        d=record["data"];pid=e.pid
        prepared_state={**prior,"prepared":{**prior["prepared"],pid:d["entries"][0]},
            "preparedAt":{**prior["preparedAt"],pid:o.compound_prepared_digest(record["previousDigest"],d["entries"],d["quote"])},
            "quoteByPhase":{**prior["quoteByPhase"],pid:d["quote"]}}
        # This is a canonical valid request for a genuinely prepared future
        # phase. V1 checked only its count; it omitted the current-entry ID tie.
        future_request=o.signing_request(e.host.c,prepared_state,record["previousDigest"],[future],self.auth(e)["authorization_sha256"])
        self.assertEqual(future_request["packets"][0]["preparedRecordDigest"],prior["preparedAt"][future])
        bad=copy.deepcopy(records);bad[-1]["data"]["attempt"]["request"]=future_request
        with self.assertRaisesRegex(o.Stop,"compound_current_signing_request"):o.replay(e.host.j,bad)
        state=o.replay(e.host.j,records)
        imported=o.import_response(e.host.c,e.host.j,state,self.response(e,state["handoff"]),self.auth(e)["authorization_sha256"])
        operator=o.Journal(e.host.j,e.host.base)
        try:
            attempt={"id":pid,"stage":"SEND","evidencePath":str(operator.path.with_name(operator.path.name+".evidence")/f"{len(operator.chain):08}-send"),"transportMode":"OFFLINE_INJECTED_TEST"}
            for change_import in (True,False):
                changed=copy.deepcopy(imported);send=attempt.copy()
                if change_import:changed["entries"][0]["id"]=future
                else:send["id"]=future
                with self.assertRaisesRegex(o.Stop,"compound_current_import_attempt"):
                    operator.append(B,{"import":changed,"attempt":send})
        finally:operator.close()
        self.assertEqual(e.records(),records)

    def test_post_intent_expiry_holds_and_recovery_uses_only_original_signature(self):
        e=self.environment();append=o.Journal.append;auth=self.auth(e)
        def expired(instance,kind,data,**kw):
            record=append(instance,kind,data,**kw)
            if kind=="INTENT":e.send_height=961
            return record
        with mock.patch.object(o.Journal,"append",new=expired):
            with self.assertRaisesRegex(o.Stop,"post_intent_lifetime_margin"):self.run_compound(e,**auth)
        self.assertEqual(e.records()[-1]["kind"],"INTENT");self.assertFalse(e.sent);e.host.refresh_tail()
        changed=json.loads(auth["authorization_raw"]);changed["expiresUnixTimestamp"]="18446744073709551614";raw=o.encode(changed)
        with self.assertRaisesRegex(o.Stop,"recovery_original_authorization_required"):
            e.host.call("RECOVER",{"id":e.pid},_transport=e.transport,authorization_raw=raw,authorization_sha256=o.sha(raw))
        captured=[]
        def unresolved(request,output,**kw):
            captured.append(request)
            return {"observation":None,"manifestReceipt":{"sha256":"a"*64}}
        with mock.patch.object(e.host.c,"collect",side_effect=unresolved):
            e.host.call("RECOVER",{"id":e.pid},_transport=e.transport,**auth)
        self.assertEqual(len(captured),1);self.assertEqual(captured[0]["signature"],e.host.tx["signature"])
        self.assertEqual(captured[0]["wireBase64"],e.host.tx["signedWireBase64"])
        self.assertIs(captured[0]["includeExpiry"],False);self.assertEqual(e.records()[-1]["kind"],"UNKNOWN")
        self.assertFalse(e.sent)


if __name__=="__main__":unittest.main()
