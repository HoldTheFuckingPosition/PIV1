"""Explicit durable-only outputs; historical signatures and synthetic public I/O.

The imported full-host ELF/account/message seams remain synthetic. No private
file, native signer, new signature, actual network or real transaction is used.
"""
import importlib.util
import json
from pathlib import Path
import unittest
from unittest import mock

HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location("receipt_lifetime_helpers",HERE/"test_startup_lifetime_capture.py")
t=importlib.util.module_from_spec(spec);spec.loader.exec_module(t);o=t.o


class CommitReceiptTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):t.LifetimeCaptureTests.setUpClass()

    def environment(self):return t.Environment(self)

    def auth(self,e):
        a=json.loads(e.host.auth()["authorization_raw"])
        a.update(schema="piv1-launcher-release-authorization-v2",signerBackendSha256=o.SIGNER_BACKEND_SHA,signerProfileSha256="e"*64)
        raw=o.encode(a);return {"authorization_raw":raw,"authorization_sha256":o.sha(raw)}

    def capture(self,e,auth,mode="REPORT",transport=None):
        e.capturing=True
        try:
            return e.host.call("PREPARE",{"captureLifetime":True,"packets":[{"id":e.pid,"wireBase64":e.host.prepared["wireBase64"]}]},_transport=transport or e.transport,output_mode=mode,**auth)
        finally:e.capturing=False

    def sign(self,e,auth,mode="REPORT",backend=None):
        def historical_response(payload):
            self.assertEqual(e.records()[-1]["kind"],"SIGN_ATTEMPT")
            return {"schema":"piv1-startup-signing-response-v1","requestId":payload["request"]["requestId"],"packets":[{"id":e.pid,"wireBase64":e.host.tx["signedWireBase64"]}]}
        return e.host.call("SIGN",{"ids":[e.pid],"signerProfilePath":"/not-opened/profile.json"},live_sign=True,_signer=backend or historical_response,output_mode=mode,**auth)

    def assert_receipt(self,e,result,action,kind):
        record=e.records()[-1]
        self.assertEqual(result,{"schema":"piv1-startup-release-commit-receipt-v1","status":"DURABLE_COMMIT_ONLY","action":action,"recordSequence":record["sequence"],"recordKind":kind,"recordDigest":record["digest"],"tail":record["digest"],"durableCommit":True,"postCommitReplayPerformed":False})
        path=o.journal_path(e.host.base)/f"{int(result['recordSequence']):08}.json"
        self.assertEqual(path.read_bytes(),o.encode(record))

    def test_complete_small_host_schedule_uses_eight_instead_of_ten_predispatch_replays(self):
        for mode,expected in (("REPORT",10),("COMMIT_RECEIPT",8)):
            with self.subTest(mode=mode):
                e=self.environment()
                try:
                    auth=self.auth(e);events=[];quoted=[False];dispatched=[False];stage=["PREPARE"];original=o.replay
                    def replay(j,records):
                        if quoted[0] and not dispatched[0]:events.append((stage[0],records[-1]["kind"]))
                        return original(j,records)
                    def transport(raw):
                        methods=[q["method"] for q in json.loads(raw)]
                        if "sendTransaction" in methods:
                            self.assertEqual(len(events),expected);dispatched[0]=True
                        response=e.transport(raw)
                        if e.capturing and "getLatestBlockhash" in methods:quoted[0]=True
                        return response
                    with mock.patch.object(o,"replay",side_effect=replay):
                        prepared=self.capture(e,auth,mode,transport);stage[0]="SIGN"
                        signed=self.sign(e,auth,mode);stage[0]="SEND"
                        sent=e.host.call("SEND",{"id":e.pid},_transport=transport,**auth)
                    self.assertTrue(dispatched[0]);self.assertEqual(e.calls.count("sendTransaction"),1)
                    self.assertEqual([sum(stage==name for stage,_ in events) for name in ("PREPARE","SIGN","SEND")],[2,5,3] if mode=="REPORT" else [1,4,3])
                    self.assertEqual(sent["status"],"UNKNOWN_OR_SUBMITTED_RECOVERY_ONLY")
                    if mode=="REPORT":
                        self.assertFalse(prepared["liveReady"]);self.assertIsNone(signed["pendingSignerAttempt"])
                    else:
                        # Compare against their own persisted identities, after
                        # subsequent actions have independently reopened them.
                        records=e.records()
                        for result,action,kind in ((prepared,"PREPARE","PREPARE"),(signed,"SIGN","IMPORT")):
                            record=records[int(result["recordSequence"])]
                            self.assertEqual(result["recordDigest"],record["digest"])
                            self.assertEqual(result["recordKind"],kind);self.assertEqual(result["action"],action)
                            self.assertNotIn("liveReady",result);self.assertNotIn("completedPhases",result)
                    with self.assertRaisesRegex(o.Stop,"intent_exists_recovery_only"):
                        e.host.call("SEND",{"id":e.pid},_transport=transport,**auth)
                    self.assertEqual(e.calls.count("sendTransaction"),1)
                finally:e.full.doCleanups()

    def test_invalid_mode_and_unsupported_actions_stop_before_helpers(self):
        with mock.patch.object(o,"helpers",side_effect=AssertionError("must not open anything")):
            for action in (None,"CREATE","INSPECT","HANDOFF","IMPORT","BIND_SEAL","SEND","RECOVER"):
                with self.subTest(action=action),self.assertRaisesRegex(o.Stop,"commit_receipt_action"):
                    o.operate({"schema":o.SCHEMA,"action":action,"expectedTail":None,"data":None},"/unused",output_mode="COMMIT_RECEIPT")
            for mode in ("UNKNOWN",None,[],{}):
                with self.subTest(mode=mode),self.assertRaisesRegex(o.Stop,"output_mode"):
                    o.operate({"schema":o.SCHEMA,"action":"PREPARE","expectedTail":None,"data":None},"/unused",output_mode=mode)
            self.assertEqual(o.operate({"schema":o.SCHEMA,"action":None,"expectedTail":None,"data":None},"/unused"),{"status":"INCOMPLETE","authorized":False,"liveReady":False})

    def test_postcommit_raw_changes_are_reported_only_by_fresh_validation(self):
        for mode in ("REPORT","COMMIT_RECEIPT"):
            with self.subTest(mode=mode):
                e=self.environment();changed=[]
                try:
                    auth=self.auth(e);original=e.host.j.write_file
                    def write(fd,name,raw):
                        original(fd,name,raw);record=json.loads(raw)
                        if record.get("schema")==o.SCHEMA and record.get("kind")=="PREPARE":
                            path=Path(record["data"]["quote"]["path"])/"rpc-02.response.bin"
                            saved=path.read_bytes();changed.append((path,saved));path.write_bytes(saved+b' ')
                    with mock.patch.object(e.host.j,"write_file",side_effect=write):
                        if mode=="REPORT":
                            with self.assertRaisesRegex(o.Stop,"quote_artifact_hash"):self.capture(e,auth,mode)
                        else:self.assert_receipt(e,self.capture(e,auth,mode),"PREPARE","PREPARE")
                    self.assertEqual(e.records()[-1]["kind"],"PREPARE");e.host.refresh_tail();before=e.records();callback=mock.Mock()
                    with self.assertRaisesRegex(o.Stop,"quote_artifact_hash"):self.sign(e,auth,"COMMIT_RECEIPT",callback)
                    callback.assert_not_called();self.assertEqual(e.records(),before)
                finally:
                    for path,saved in changed:path.write_bytes(saved)
                    e.full.doCleanups()

    def test_uncertain_commit_or_changed_persisted_record_never_returns_receipt(self):
        for fault in ("write_uncertain","record_changed"):
            with self.subTest(fault=fault):
                e=self.environment()
                try:
                    original=e.host.j.write_file
                    def write(fd,name,raw):
                        original(fd,name,raw);record=json.loads(raw)
                        if record.get("schema")==o.SCHEMA and record.get("kind")=="PREPARE":
                            if fault=="write_uncertain":raise OSError("injected post-fsync uncertainty")
                            path=o.journal_path(e.host.base)/name;path.write_bytes(raw+b' ')
                    reason="commit_uncertain_no_retry_or_adoption" if fault=="write_uncertain" else "commit_receipt_uncertain_no_retry_or_adoption"
                    with mock.patch.object(e.host.j,"write_file",side_effect=write):
                        with self.assertRaisesRegex(o.Stop,"^"+reason+"$"):self.capture(e,self.auth(e),"COMMIT_RECEIPT")
                    self.assertEqual([r["kind"] for r in e.records()],["CREATE","PREPARE"])
                finally:e.full.doCleanups()

    def test_receipt_sign_cannot_bypass_callback_raw_tamper_or_original_attempt_hold(self):
        e=self.environment();auth=self.auth(e);self.capture(e,auth,"COMMIT_RECEIPT")
        quote=e.records()[-1]["data"]["quote"];path=Path(quote["path"])/"rpc-02.response.bin";saved=path.read_bytes();calls=[]
        def backend(payload):
            calls.append(payload["request"]["requestId"]);path.write_bytes(saved+b' ')
            return {"schema":"piv1-startup-signing-response-v1","requestId":payload["request"]["requestId"],"packets":[{"id":e.pid,"wireBase64":e.host.tx["signedWireBase64"]}]}
        try:
            with self.assertRaisesRegex(o.Stop,"quote_artifact_hash"):self.sign(e,auth,"COMMIT_RECEIPT",backend)
        finally:path.write_bytes(saved)
        e.host.refresh_tail();self.assertEqual(e.records()[-1]["kind"],"SIGN_ATTEMPT")
        inspected=e.host.call("INSPECT");self.assertIsNotNone(inspected["pendingSignerAttempt"])
        with self.assertRaisesRegex(o.Stop,"signer_attempt_unresolved"):self.sign(e,auth,"COMMIT_RECEIPT",backend)
        self.assertEqual(len(calls),1);self.assertFalse(e.sent)


if __name__=="__main__":unittest.main()
