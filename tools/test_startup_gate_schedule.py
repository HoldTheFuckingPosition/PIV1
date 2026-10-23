"""Gate scheduling and pure durable readback, no signing or network.

Selected plans use the pinned public Node planner. Small new parent/gate files
are synthetic: parent semantic validation is a declared prior-bound-seal seam.
Controller order tests use an in-memory journal/replay seam, not upload history.
"""
import base64
import copy
from contextlib import contextmanager,ExitStack
import importlib.util
import json
import os
from pathlib import Path
import struct
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

HERE=Path(__file__).parent
def load(name,path):
    spec=importlib.util.spec_from_file_location(name,path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
o=load("gate_schedule_operator",HERE/"startup_release_operator.py")
h=load("gate_schedule_independent_serializers",HERE/"test_startup_initializer_storage.py")


class BoundaryReached(Exception):pass


class GateScheduleTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.r,cls.c,cls.s,cls.j=o.helpers();cls.a=o.startup_manifest();cls.ref=cls.a.oracle()
        cls.derived=cls.j._node_request({"preManifest":cls.ref["preManifest"],"anchor":"1234567890"},planner=True)
        cls.plan=cls.derived["plan"];cls.g=cls.derived["postManifest"]["initializer"]["governance"]
        cls.pdas=h.oracle_pdas(cls.g)

    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name)

    def case(self,stage="PRE_VOTE"):
        index=4 if stage=="PRE_VOTE" else 8;base=self.root/("base-"+stage)
        phase={"id":"phase_"+str(index),"kind":"INITIALIZER","initializerIndex":index,"feeCapLamports":str(self.ref["fees"][939+index]),"minimumRemainingBlockHeights":"40"}
        actual={"unsignedInitializerPlan":self.plan,"postManifest":self.derived["postManifest"],"compactSha256":o.sha(bytes.fromhex(self.plan["compact"]["hex"])),"nativeSha256":self.plan["native"]["sha256"],"contextFloor":"80","clock":{"contextSlot":"100"}}
        parent=self.root/("parent-"+stage);fd=self.j.directory(parent,True)
        try:
            self.j.write_file(fd,"lock",self.j.LOCK_BYTES);meta=os.stat("lock",dir_fd=fd)
            identity={"device":str(meta.st_dev),"inode":str(meta.st_ino),"ctimeNs":str(meta.st_ctime_ns)}
            body={"schema":self.j.RECORD_SCHEMA,"sequence":"0","previousDigest":None,"lockIdentity":identity,"kind":"SEAL","data":actual}
            record={**body,"digest":o.sha(o.encode(body))};self.j.write_file(fd,"00000000.json",o.encode(record))
        finally:os.close(fd)
        seal={"parentPath":str(parent),"parentTail":self.j.tail([record]),"baseParentTail":"b"*64,"sealDataSha256":o.sha(o.encode(actual)),"postManifestSha256":o.sha(o.encode(actual["postManifest"])),"nativeSha256":actual["nativeSha256"],"transportMode":"OFFLINE_INJECTED_TEST","stateBinding":{"plan":self.plan,"postManifest":actual["postManifest"],"proposalBump":self.pdas["proposal"]["bump"]}}
        ledger={"actualFeeLamports":"0","reservedFeeLamports":"0","cumulativeFeeCapLamports":"1"}
        state={"seal":seal,"baseParentPath":str(base),"floor":70,"manifest":{"phases":[phase],"parentTail":"b"*64,"historicalLedger":ledger,"proposedFeeBudget":{"feeCapLamports":"1"}},"cursor":0,"attempt":{"transportMode":"OFFLINE_INJECTED_TEST"}}
        evidence=o.journal_path(base);evidence=evidence.with_name(evidence.name+".evidence");output=evidence/("00000002-"+stage.lower())/"readback"
        output.parent.mkdir(parents=True,mode=0o700);os.chmod(evidence,0o700)
        snapshot=h.snapshot(actual,self.pdas,stage);snapshot["contextSlot"]="120"
        clock={"exists":True,"owner":self.r.SYSVAR,"executable":False,"lamports":"1","dataBase64":base64.b64encode(struct.pack("<QqQQq",120,0,0,0,1234567890)).decode()}
        storage={"status":"VERIFIED_SUPPLIED_INITIALIZER_STORAGE_ONLY","stage":stage,"parentTail":seal["parentTail"],"sealDataSha256":seal["sealDataSha256"],"postManifestSha256":seal["postManifestSha256"],"inheritedLedgerReference":ledger,"snapshotContextSlot":"120","snapshotSha256":o.sha(o.encode(snapshot)),"pdaDerivations":self.pdas,"evidence":self.s.validate_snapshot(snapshot,actual,self.pdas,stage)}
        timing=self.r.clock_evidence(clock,120,100,stage,storage,self.g["timeLockSeconds"])
        request={"schema":self.r.SCHEMA,"expectedTail":seal["parentTail"],"stage":stage,"minimumContextSlot":"70"}
        report={"schema":self.r.SCHEMA,"status":"VERIFIED_COLLECTED_STORAGE_AND_CLOCK_EVIDENCE_ONLY","endpoint":self.c.ENDPOINT,"expectedGenesis":self.c.GENESIS,"genesisMatched":True,"transportMode":"OFFLINE_INJECTED_TEST","parentTail":seal["parentTail"],"sealDataSha256":seal["sealDataSha256"],"postManifestSha256":seal["postManifestSha256"],"inheritedLedgerReference":ledger,"sourcePins":self.r.PINS,"minimumContextSlot":"100","accountKeys":[self.plan[k] for k in ("multisig","transaction","proposal")]+[self.r.CLOCK],"snapshotSha256":storage["snapshotSha256"],"snapshotContextSlot":"120","storageReport":storage,"clockEvidence":timing}
        store=self.c.Store(output)
        def write(name,value):store.write(name,o.encode(value))
        write("input.json",request);write("parent-reference.json",{k:report[k] for k in ("parentTail","sealDataSha256","postManifestSha256","inheritedLedgerReference","sourcePins")})
        values={**snapshot["accounts"],self.r.CLOCK:clock}
        replies=[self.c.GENESIS,{"context":{"slot":120},"value":[{"owner":values[k]["owner"],"executable":False,"lamports":int(values[k]["lamports"]),"data":[values[k]["dataBase64"],"base64"]} for k in report["accountKeys"]]}]
        for i,(method,params) in enumerate((("getGenesisHash",[]),("getMultipleAccounts",[report["accountKeys"],{"encoding":"base64","commitment":"finalized","minContextSlot":100}])),1):
            prefix=f"http-{i:02}";write(prefix+".request.json",[{"jsonrpc":"2.0","id":i,"method":method,"params":params}]);write(prefix+".started.json",{"synthetic":True});write(prefix+".response.bin",[{"jsonrpc":"2.0","id":i,"result":replies[i-1]}]);write(prefix+".http.json",{"status":200,"headers":[]})
        write("storage-snapshot.json",snapshot);write("clock-account.json",{"address":self.r.CLOCK,"contextSlot":"120",**clock});write("storage-request.json",{"schema":self.s.SCHEMA,"expectedTail":seal["parentTail"],"stage":stage,"snapshot":snapshot});write("storage-report.json",storage);write("clock-evidence.json",timing)
        report["artifacts"]=store.artifacts.copy();receipt=store.write("manifest.json",o.encode(report));store.close()
        data={"id":phase["id"],"contextSlot":"120","sealDataSha256":seal["sealDataSha256"],"snapshotSha256":storage["snapshotSha256"],"evidencePath":str(output),"manifestSha256":receipt["sha256"],"transportMode":"OFFLINE_INJECTED_TEST"}
        return SimpleNamespace(state=state,data=data,stage=stage,output=output,report=report,actual=actual,phase=phase,timing=timing)

    def rewrite_report(self,case,change):
        report=copy.deepcopy(case.report);change(report)
        raw=o.encode(report);(case.output/"manifest.json").write_bytes(raw);case.data["manifestSha256"]=o.sha(raw)

    def test_secure_pre_post_readback_and_replay_use_exact_source_pdas_without_node(self):
        for stage in ("PRE_VOTE","POST_VOTE"):
            case=self.case(stage)
            with mock.patch.object(o.subprocess,"Popen",side_effect=AssertionError("pure readback must not launch")):
                self.assertEqual(o.verify_gate(self.j,case.state,stage,case.data),case.timing)
                create={"kind":"CREATE","data":{"manifest":case.state["manifest"],"historicalLedger":case.state["manifest"]["historicalLedger"],"contextFloor":"70","lastFinalizedSlot":"0","baseParentPath":case.state["baseParentPath"]}}
                attempt={"id":case.phase["id"],"stage":stage,"evidencePath":str(case.output.parent),"transportMode":"OFFLINE_INJECTED_TEST"}
                records=[create,{"kind":"BIND_SEAL","data":case.state["seal"]},{"kind":"ATTEMPT","data":attempt},{"kind":stage,"data":case.data,"digest":"d"*64}]
                # Only selected-manifest/initial-seal validation is a prior-state
                # seam; actual gate replay, raw/PDA/Clock checks run unchanged.
                with mock.patch.object(o,"full_startup",return_value=True),mock.patch.object(self.a,"validate"),mock.patch.object(self.a,"validate_seal"),mock.patch.object(o,"startup_manifest",return_value=self.a):state=o.replay(self.j,records)
                self.assertEqual(state["voteGateAt"][stage],"d"*64);self.assertEqual(state["voteGateClock"][stage],case.timing)
                quote={key:None for key in ("schema","path","operatorSequence","operatorTail","operatorSourceSha256","baseParentPath","baseParentTail","manifestSha256","authorization","authorizationSha256","minimumContextSlot","transportMode","orderedPreparedSha256","manifestSha256Receipt")}
                quote.update(schema="piv1-startup-lifetime-quote-v1",path=str(case.output.parent.parent/"00000001-prepare-00"),operatorSequence="1",operatorTail="e"*64,operatorSourceSha256=o.sha(Path(o.__file__).read_bytes()))
                with self.assertRaisesRegex(o.Stop,"quote_gate_predecessor"):
                    o.verify_quote(self.c,self.j,state,[{"id":case.phase["id"]}],quote,{"sequence":"1","previousDigest":"e"*64})
            # Real selected unsigned packet/decode and exclusive raw quote
            # capture+verification follow the real gate-derived replay state.
            # Only prior launcher authorization and public RPC are explicit fakes.
            index=case.phase["initializerIndex"];wire=self.plan["packets"][index]["wireBase64"]
            raw=base64.b64decode(wire);tables=[{"address":self.plan["alt"]["address"],"addresses":self.plan["alt"]["addresses"]}] if raw[1+64*raw[0]]==128 else []
            review=o.describe(self.j,[{"wireBase64":wire,"lookupTables":tables}])[0]
            fd=self.j.directory(o.journal_path(state["baseParentPath"]),True)
            try:
                op=SimpleNamespace(path=o.journal_path(state["baseParentPath"]),fd=fd,chain=records,tail=lambda:"d"*64)
                calls=[];blockhash=self.c.b58encode(bytes([1])*32)
                def transport(raw):
                    questions=json.loads(raw);replies=[]
                    for q in questions:
                        calls.append(q)
                        if q["method"]=="getGenesisHash":value=self.c.GENESIS
                        elif q["method"]=="getLatestBlockhash":value={"context":{"slot":130},"value":{"blockhash":blockhash,"lastValidBlockHeight":1000}}
                        elif q["method"]=="getFeeForMessage":value={"context":{"slot":130},"value":int(case.phase["feeCapLamports"])}
                        elif q["method"]=="getBlockHeight":value=100
                        elif q["method"]=="isBlockhashValid":value={"context":{"slot":130},"value":True}
                        else:raise AssertionError("unexpected fake quote method")
                        replies.append({"jsonrpc":"2.0","id":q["id"],"result":value})
                    return 200,[],o.encode(replies)
                auth=o.encode({})
                with mock.patch.object(o,"authorization",return_value=o.sha(auth)):
                    entries,quote=o.capture_quote(self.c,self.j,op,state,[{"id":case.phase["id"],"wireBase64":wire}],[review],auth,o.sha(auth),transport)
                    self.assertEqual(o.verify_quote(self.c,self.j,state,entries,quote,{"sequence":"4","previousDigest":"d"*64}),130)
                self.assertEqual(quote["operatorTail"],state["voteGateAt"][stage]);self.assertEqual(quote["minimumContextSlot"],"120")
                self.assertEqual(calls[1]["params"][0]["minContextSlot"],120)
                self.assertEqual([q["method"] for q in calls],["getGenesisHash","getLatestBlockhash","getFeeForMessage","getBlockHeight","isBlockhashValid"])
            finally:os.close(fd)

    def test_raw_parent_source_report_and_artifact_changes_refuse_reuse(self):
        case=self.case()
        original=(case.output/"manifest.json").read_bytes();receipt=case.data["manifestSha256"]
        changes=(lambda r:r.update(sourcePins={}),lambda r:r.update(minimumContextSlot="70"),lambda r:r["storageReport"]["pdaDerivations"]["multisig"].update(bump=0))
        for change in changes:
            self.rewrite_report(case,change)
            with self.assertRaises(ValueError):o.verify_gate(self.j,case.state,case.stage,case.data)
            (case.output/"manifest.json").write_bytes(original);case.data["manifestSha256"]=receipt
        raw=(case.output/"storage-snapshot.json").read_bytes();(case.output/"storage-snapshot.json").write_bytes(b'{}\n')
        with self.assertRaisesRegex(o.Stop,"gate_artifact_hash"):o.verify_gate(self.j,case.state,case.stage,case.data)
        (case.output/"storage-snapshot.json").write_bytes(raw)
        case.state["seal"]["parentTail"]={"sequence":"1","digest":"a"*64}
        with self.assertRaises(ValueError):o.verify_gate(self.j,case.state,case.stage,case.data)

    @contextmanager
    def controller(self,index):
        # Explicit scheduling-only controller seam. No durable/native-control
        # claim follows from this in-memory state; raw replay is tested above.
        review=copy.deepcopy(self.ref["staticMessages"]["bootstrap"])
        phases=[{"id":f"init_{i}","kind":"INITIALIZER","initializerIndex":i,"feeCapLamports":"1","minimumRemainingBlockHeights":"40"} for i in range(9)]
        base=self.root/"controller";state={"manifest":{"phases":phases,"packageSha256":"c"*64},"cursor":index,"failed":False,"signerAttempt":None,"intents":{},"signed":{phases[index]["id"]:{"requestId":"a","authorizationSha256":"a"*64}},"prepared":{},"transportMode":"OFFLINE_INJECTED_TEST","seal":{"reviewedPackets":[review]*9,"nativeSha256":self.plan["native"]["sha256"],"anchorUnixTimestamp":"1234567890","transactionIndex":self.g["transactionIndex"]},"quoteByPhase":{},"voteGateAt":{},"voteGateClock":{},"preVote":{},"postVote":{}}
        events=[];chain=[{"data":{"baseParentPath":str(base)}}];op=SimpleNamespace(chain=chain,path=o.journal_path(base),tail=lambda:"d"*64,close=lambda:None)
        def append(kind,data):events.append(kind);chain.append({"kind":kind,"data":data});return chain[-1]
        op.append=append
        def collected(r,c,j,op,current,stage,transport):
            append("ATTEMPT",{});return {"stage":stage}
        def replay(j,records):
            if len(records)>1 and records[-1]["kind"] in ("PRE_VOTE","POST_VOTE"):state["voteGateAt"][records[-1]["kind"]]="d"*64
            return state
        with ExitStack() as stack:
            for name,value in (("Journal",mock.Mock(return_value=op)),("replay",replay),("full_startup",lambda m:True),("authorization",lambda *a,**kw:"a"*64),("describe",lambda *a,**kw:[review]*len(a[1])),("active_phase",lambda c,s,p:p),("check_signed",mock.Mock()),("gate",mock.Mock(side_effect=collected))):stack.enter_context(mock.patch.object(o,name,value))
            yield SimpleNamespace(state=state,events=events,op=op,base=base,phases=phases,gate=o.gate)

    def test_captured_prepare_collects_before_quote_recollects_and_refuses_future_gates(self):
        for index in (4,8):
            with self.controller(index) as f:
                pid=f.phases[index]["id"];request={"schema":o.SCHEMA,"action":"PREPARE","expectedTail":f.op.tail(),"data":{"captureLifetime":True,"packets":[{"id":pid,"wireBase64":"explicit-template-seam"}]}}
                def quote(*args):f.events.append("QUOTE");raise BoundaryReached()
                with mock.patch.object(o,"capture_quote",side_effect=quote):
                    for _ in range(2):
                        with self.assertRaises(BoundaryReached):o.operate(request,f.base,_transport=lambda raw:None,authorization_raw=o.encode({}))
                stage="PRE_VOTE" if index==4 else "POST_VOTE"
                self.assertEqual(f.events,["ATTEMPT",stage,"QUOTE"]*2)
                f.events.clear()
                def incomplete(*args):f.op.append("ATTEMPT",{});raise o.Stop("collection_incomplete")
                f.gate.side_effect=incomplete
                with mock.patch.object(o,"capture_quote") as quote:
                    with self.assertRaisesRegex(o.Stop,"collection_incomplete"):o.operate(request,f.base,_transport=lambda raw:None,authorization_raw=o.encode({}))
                    quote.assert_not_called()
                self.assertEqual(f.events,["ATTEMPT"])
            with self.controller(index-1) as f:
                request={"schema":o.SCHEMA,"action":"PREPARE","expectedTail":f.op.tail(),"data":{"captureLifetime":True,"packets":[{"id":p["id"],"wireBase64":"seam"} for p in f.phases[index-1:index+1]]}}
                with self.assertRaisesRegex(o.Stop,"quote_crosses_vote_boundary"):o.operate(request,f.base,_transport=lambda raw:None,authorization_raw=o.encode({}))
                f.gate.assert_not_called();self.assertEqual(f.events,[])

    def test_send_skips_only_exact_captured_gate_and_preserves_clock_timelock_checks(self):
        for index in (4,8):
            with self.controller(index) as f:
                phase=f.phases[index];pid=phase["id"];stage=o.vote_stage(phase);f.state["prepared"][pid]={};f.state["voteGateAt"][stage]="d"*64
                request={"schema":o.SCHEMA,"action":"SEND","expectedTail":f.op.tail(),"data":{"id":pid}}
                f.state["quoteByPhase"][pid]={"authorizationSha256":"a"*64,"operatorTail":"d"*64}
                with mock.patch.object(o,"evidence_store",side_effect=BoundaryReached()):
                    with self.assertRaises(BoundaryReached):o.operate(request,f.base,_transport=lambda raw:None)
                    f.gate.assert_not_called()
                    f.state["quoteByPhase"][pid]["operatorTail"]="e"*64
                    with self.assertRaisesRegex(o.Stop,"quote_gate_predecessor"):o.operate(request,f.base,_transport=lambda raw:None)
                    f.gate.assert_not_called();f.state["quoteByPhase"].clear()
                    with self.assertRaises(BoundaryReached):o.operate(request,f.base,_transport=lambda raw:None)
                    f.gate.assert_called_once();self.assertEqual(f.events,["ATTEMPT",stage])
                f.state["voteGateClock"][stage]={"unixTimestamp":"100","approvedTimestamp":"99","configuredDelaySeconds":"1"}
                o.verify_gate_clock(f.state,phase,{"unixTimestamp":"100"})
                with self.assertRaisesRegex(o.Stop,"gate_clock_regression"):o.verify_gate_clock(f.state,phase,{"unixTimestamp":"99"})
                if index==8:
                    f.state["voteGateClock"][stage]["approvedTimestamp"]=str(-(1<<63))
                    with self.assertRaisesRegex(o.Stop,"gate_timelock_arithmetic"):o.verify_gate_clock(f.state,phase,{"unixTimestamp":"100"})


if __name__=="__main__":unittest.main()
