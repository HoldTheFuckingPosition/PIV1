"""Successful evidence envelopes, with historical signatures and fake public I/O.

The small full-host fixture retains its explicit synthetic ELF/account/template
seams. No native worker, network, signing, key read or large history is used.
Failure/retry aggregate capacity is deliberately not claimed by these tests.
"""
import base64
import copy
import importlib.util
import json
from pathlib import Path
import struct
import tempfile
import unittest
from unittest import mock

HERE=Path(__file__).parent
def load(name,path):
    spec=importlib.util.spec_from_file_location(name,path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
l=load("envelope_lifetime_helpers",HERE/"test_startup_lifetime_capture.py");o=l.o
g=load("envelope_gate_helpers",HERE/"test_startup_gate_schedule.py")


class EvidenceEnvelopeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        l.LifetimeCaptureTests.setUpClass()
        cls.r,cls.c,cls.s,cls.j=o.helpers();cls.a=o.startup_manifest()

    def test_source_account_copy_arithmetic_and_exact_success_size_boundary(self):
        ref=self.a.oracle();digests=copy.deepcopy(ref["baseline"]);roles=ref["roles"]
        # The accepted Buffer-create POST adds this exact source allocation.
        digests[roles["loaderBuffer"]]={"exists":True,"dataBytes":894589}
        proof=self.a.proof_keys({"kind":"BUFFER_CREATE"})
        complete=o.account_payload(digests);subset=o.account_payload(digests,proof)
        self.assertEqual(complete-subset,4*((1106125+2)//3))
        self.assertEqual(o.account_payload({"buffer":digests[roles["loaderBuffer"]]}),1192788)
        recovery=3*subset+2*complete
        expected=3*sum(4*((d["dataBytes"]+2)//3) for key,d in digests.items() if d["exists"] and key in proof)+2*sum(4*((d["dataBytes"]+2)//3) for d in digests.values() if d["exists"])
        self.assertEqual(recovery,expected)
        gate=3*sum(4*((size+2)//3) for size in (330,1548,646))+2*56
        self.assertEqual(gate,10216)
        self.assertEqual((o.ACTION_EVIDENCE,o.MAX_EVIDENCE,o.MAX_JOURNAL,o.MAX_RECORDS),(256*1024**2,8*1024**3,128*1024**2,8192))
        with tempfile.TemporaryDirectory() as tmp:
            stores=[self.c.Store(Path(tmp)/name) for name in ("collection","full-state")]
            try:
                stores[0].write("response.bin",b"x"*subset)
                stores[1].write("snapshot.json",b"x"*(recovery-subset+o.SUCCESS_SERIALIZATION))
                o.successful_evidence(self.j,[s.path for s in stores],recovery)
                stores[1].write("extra.json",b" ")
                with self.assertRaisesRegex(o.Stop,"successful_evidence_envelope"):
                    o.successful_evidence(self.j,[s.path for s in stores],recovery)
            finally:
                for store in stores:store.close()

    def test_valid_padded_quote_and_preflight_refuse_without_send_and_keep_raw(self):
        e=l.Environment(self);before=e.records()
        def padded(raw):
            status,headers,body=e.transport(raw)
            if any(q["method"] in ("getLatestBlockhash","getVersion") for q in json.loads(raw)):
                body+=b" "*(o.SUCCESS_SERIALIZATION+1)
                self.assertIsInstance(e.host.c.strict_json(body),list)
            return status,headers,body
        e.capturing=True
        try:
            with self.assertRaisesRegex(o.Stop,"successful_evidence_envelope"):
                e.host.call("PREPARE",{"captureLifetime":True,"packets":[{"id":e.pid,"wireBase64":e.host.prepared["wireBase64"]}]},_transport=padded,**e.host.auth())
        finally:e.capturing=False
        self.assertEqual(e.records(),before)
        root=o.journal_path(e.host.base).with_name(o.journal_path(e.host.base).name+".evidence")
        orphan=next(root.iterdir());saved={p.name:p.read_bytes() for p in orphan.iterdir()}
        self.assertIn("manifest.json",saved)
        self.assertGreater(len(saved["rpc-01.response.bin"]),o.SUCCESS_SERIALIZATION)
        e.capture();e.signed()
        self.assertEqual({p.name:p.read_bytes() for p in orphan.iterdir()},saved)
        with self.assertRaisesRegex(o.Stop,"successful_evidence_envelope"):
            e.host.call("SEND",{"id":e.pid},_transport=padded,**e.host.auth())
        self.assertFalse(e.sent);self.assertFalse(any(r["kind"]=="INTENT" for r in e.records()))
        self.assertEqual(e.records()[-1]["kind"],"ATTEMPT")
        e.host.refresh_tail();self.assertFalse(e.host.call("INSPECT")["liveReady"])

    def test_post_intent_overflow_keeps_inspection_and_original_signature_recovery(self):
        for padded_stage in ("lifetime","send","authorization"):
            e=l.Environment(self)
            try:
                e.capture();e.signed();pad=[padded_stage];recovery_pad=[False]
                def transport(raw):
                    questions=json.loads(raw);replies=[]
                    for q in questions:
                        method=q["method"]
                        if method in ("getSignatureStatuses","getTransaction"):
                            _,_,body=e.host.transport(o.encode([q]))
                        else:_,_,body=e.transport(o.encode([q]))
                        values=json.loads(body)
                        if method=="getMultipleAccounts" and (e.sent or recovering[0]):
                            accounts=copy.deepcopy(e.accounts);roles=e.ref["roles"];ctx=e.slot+1
                            g0,h2=e.a.actor_balances(1);accounts[roles["g0"]]["lamports"]=str(g0);accounts[roles["h2"]]["lamports"]=str(h2)
                            buffer=struct.pack("<IB",1,1)+e.host.c.b58decode(roles["g0"],32)+bytes(len(e.full.image))
                            accounts[roles["loaderBuffer"]]={"exists":True,"owner":e.a.LOADER,"executable":False,"lamports":e.ref["rents"]["894589"],"dataBase64":base64.b64encode(buffer).decode()}
                            accounts[roles["clock"]]["dataBase64"]=base64.b64encode(struct.pack("<QqQQq",ctx,0,e.jito_epoch,0,100)).decode()
                            values[0]["result"]={"context":{"slot":ctx},"value":[e.host.wire_account(accounts[k]) for k in q["params"][0]]}
                        replies.extend(values)
                    body=o.encode(replies)
                    late_lifetime=any(q["method"]=="getLatestBlockhash" for q in questions) and not e.capturing
                    if (pad[0]=="lifetime" and late_lifetime) or (pad[0]=="send" and any(q["method"]=="sendTransaction" for q in questions)) or (recovery_pad[0] and any(q["method"]=="getSignatureStatuses" for q in questions)):
                        body+=b" "*(o.SUCCESS_SERIALIZATION+1)
                    return 200,[],body
                recovering=[False]
                if padded_stage=="authorization":
                    real_envelope=o.successful_evidence;clock=[1]
                    def expire_during_envelope(j,paths,payload,**kw):
                        result=real_envelope(j,paths,payload,**kw)
                        if kw.get("preflight_queries") is None and paths[0].name.endswith("-send"):
                            clock[0]=18446744073709551615
                        return result
                    with mock.patch.object(o,"successful_evidence",side_effect=expire_during_envelope),mock.patch.object(o.time,"time",side_effect=lambda:clock[0]):
                        with self.assertRaisesRegex(o.Stop,"launcher_authorization_expired"):
                            e.host.call("SEND",{"id":e.pid},_transport=transport,**e.host.auth())
                    self.assertFalse(e.sent);self.assertEqual(e.records()[-1]["kind"],"INTENT")
                elif padded_stage=="lifetime":
                    with self.assertRaisesRegex(o.Stop,"successful_evidence_envelope"):
                        e.host.call("SEND",{"id":e.pid},_transport=transport,**e.host.auth())
                    self.assertFalse(e.sent);self.assertEqual(e.records()[-1]["kind"],"INTENT")
                    # The synthetic historical inclusion below is an explicit
                    # original-signature proof seam, not evidence of this send.
                else:
                    e.host.call("SEND",{"id":e.pid},_transport=transport,**e.host.auth())
                    self.assertTrue(e.sent);self.assertEqual(e.records()[-1]["kind"],"UNKNOWN")
                e.host.refresh_tail();report=e.host.call("INSPECT")
                self.assertEqual(report["status"],"UNKNOWN_OR_SUBMITTED_RECOVERY_ONLY")
                self.assertEqual(report["proposedFeeLedger"]["reservedFeeLamports"],"10000")
                reads=len(e.calls)
                with self.assertRaisesRegex(o.Stop,"intent_exists_recovery_only"):
                    e.host.call("SEND",{"id":e.pid},_transport=transport,**e.host.auth())
                self.assertEqual(len(e.calls),reads)
                # Oversize completed recovery must leave the original INTENT
                # unresolved too; a later unique read-only attempt may succeed.
                recovering[0]=True;pad[0]=None;recovery_pad[0]=True
                with self.assertRaisesRegex(o.Stop,"successful_evidence_envelope"):
                    e.host.call("RECOVER",{"id":e.pid},_transport=transport,**e.host.auth())
                e.host.refresh_tail();self.assertEqual(e.records()[-1]["kind"],"ATTEMPT")
                self.assertEqual(e.host.call("INSPECT")["completedPhases"],0)
                recovery_pad[0]=False
                result=e.host.call("RECOVER",{"id":e.pid},_transport=transport,**e.host.auth())
                self.assertEqual(result["completedPhases"],1)
                state=o.replay(e.host.j,e.records());intent=state["intents"][e.pid]
                self.assertEqual(intent["signature"],e.host.tx["signature"])
                self.assertEqual(e.calls.count("sendTransaction"),int(padded_stage=="send"))
                # The retained post-INTENT overflow is still present after
                # successful recovery; replay checks only its preflight prefix.
                if padded_stage!="authorization":
                    self.assertGreater(o.evidence_bytes(e.host.j,Path(intent["evidencePath"])),o.account_payload(intent["coverage"]["accountDigests"])+o.SUCCESS_SERIALIZATION)
            finally:e.full.doCleanups()

    def test_gate_padding_refuses_after_real_raw_and_pure_checks(self):
        g.GateScheduleTests.setUpClass()
        case_helper=g.GateScheduleTests(methodName="test_secure_pre_post_readback_and_replay_use_exact_source_pdas_without_node")
        case_helper.setUp();self.addCleanup(case_helper.doCleanups)
        case=case_helper.case()
        self.assertEqual(g.o.verify_gate(case_helper.j,case.state,case.stage,case.data),case.timing)
        path=case.output/"http-01.response.bin";body=path.read_bytes()+b" "*(o.SUCCESS_SERIALIZATION+1);path.write_bytes(body)
        report=copy.deepcopy(case.report)
        artifact=next(a for a in report["artifacts"] if a["name"]==path.name);artifact.update(bytes=len(body),sha256=o.sha(body))
        raw=o.encode(report);(case.output/"manifest.json").write_bytes(raw);case.data["manifestSha256"]=o.sha(raw)
        with self.assertRaisesRegex(g.o.Stop,"successful_evidence_envelope"):
            g.o.verify_gate(case_helper.j,case.state,case.stage,case.data)
        self.assertEqual(path.read_bytes(),body)


if __name__=="__main__":unittest.main()
