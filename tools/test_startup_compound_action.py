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
C="IMPORT_SEND_ATTEMPT_INTENT"


class CompoundActionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):t.LifetimeCaptureTests.setUpClass()

    def environment(self):
        e=t.Environment(self);transport=e.transport
        def compound_transport(raw):
            queries=json.loads(raw)
            if [q["method"] for q in queries]==["sendTransaction"] and e.records()[-1]["kind"]==C:
                self.assertEqual(e.records()[-1]["kind"],C)
                self.assertEqual(queries[0]["params"][0],e.host.tx["signedWireBase64"])
                self.assertIs(queries[0]["params"][1]["skipPreflight"],False)
                self.assertEqual(queries[0]["params"][1]["maxRetries"],0)
                e.queries.append(queries);e.calls.append("sendTransaction");e.sent=True
                return 200,[],o.encode([{"jsonrpc":"2.0","id":queries[0]["id"],"result":e.host.tx["signature"]}])
            return transport(raw)
        e.transport=compound_transport
        return e

    def escrow(self,e):
        paths=list(o.journal_path(e.host.base).with_name(o.journal_path(e.host.base).name+".evidence").glob("*-send/signer-response.json"))
        self.assertEqual(len(paths),1)
        return paths[0]

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

    def test_four_fresh_predispatch_replays_two_checkers_and_one_journal_open(self):
        e=self.environment();events=[];checks=[];quoted=[False];sent=[False];opens=[]
        replay=o.replay;check=o.check_signed;init=o.Journal.__init__
        def observed_replay(j,records):
            if quoted[0] and not sent[0]:events.append(records[-1]["kind"])
            return replay(j,records)
        def observed_check(*args,**kw):
            self.assertEqual(e.records()[-1]["kind"],A)
            self.assertEqual(json.loads(self.escrow(e).read_bytes())["requestId"],e.records()[-1]["data"]["attempt"]["request"]["requestId"])
            checks.append("before-B")
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
                self.assertEqual(events,[A,A,B,C]);self.assertEqual(checks,["before-B"]*2)
                self.assertEqual(e.records()[-1]["kind"],C);sent[0]=True
            result=e.transport(raw)
            if e.capturing and "getLatestBlockhash" in methods:quoted[0]=True
            return result
        with mock.patch.object(o,"replay",side_effect=observed_replay),mock.patch.object(o,"check_signed",side_effect=observed_check),mock.patch.object(o.Journal,"__init__",new=opened):
            result=self.run_compound(e,backend=backend,transport=transport)
        self.assertEqual(len(opens),1);self.assertTrue(e.sent)
        self.assertEqual([r["kind"] for r in e.records()],["CREATE",A,C,"SUBMITTED"])
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

    def test_raw_tamper_at_a_and_transient_b_and_c_is_freshly_refused(self):
        for target,when in ((A,"before"),(A,"after"),(B,"before"),(C,"before")):
            with self.subTest(target=target,when=when):
                e=self.environment();saved=[];callbacks=[];append=o.Journal.append;write=e.host.j.write_file;replay=o.replay
                def corrupt(quote):
                    path=Path(quote["path"])/"rpc-02.response.bin";raw=path.read_bytes();saved.append((path,raw));path.write_bytes(raw+b' ')
                def precommit(instance,kind,data,**kw):
                    if kind==target and when=="before":corrupt(data["quote"] if kind==A else instance.chain[-1]["data"]["quote"])
                    return append(instance,kind,data,**kw)
                def transient(j,records):
                    if target==B and records[-1]["kind"]==B:corrupt(records[-2]["data"]["quote"])
                    return replay(j,records)
                def durable(fd,name,raw):
                    write(fd,name,raw);record=json.loads(raw)
                    if record.get("kind")==target and when=="after":
                        corrupt(record["data"]["quote"] if target==A else e.records()[-2]["data"]["quote"])
                def backend(payload):
                    e.capturing=False;callbacks.append(True);return self.response(e,payload["request"])
                try:
                    with mock.patch.object(o.Journal,"append",new=precommit),mock.patch.object(e.host.j,"write_file",side_effect=durable),mock.patch.object(o,"replay",side_effect=transient):
                        with self.assertRaisesRegex(o.Stop,"quote_artifact_hash"):self.run_compound(e,backend=backend)
                    expected=["CREATE"]+([A] if target!=A or when=="after" else [])
                    self.assertEqual([r["kind"] for r in e.records()],expected)
                    self.assertEqual(len(callbacks),int(target!=A));self.assertFalse(e.sent)
                    self.assertEqual(any(q["method"]=="getMultipleAccounts" for batch in e.queries for q in batch),target==C)
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
                    self.assertEqual(json.loads(self.escrow(e).read_bytes()),self.response(e,e.records()[-1]["data"]["attempt"]["request"]))
                finally:
                    for path,raw in saved:path.write_bytes(raw)
                    e.full.doCleanups()

    def test_returned_response_survives_auth_refusal_and_only_manual_import_resolves(self):
        e=self.environment();authorization=o.authorization;returned=[False];closed=[];close=e.host.c.Store.close
        def backend(payload):
            e.capturing=False;returned[0]=True
            return self.response(e,payload["request"])
        def expired(*args,**kw):
            if returned[0]:raise o.Stop("injected_post_callback_authorization_refusal")
            return authorization(*args,**kw)
        def closing(store):
            if store.path.name.endswith("-send"):closed.append(store.path)
            return close(store)
        with mock.patch.object(o,"authorization",side_effect=expired),mock.patch.object(e.host.c.Store,"close",new=closing):
            with self.assertRaisesRegex(o.Stop,"injected_post_callback_authorization_refusal"):self.run_compound(e,backend=backend)
        self.assertEqual(len(closed),1);self.assertEqual([r["kind"] for r in e.records()],["CREATE",A])
        response=json.loads(self.escrow(e).read_bytes());self.assertFalse(e.sent)
        self.assertNotIn("getMultipleAccounts",e.calls)
        e.host.refresh_tail();report=e.host.call("INSPECT")
        self.assertEqual(report["status"],"SIGNER_ATTEMPT_UNRESOLVED_ORIGINAL_RESPONSE_ONLY")
        with self.assertRaisesRegex(o.Stop,"signer_attempt_unresolved"):self.run_compound(e)
        resolved=e.host.call("IMPORT",response,**self.auth(e))
        self.assertIsNone(resolved["pendingSignerAttempt"]);self.assertTrue(self.escrow(e).exists())
        self.assertEqual(e.records()[-1]["kind"],"IMPORT");self.assertFalse(e.sent)
        original_path=self.escrow(e);original_raw=original_path.read_bytes()
        e.host.call("SEND",{"id":e.pid},_transport=e.transport,**self.auth(e))
        send=next(r["data"] for r in e.records() if r["kind"]=="ATTEMPT")
        self.assertNotEqual(Path(send["evidencePath"]),original_path.parent)
        self.assertEqual(Path(send["evidencePath"]).name,"00000003-send")
        self.assertEqual(original_path.read_bytes(),original_raw);self.assertTrue(e.sent)

    def test_existing_orphan_output_refuses_before_a_or_signer_and_is_retained(self):
        e=self.environment();root=o.journal_path(e.host.base).with_name(o.journal_path(e.host.base).name+".evidence")
        fd=e.host.j.directory(root,True)
        try:
            output=root/"00000002-send";orphan=e.host.c.Store(output)
            try:orphan.write("orphan.bin",b"retained public orphan")
            finally:orphan.close()
        finally:o.os.close(fd)
        backend=mock.Mock(side_effect=AssertionError("must not sign"))
        with self.assertRaises(FileExistsError):self.run_compound(e,backend=backend)
        backend.assert_not_called();self.assertEqual([r["kind"] for r in e.records()],["CREATE"])
        self.assertEqual((output/"orphan.bin").read_bytes(),b"retained public orphan")
        self.assertFalse(e.sent)

    def test_uncertain_response_write_preserves_a_and_exact_reply_without_collection(self):
        e=self.environment();write=e.host.c.Store.write;close=e.host.c.Store.close;closed=[]
        def uncertain(store,name,raw):
            result=write(store,name,raw)
            if name=="signer-response.json":raise OSError("injected R post-fsync uncertainty")
            return result
        def closing(store):
            if store.path.name.endswith("-send"):closed.append(store.path)
            return close(store)
        with mock.patch.object(e.host.c.Store,"write",new=uncertain),mock.patch.object(e.host.c.Store,"close",new=closing):
            with self.assertRaisesRegex(o.Stop,"^compound_response_uncertain_no_retry_or_adoption$"):self.run_compound(e)
        self.assertEqual(len(closed),1);self.assertEqual([r["kind"] for r in e.records()],["CREATE",A])
        self.assertEqual(json.loads(self.escrow(e).read_bytes()),self.response(e,e.records()[-1]["data"]["attempt"]["request"]))
        self.assertNotIn("getMultipleAccounts",e.calls);self.assertFalse(e.sent)

    def test_c_readback_rejects_changed_response_or_committed_bytes_before_post_queries(self):
        for fault in ("response","record"):
            with self.subTest(fault=fault):
                e=self.environment();write=e.host.j.write_file;saved=[];at_commit=[]
                def durable(fd,name,raw):
                    write(fd,name,raw);record=json.loads(raw)
                    if record.get("kind")==C:
                        at_commit.append(len(e.calls))
                        path=self.escrow(e) if fault=="response" else o.journal_path(e.host.base)/name
                        previous=path.read_bytes();saved.append((path,previous));path.write_bytes(previous+b" ")
                try:
                    with mock.patch.object(e.host.j,"write_file",side_effect=durable):
                        with self.assertRaisesRegex(o.Stop,"compound_response_receipt" if fault=="response" else "compound_dispatch_record_changed"):
                            self.run_compound(e)
                    self.assertEqual(e.records()[-1]["kind"],C);self.assertFalse(e.sent)
                    self.assertEqual(at_commit,[len(e.calls)])
                finally:
                    for path,raw in saved:path.write_bytes(raw)
                e.host.refresh_tail();state=o.replay(e.host.j,e.records())
                self.assertIn(e.pid,state["intents"]);self.assertIsNone(state["signerAttempt"])
                with self.assertRaisesRegex(o.Stop,"compound_phase_already_prepared"):self.run_compound(e)
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
        for target in (A,C):
            with self.subTest(target=target):
                e=self.environment();write=e.host.j.write_file;callbacks=[]
                def uncertain(fd,name,raw):
                    write(fd,name,raw)
                    if json.loads(raw).get("kind")==target:raise OSError("injected post-fsync uncertainty")
                def backend(payload):
                    e.capturing=False;callbacks.append(True);return self.response(e,payload["request"])
                with mock.patch.object(e.host.j,"write_file",side_effect=uncertain):
                    with self.assertRaisesRegex(o.Stop,"^commit_uncertain_no_retry_or_adoption$"):self.run_compound(e,backend=backend)
                self.assertEqual([r["kind"] for r in e.records()],["CREATE",A]+([C] if target==C else []))
                self.assertEqual(len(callbacks),int(target==C));self.assertFalse(e.sent)
                self.assertEqual(any(q["method"]=="getMultipleAccounts" for batch in e.queries for q in batch),target==C)
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
        # C applies the old import/attempt predicates and the unchanged INTENT
        # branch, while its path must be rejected before any response read.
        current=self.environment();self.run_compound(current)
        committed=current.records();prefix=committed[:3];self.assertEqual(prefix[-1]["kind"],C)
        state=o.replay(current.host.j,prefix);slots=o.evidence_plan(state["manifest"],current.ref)
        expected=sum(slots.values())-slots[("QUOTE",current.pid)]-slots[("SEND",current.pid)]
        self.assertEqual(o.remaining_reservations(state["manifest"],prefix,slots)[0],expected)
        self.assertEqual(o.remaining_reservations(state["manifest"],committed,slots)[0],expected)
        bad=copy.deepcopy(prefix);bad[-1]["data"]["attempt"]["evidencePath"]="/must-not-be-read"
        with mock.patch.object(o,"compound_response",side_effect=AssertionError("redirected read")) as read:
            with self.assertRaisesRegex(o.Stop,"compound_send_path_binding"):o.replay(current.host.j,bad)
            read.assert_not_called()
        for fault,reason in (("receipt","object_fields"),("attempt","signer_attempt_changed"),("wire","compound_response_binding"),("intent","intent_signed_binding")):
            bad=copy.deepcopy(prefix);d=bad[-1]["data"]
            if fault=="receipt":d["response"].pop("signerAttemptSha256")
            elif fault=="attempt":d["response"]["signerAttemptSha256"]="0"*64
            elif fault=="wire":d["import"]["entries"][0]["wireBase64"]="AA=="
            else:d["intent"]["wireSha256"]="0"*64
            with self.subTest(fault=fault),self.assertRaisesRegex(o.Stop,reason):o.replay(current.host.j,bad)

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
            if kind==C:e.send_height=961
            return record
        with mock.patch.object(o.Journal,"append",new=expired):
            with self.assertRaisesRegex(o.Stop,"post_intent_lifetime_margin"):self.run_compound(e,**auth)
        self.assertEqual(e.records()[-1]["kind"],C);self.assertFalse(e.sent);e.host.refresh_tail()
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
