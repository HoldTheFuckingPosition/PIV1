"""Finite admission reservations; no native worker, network or new signatures.

Counter/admission units use explicit prior-validated history seams. The small
controller case runs actual journals/replay and original-signature recovery
with the retained synthetic ELF/template/account and public historical wire.
"""
import base64
import copy
import importlib.util
import json
from pathlib import Path
import struct
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location("reservation_lifetime_helpers",HERE/"test_startup_lifetime_capture.py")
l=importlib.util.module_from_spec(spec);spec.loader.exec_module(l);o=l.o


class RetryReservationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        l.LifetimeCaptureTests.setUpClass();cls.r,cls.c,cls.s,cls.j=o.helpers();cls.a=o.startup_manifest();cls.ref=cls.a.oracle()
        cls.manifest={"schema":cls.a.SCHEMA,"phases":[{"id":f"startup_{i:04}","kind":cls.a.phase_kind(i),"initializerIndex":i-939 if 939<=i<=947 else None} for i in range(949)]}

    def record(self,kind,data):return {"kind":kind,"data":data}

    def test_exact_source_profile_matches_retained_independent_capacity_totals(self):
        slots=o.evidence_plan(self.manifest,self.ref);unit=o.SUCCESS_SERIALIZATION
        self.assertEqual(len(slots),2849);self.assertEqual(sum(slots.values()),7814844156)
        self.assertEqual(sum(v-unit for (kind,_),v in slots.items() if kind=="SEND"),1185359488)
        self.assertEqual(sum(v-unit for (kind,_),v in slots.items() if kind=="RECOVER"),3642071212)
        self.assertEqual(sum(v-unit for (kind,_),v in slots.items() if kind in ("PRE_VOTE","POST_VOTE")),20432)
        # Selected boundary rows from the independently executed E-l audit;
        # no dependency on external receipts or copied account fixture bytes.
        rows={0:(4185740,22468132),1:(1192952,3578856),936:(1192952,3578856),937:(5378528,22468412),940:(5380664,22478812),942:(5380648,22483052),948:(5385444,22502712)}
        for index,(send,recover) in rows.items():
            pid=f"startup_{index:04}"
            self.assertEqual((slots[("SEND",pid)]-unit,slots[("RECOVER",pid)]-unit),(send,recover))
        self.assertEqual(o.remaining_reservations(self.manifest,[self.record("CREATE",{})],slots),(7814844156,7597))
        self.assertEqual(1+7597,7598)
        self.assertEqual(o.MAX_EVIDENCE-7814844156-2*o.ACTION_EVIDENCE,238219524)

    def test_one_time_quote_batch_gate_intent_terminal_credits_never_renew(self):
        slots=o.evidence_plan(self.manifest,self.ref);initial=sum(slots.values());chain=[self.record("CREATE",{})]
        first,second=self.manifest["phases"][:2];entries=[{"id":p["id"]} for p in (first,second)]
        # Neither legacy list preparation nor a noncapture counter input frees
        # a quote slot. Actual dict PREPARE additionally requires its quote.
        chain+=[self.record("PREPARE",entries),self.record("PREPARE",{"entries":entries})]
        self.assertEqual(o.remaining_reservations(self.manifest,chain,slots)[0],initial)
        capture=self.record("PREPARE",{"entries":entries,"quote":{"path":"prior-bound-capture"}})
        chain+=[capture];remaining=initial-2*o.SUCCESS_SERIALIZATION
        self.assertEqual(o.remaining_reservations(self.manifest,chain,slots)[0],remaining)
        chain+=[copy.deepcopy(capture),self.record("ATTEMPT",{}),self.record("UNKNOWN",{})]
        self.assertEqual(o.remaining_reservations(self.manifest,chain,slots)[0],remaining)
        for stage,index in (("PRE_VOTE",943),("POST_VOTE",947)):
            gate=self.record(stage,{"id":self.manifest["phases"][index]["id"]});chain+=[gate,copy.deepcopy(gate)]
            remaining-=slots[(stage,gate["data"]["id"])]
        chain+=[self.record("BIND_SEAL",{}),self.record("INTENT",{"id":first["id"]})];remaining-=slots[("SEND",first["id"])]
        self.assertEqual(o.remaining_reservations(self.manifest,chain,slots),(remaining,7592))
        chain+=[self.record("ATTEMPT",{}),self.record("UNKNOWN",{})]
        self.assertEqual(o.remaining_reservations(self.manifest,chain,slots)[0],remaining)
        chain+=[self.record("FINALIZED",{"id":first["id"]})];remaining-=slots[("RECOVER",first["id"])]
        self.assertEqual(o.remaining_reservations(self.manifest,chain,slots),(remaining,7584))
        # Unquoted preparation's unused future quote retires only at INTENT.
        third=self.manifest["phases"][2]["id"];chain+=[self.record("INTENT",{"id":third})]
        self.assertEqual(o.remaining_reservations(self.manifest,chain,slots)[0],remaining-slots[("QUOTE",third)]-slots[("SEND",third)])

    def test_actual_orphan_superseded_unknown_bytes_and_exact_admission_boundaries(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp)/"operator";fd=self.j.directory(path,True);self.addCleanup(__import__('os').close,fd)
            root=path.with_name(path.name+".evidence");rfd=self.j.directory(root,True);__import__('os').close(rfd)
            # Every byte counts, regardless of journal linkage or success.
            for name in ("00000001-prepare-00","00000002-prepare-00","00000003-send","00000004-recover"):
                store=self.c.Store(root/name)
                try:store.write("retained.bin",b"x"*17)
                finally:store.close()
            nested=self.c.Store(root/"00000004-recover"/"collection")
            try:nested.write("partial-response.bin",b"x"*19)
            finally:nested.close()
            chain=[self.record("CREATE",{})];state={"manifest":self.manifest,"cursor":0};append=mock.Mock()
            op=SimpleNamespace(path=path,fd=fd,chain=chain,append=append)
            remaining,needed=o.remaining_reservations(self.manifest,chain,o.evidence_plan(self.manifest,self.ref));actual=4*17+19
            limit=actual+remaining+o.ACTION_EVIDENCE
            with mock.patch.object(o,"MAX_EVIDENCE",limit):
                self.assertEqual(o.evidence_root(self.j,op,state,"QUOTE"),root)
                (root/"00000003-send"/"retained.bin").write_bytes(b"x"*18)
                before={p.name for p in root.iterdir()}
                with self.assertRaisesRegex(o.Stop,"future_evidence_reservation_hold"):o.quote_store(self.c,self.j,op,state)
                with self.assertRaisesRegex(o.Stop,"future_evidence_reservation_hold"):o.evidence_store(self.c,self.j,op,"SEND",state,lambda raw:None)
                self.assertEqual({p.name for p in root.iterdir()},before);append.assert_not_called()
                # Read-only recovery bypasses future reservation, never caps.
                self.assertEqual(o.evidence_root(self.j,op,state,"RECOVER"),root)
            with mock.patch.object(o,"MAX_RECORDS",len(chain)+needed):
                self.assertEqual(o.evidence_root(self.j,op,state,"QUOTE"),root)
            with mock.patch.object(o,"MAX_RECORDS",len(chain)+needed-1):
                with self.assertRaisesRegex(o.Stop,"future_record_reservation_hold"):o.quote_store(self.c,self.j,op,state)
            with mock.patch.object(o,"MAX_RECORDS",len(chain)+1):
                with self.assertRaisesRegex(o.Stop,"recovery_record_headroom"):o.evidence_store(self.c,self.j,op,"RECOVER",state,lambda raw:None)
            append.assert_not_called()

    def test_forward_exhaustion_keeps_actual_inspection_original_recovery_and_hard_hold(self):
        e=l.Environment(self);e.capture();e.signed();e.host.call("SEND",{"id":e.pid},_transport=e.transport,**e.host.auth())
        chain=e.records();state=o.replay(e.host.j,chain);path=o.journal_path(e.host.base);root=path.with_name(path.name+".evidence")
        actual=sum(p.stat().st_size for p in root.rglob("*") if p.is_file())
        remaining,_=o.remaining_reservations(state["manifest"],chain,o.evidence_plan(state["manifest"],e.ref))
        fd=e.host.j.directory(path);self.addCleanup(__import__('os').close,fd);op=SimpleNamespace(path=path,fd=fd,chain=chain)
        with mock.patch.object(o,"MAX_EVIDENCE",actual+remaining+o.ACTION_EVIDENCE-1):
            with self.assertRaisesRegex(o.Stop,"future_evidence_reservation_hold"):o.quote_store(e.host.c,e.host.j,op,state)
            self.assertEqual(e.host.call("INSPECT")["status"],"UNKNOWN_OR_SUBMITTED_RECOVERY_ONLY")
        # Actual hard exhaustion refuses before ATTEMPT and does not poison
        # the committed original intent or inspection.
        tail=e.host.tail
        with mock.patch.object(o,"MAX_EVIDENCE",actual+o.ACTION_EVIDENCE-1):
            with self.assertRaisesRegex(o.Stop,"evidence_total_bound"):
                e.host.call("RECOVER",{"id":e.pid},_transport=e.transport,**e.host.auth())
            self.assertEqual(e.host.call("INSPECT")["tail"],tail)
        with mock.patch.object(o,"MAX_RECORDS",len(chain)+1):
            with self.assertRaisesRegex(o.Stop,"recovery_record_headroom"):
                e.host.call("RECOVER",{"id":e.pid},_transport=e.transport,**e.host.auth())
            self.assertEqual(e.host.call("INSPECT")["tail"],tail)
        def transport(raw):
            replies=[]
            for q in json.loads(raw):
                if q["method"] in ("getSignatureStatuses","getTransaction"):
                    _,_,body=e.host.transport(o.encode([q]));replies.extend(json.loads(body))
                elif q["method"]=="getMultipleAccounts":
                    accounts=copy.deepcopy(e.accounts);roles=e.ref["roles"];ctx=e.slot+1
                    g0,h2=e.a.actor_balances(1);accounts[roles["g0"]]["lamports"]=str(g0);accounts[roles["h2"]]["lamports"]=str(h2)
                    data=struct.pack("<IB",1,1)+e.host.c.b58decode(roles["g0"],32)+bytes(len(e.full.image))
                    accounts[roles["loaderBuffer"]]={"exists":True,"owner":e.a.LOADER,"executable":False,"lamports":e.ref["rents"]["894589"],"dataBase64":base64.b64encode(data).decode()}
                    accounts[roles["clock"]]["dataBase64"]=base64.b64encode(struct.pack("<QqQQq",ctx,0,0,0,100)).decode()
                    replies.append({"jsonrpc":"2.0","id":q["id"],"result":{"context":{"slot":ctx},"value":[e.host.wire_account(accounts[k]) for k in q["params"][0]]}})
                else:
                    _,_,body=e.transport(o.encode([q]));replies.extend(json.loads(body))
            return 200,[],o.encode(replies)
        with mock.patch.object(o,"MAX_EVIDENCE",actual+remaining+o.ACTION_EVIDENCE-1):
            result=e.host.call("RECOVER",{"id":e.pid},_transport=transport,**e.host.auth())
        self.assertEqual(result["completedPhases"],1);self.assertEqual(e.calls.count("sendTransaction"),1)
        self.assertEqual(o.replay(e.host.j,e.records())["intents"][e.pid]["signature"],e.host.tx["signature"])
        self.assertEqual({p.name:p.read_bytes() for p in e.host.base.iterdir()},e.host.base_bytes)


if __name__=="__main__":unittest.main()
