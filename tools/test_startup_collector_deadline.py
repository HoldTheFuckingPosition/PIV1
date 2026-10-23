"""Whole native collection deadlines with fake children and public host fixtures.

No transport child or network runs. Literal-worker dispatch uses explicit pinned
collector seams; controller signatures/ELF/accounts retain existing test seams.
"""
import base64
import builtins
from contextlib import contextmanager, ExitStack
import importlib.util
import io
import json
import os
from pathlib import Path
import resource
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location("collector_deadline_helpers",HERE/"test_startup_rpc_deadline.py")
d=importlib.util.module_from_spec(spec);spec.loader.exec_module(d);o=d.o;t=d.t


@contextmanager
def collector_child(frame=b'',*,drip=False,exitcode=0,spawn=None,group_kill_error=False):
    real_read=os.read;real_write=os.write
    with d.fake_child(frame,drip=drip,group_kill_error=group_kill_error) as child:
        observed=[False];wait=child.process.wait;pipe_read=os.read;pipe_write=os.write
        def waitid(*args):
            observed[0]=True
            return SimpleNamespace(si_code=os.CLD_EXITED,si_status=exitcode)
        def reap(timeout=None):
            result=wait(timeout)
            if observed[0]:child.process.returncode=exitcode;return exitcode
            return result
        child.process.wait=mock.Mock(side_effect=reap)
        def popen(argv,*args,**kwargs):
            expected=[str(Path(o.sys.executable).resolve()),"-I","-S","-c",o.COLLECTION_SCRIPT]
            if argv!=expected:raise AssertionError("collector fake may only start the literal worker")
            if spawn is not None:spawn()
            return child.process
        child.popen.side_effect=popen
        with ExitStack() as stack:
            stack.enter_context(mock.patch.object(o.os,"read",side_effect=lambda fd,size:pipe_read(fd,size) if fd in (101,102) else real_read(fd,size)))
            stack.enter_context(mock.patch.object(o.os,"write",side_effect=lambda fd,raw:pipe_write(fd,raw) if fd in (101,102) else real_write(fd,raw)))
            child.peek=stack.enter_context(mock.patch.object(o.os,"waitid",side_effect=waitid))
            yield child


class CollectorDeadlineTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):d.RpcDeadlineTests.setUpClass();cls.r,cls.c,_,cls.j=o.helpers()

    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name)
        self.host=t.t.h.OperatorTests(methodName="test_nullable_imports_no_helpers")
        self.host.setUp();self.addCleanup(self.host.doCleanups)

    def request(self):
        tx=self.host.tx
        return {"schema":self.c.SCHEMA,"wireBase64":tx["signedWireBase64"],"expectedWireSha256":tx["signedWireSha256"],"expectedMessageSha256":tx["messageSha256"],"signature":tx["signature"],"expectedFeeLamports":tx["feeLamports"],"accountKeys":list(t.t.h.FIXTURE["fundPoststate"]["accounts"]),"minimumContextSlot":"1","includeExpiry":False}

    def payload(self,mode,request,output,parent=None,seal=None):
        return {"schema":"piv1-startup-collection-request-v1","mode":mode,"request":request,"outputPath":str(output),"parentPath":str(parent) if parent is not None else None,"sealSha256":seal}

    def prepared(self,mode="RECOVER",*,change=None):
        output=self.root/"output";prepared=self.root/"prepared";parent=self.root/"sealed-parent";seal="b"*64
        if mode=="RECOVER":
            request=self.request()
            with mock.patch.object(self.c,"tls_post",side_effect=self.host.transport):report=self.c.collect(request,prepared,_transport=None)
        else:
            request={"schema":self.r.SCHEMA,"expectedTail":{"sequence":"5","digest":"a"*64},"stage":"PRE_VOTE","minimumContextSlot":"100"}
            store=self.c.Store(prepared);artifact=store.write("input.json",o.encode(request));store.close()
            report={"schema":self.r.SCHEMA,"transportMode":"FIXED_ORIGIN_TLS","endpoint":self.c.ENDPOINT,"expectedGenesis":self.c.GENESIS,"parentTail":request["expectedTail"],"sealDataSha256":seal,"sourcePins":self.r.PINS,"minimumContextSlot":"100","storageReport":{"stage":"PRE_VOTE","parentTail":request["expectedTail"]},"artifacts":[artifact]}
        if change is not None:change(report)
        # Any edits here are to a new test-only report, never retained evidence.
        raw=o.encode({key:value for key,value in report.items() if key!="manifestReceipt"})
        (prepared/"manifest.json").write_bytes(raw);os.chmod(prepared/"manifest.json",0o600)
        receipt={"name":"manifest.json","bytes":len(raw),"sha256":o.sha(raw)}
        value=self.payload(mode,request,output,parent if mode=="GOVERNANCE" else None,seal if mode=="GOVERNANCE" else None)
        frame={"schema":"piv1-startup-collection-result-v1","mode":mode,"requestSha256":o.sha(o.encode(value)),"outputPath":str(output),"manifestReceipt":receipt}
        options={"parent_path":parent,"seal_sha256":seal,"locked_operator_path":self.root/"operator"} if mode=="GOVERNANCE" else {}
        return SimpleNamespace(output=output,prepared=prepared,request=request,frame=frame,options=options,report=report,mode=mode)

    def run_prepared(self,case,*,frame=None,spawn=None):
        with collector_child(o.encode(frame or case.frame),spawn=spawn or (lambda:case.prepared.rename(case.output))) as child:
            report=o.native_collection(case.mode,case.request,case.output,**case.options)
        return report,child

    def test_complete_native_receipt_revalidates_raw_evidence_and_preserves_mode(self):
        case=self.prepared();report,child=self.run_prepared(case)
        self.assertEqual(report["status"],"COLLECTED_PUBLIC_EVIDENCE_ONLY")
        self.assertEqual(report["transportMode"],"FIXED_ORIGIN_TLS")
        self.assertEqual(report["manifestReceipt"],case.frame["manifestReceipt"])
        self.assertEqual(report["observation"]["transaction"]["wireBase64"],case.request["wireBase64"])
        child.peek.assert_called_once_with(os.P_PID,child.process.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
        self.assertEqual(child.process.returncode,0)
        self.assertEqual([event[0] for event in child.events],["kill","wait"])
        self.assertEqual(child.popen.call_args.args[0][-1],o.COLLECTION_SCRIPT)
        self.assertTrue(child.popen.call_args.kwargs["start_new_session"])

    def test_stale_or_changed_manifest_artifact_and_request_bindings_refuse(self):
        for fault in ("manifest","artifact","output","request","source"):
            with self.subTest(fault=fault):
                case=self.prepared()
                frame=dict(case.frame)
                if fault=="output":frame["outputPath"]=str(self.root/"wrong-output")
                if fault=="request":frame["requestSha256"]="0"*64
                def spawn():
                    case.prepared.rename(case.output)
                    if fault=="manifest":(case.output/"manifest.json").write_bytes(b'{}\n')
                    if fault=="artifact":(case.output/"input.json").write_bytes(b'{}\n')
                with collector_child(o.encode(frame),spawn=spawn) as child:
                    if fault=="source":
                        calls=[0];original=o.helpers
                        def helpers():
                            calls[0]+=1
                            if calls[0]>1:raise o.Stop("helper_source_pin")
                            return original()
                        with mock.patch.object(o,"helpers",side_effect=helpers),self.assertRaisesRegex(o.Stop,"native_collection_incomplete_no_adoption"):o.native_collection("RECOVER",case.request,case.output)
                    else:
                        with self.assertRaisesRegex(o.Stop,"native_collection_incomplete_no_adoption"):o.native_collection("RECOVER",case.request,case.output)
                self.assertIsNotNone(child.process.returncode)
                __import__('shutil').rmtree(case.output)

    def test_governance_parent_stage_floor_and_held_lock_are_bound(self):
        for field,value in (("parentTail",{"sequence":"6","digest":"a"*64}),("sealDataSha256","0"*64),("minimumContextSlot","99"),("storageReport",{"stage":"POST_VOTE","parentTail":{"sequence":"5","digest":"a"*64}})):
            with self.subTest(field=field):
                case=self.prepared("GOVERNANCE",change=lambda report:report.update({field:value}))
                with self.assertRaisesRegex(o.Stop,"native_collection_incomplete_no_adoption"):self.run_prepared(case)
                __import__('shutil').rmtree(case.output)
        case=self.prepared("GOVERNANCE")
        with mock.patch.object(o.subprocess,"Popen",side_effect=AssertionError("held operator must not be acquired")):
            with self.assertRaisesRegex(o.Stop,"collection_parent_is_operator"):
                o.native_collection("GOVERNANCE",case.request,case.output,parent_path=case.options["parent_path"],seal_sha256="b"*64,locked_operator_path=case.options["parent_path"])
        report,_=self.run_prepared(case);self.assertEqual(report["storageReport"]["stage"],"PRE_VOTE")

    def test_timeout_and_exited_leader_still_kill_group_and_reap_without_adoption(self):
        for drip,error in ((True,False),(False,False),(False,True)):
            with self.subTest(drip=drip,group_error=error),collector_child(b'{}\n',drip=drip,group_kill_error=error) as child:
                with self.assertRaisesRegex(o.Stop,"native_collection_incomplete_no_adoption"):o.native_collection("RECOVER",self.request(),self.root/"unused")
            kinds=[event[0] for event in child.events]
            self.assertEqual(kinds[0],"kill");self.assertEqual(kinds[-1],"wait")
            if error:self.assertIn("fallback-kill",kinds)
            if drip:self.assertGreaterEqual(child.tick[0],180)
            self.assertIsNotNone(child.process.returncode)
        case=self.prepared()
        with collector_child(o.encode(case.frame),spawn=lambda:case.prepared.rename(case.output),group_kill_error=True) as child:
            with self.assertRaisesRegex(o.Stop,"native_collection_incomplete_no_adoption"):
                o.native_collection("RECOVER",case.request,case.output)
        # Even complete valid evidence and a reaped leader cannot authorize
        # acceptance after group signaling failed to confirm descendant stop.
        self.assertEqual(child.process.returncode,0);child.process.kill.assert_called_once()

    def test_exact_worker_literal_dispatches_unchanged_collectors_with_native_none(self):
        real_exec=builtins.exec
        for mode in ("RECOVER","GOVERNANCE"):
            with self.subTest(mode=mode):
                case=self.prepared(mode);output=io.BytesIO();calls=[]
                def collect(request,*args,**kwargs):
                    calls.append((request,args,kwargs));case.prepared.rename(case.output)
                    return {**case.report,"manifestReceipt":case.frame["manifestReceipt"]}
                def execute(code,namespace,*args):
                    result=real_exec(code,namespace,*args)
                    if namespace.get("__name__")=="fixed_native_collection":
                        # Explicit collector seam after real pinned source exec;
                        # no Node, TLS, parent lock or child is actually invoked.
                        namespace["helpers"]=lambda:(self.c,None,self.j)
                        namespace["collect"]=collect
                    return result
                with mock.patch.object(builtins,"exec",side_effect=execute),mock.patch.object(self.c,"collect",side_effect=collect),mock.patch.object(o.sys,"stdin",SimpleNamespace(buffer=io.BytesIO(o.encode(self.payload(mode,case.request,case.output,case.options.get("parent_path"),case.options.get("seal_sha256")))))),mock.patch.object(o.sys,"stdout",SimpleNamespace(buffer=output)),mock.patch.object(resource,"setrlimit"),mock.patch.dict(os.environ,{"PIV1_PUBLIC_CHECK_REPO":str(HERE.parent)}):
                    real_exec(compile(o.COLLECTION_SCRIPT,"<fixed-collector-worker>","exec"),{"__name__":"fake_collector_dispatch_only"})
                self.assertEqual(json.loads(output.getvalue()),case.frame)
                self.assertEqual(calls[0][2],{"_transport":None})
                __import__('shutil').rmtree(case.output)

    def test_native_recover_timeout_keeps_original_intent_reservation_and_unique_attempts(self):
        e=t.Environment(self);e.host.prepared["lifetime"]["lastValidBlockHeight"]="1200";e.rotated=e.original
        e.host.call("PREPARE",[e.host.prepared]);e.signed()
        with mock.patch.object(o,"public_rpc_transport",side_effect=lambda c,raw:e.transport(raw)):
            e.host.call("SEND",{"id":e.pid},live=True,**e.host.auth())
        before=e.records();intent=next(record["data"] for record in before if record["kind"]=="INTENT")
        self.assertEqual(intent["transportMode"],"FIXED_ORIGIN_TLS");outputs=[]
        def interrupted(mode,request,output):
            # Runner termination is tested above. This explicit dispatch seam
            # isolates the real controller's native route and durable HOLD.
            self.assertEqual(mode,"RECOVER");self.assertFalse(request["includeExpiry"])
            self.assertEqual(request["signature"],intent["signature"])
            path=Path(output);path.mkdir(mode=0o700);target=path/"incomplete.bin"
            target.write_bytes(b'public partial');os.chmod(target,0o600);outputs.append(path)
            raise o.Stop("native_collection_incomplete_no_adoption")
        with mock.patch.object(o,"native_collection",side_effect=interrupted):
            for _ in range(2):
                with self.assertRaisesRegex(o.Stop,"native_collection_incomplete_no_adoption"):e.host.call("RECOVER",{"id":e.pid},live=True,**e.host.auth())
                e.host.refresh_tail()
        self.assertNotEqual(outputs[0],outputs[1]);self.assertTrue(all((path/"incomplete.bin").exists() and not (path/"manifest.json").exists() for path in outputs))
        records=e.records();self.assertEqual([record["kind"] for record in records[-2:]],["ATTEMPT","ATTEMPT"])
        self.assertEqual(next(record["data"] for record in records if record["kind"]=="INTENT"),intent)
        inspected=e.host.call("INSPECT");self.assertEqual(inspected["completedPhases"],0)
        self.assertEqual(inspected["proposedFeeLedger"]["reservedFeeLamports"],"10000");self.assertEqual(inspected["proposedFeeLedger"]["actualFeeLamports"],"0")
        with self.assertRaisesRegex(o.Stop,"intent_exists_recovery_only"):e.host.call("SEND",{"id":e.pid},live=True,**e.host.auth())

    def test_pre_and_post_vote_timeouts_keep_only_attempts_and_injected_gate_mode(self):
        records=[];operator=SimpleNamespace(path=self.root/"operator")
        state={"seal":{"parentPath":str(self.root/"sealed-parent"),"parentTail":{"sequence":"5","digest":"a"*64},"sealDataSha256":"b"*64},"floor":100,"cursor":0,"manifest":{"phases":[{"id":"vote"}]}}
        def store(c,j,op,stage,current,transport):
            records.append({"kind":"ATTEMPT","stage":stage});return c.Store(self.root/f"attempt-{len(records)}")
        with mock.patch.object(o,"evidence_store",side_effect=store):
            for stage in ("PRE_VOTE","POST_VOTE"):
                with collector_child(drip=True):
                    with self.assertRaisesRegex(o.Stop,"native_collection_incomplete_no_adoption"):o.gate(self.r,self.c,self.j,operator,state,stage,None)
            injected=mock.Mock(return_value={"status":"VERIFIED_COLLECTED_STORAGE_AND_CLOCK_EVIDENCE_ONLY","sealDataSha256":"b"*64,"snapshotContextSlot":"101","snapshotSha256":"c"*64,"manifestReceipt":{"sha256":"d"*64},"transportMode":"OFFLINE_INJECTED_TEST"})
            with mock.patch.object(self.r,"collect",injected),mock.patch.object(o,"native_collection",side_effect=AssertionError("injected mode must remain separate")):
                result=o.gate(self.r,self.c,self.j,operator,state,"PRE_VOTE",lambda raw:None)
            self.assertEqual(result["transportMode"],"OFFLINE_INJECTED_TEST")
        self.assertTrue(all(record["kind"]=="ATTEMPT" for record in records));self.assertEqual(state["cursor"],0)


if __name__=="__main__":unittest.main()
