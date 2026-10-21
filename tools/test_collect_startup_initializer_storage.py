"""Offline source-pinned composition, exact RPC receipts and numeric invariants."""
import base64
import copy
import hashlib
import http.client
import importlib.util
import io
import json
import os
from pathlib import Path
import struct
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

HERE = Path(__file__).parent
REPO = Path(os.environ.get("PIV1_PUBLIC_CHECK_REPO", HERE.parent))

def load(name, path):
    spec=importlib.util.spec_from_file_location(name,path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

r=load("composed_readback",HERE/"collect_startup_initializer_storage.py")
HELPER_RAW=(REPO/"tools/test_startup_initializer_storage.py").read_bytes()
assert hashlib.sha256(HELPER_RAW).hexdigest()=="be8924a65d108dd69b5d7ddefff623bed21eb7c7aea9e6bb7e3d9fbfac32b493"
h=load("independent_raw_serializers",REPO/"tools/test_startup_initializer_storage.py")


def clock(now=1234567890, slot=9007199254741001):
    return {"exists":True,"owner":r.SYSVAR,"executable":False,"lamports":"1",
        "dataBase64":base64.b64encode(struct.pack("<QqQQq",slot,-44,r.U64,r.U64,now)).decode()}


class Replies:
    def __init__(self, fixture, stage="PRE_VOTE", mutate=None, fail_at=None, failure_type=OSError, status=200, output=None):
        self.fixture=fixture;self.stage=stage;self.mutate=mutate;self.fail_at=fail_at;self.failure_type=failure_type;self.status=status;self.output=output;self.calls=[]
    def __call__(self, raw):
        questions=json.loads(raw);assert len(questions)==1
        q=questions[0];self.calls.append(q)
        if self.output:
            prefix=f"http-{len(self.calls):02}"
            assert (self.output/(prefix+".request.json")).read_bytes()==raw
            assert (self.output/(prefix+".started.json")).exists()
        if self.fail_at==len(self.calls):raise self.failure_type("offline_partial_failure",b'{"partial":')
        if q["method"]=="getGenesisHash":
            assert q["params"]==[];value="4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY"
        else:
            assert q["method"]=="getMultipleAccounts"
            f=self.fixture;plan=f.plan;keys=[plan[k] for k in ("multisig","transaction","proposal")]+[r.CLOCK]
            assert q["params"][0]==keys
            assert q["params"][1]=={"encoding":"base64","commitment":"finalized","minContextSlot":int(f.seal["clock"]["contextSlot"])}
            normalized=h.snapshot(f.seal,f.pdas,self.stage)["accounts"]
            now=1234567890+(3600 if self.stage=="POST_VOTE" else 0)
            normalized[r.CLOCK]=clock(now)
            values=[{"owner":normalized[k]["owner"],"executable":normalized[k]["executable"],"lamports":int(normalized[k]["lamports"]),"data":[normalized[k]["dataBase64"],"base64"]} for k in keys]
            value={"context":{"slot":int(f.seal["clock"]["contextSlot"])},"value":values,"unusedUiAmount":0.5}
        response={"jsonrpc":"2.0","id":q["id"],"result":value}
        if self.mutate:self.mutate(q["method"],response)
        return self.status,[("content-type","application/json")],json.dumps([response]).encode()


class GovernanceReadbackTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        # Only builders/setup are reused; no previous test method or suite is run.
        h.InitializerStorageTests.setUpClass();cls.fixture=h.InitializerStorageTests
        cls.c,cls.s,cls.j=r.helpers()
    @classmethod
    def tearDownClass(cls):h.InitializerStorageTests.tearDownClass()
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.output=Path(self.temp.name)/"evidence"
        self.before={p.name:p.read_bytes() for p in self.fixture.path.iterdir()}
    def value(self,stage="PRE_VOTE"):
        return {"schema":r.SCHEMA,"expectedTail":copy.deepcopy(self.fixture.tail),"stage":stage,"minimumContextSlot":"0"}
    def run_collect(self,stage="PRE_VOTE",**options):
        reply=Replies(self.fixture,stage,output=self.output,**options)
        result=r.collect(self.value(stage),self.fixture.path,self.output,_transport=reply)
        self.assertEqual(self.before,{p.name:p.read_bytes() for p in self.fixture.path.iterdir()})
        manifest=json.loads((self.output/"manifest.json").read_bytes())
        self.assertEqual(manifest["counts"],result["counts"])
        for item in manifest["artifacts"]:
            path=self.output/item["name"];self.assertEqual(r.sha(path.read_bytes()),item["sha256"])
            self.assertEqual(path.stat().st_mode&0o777,0o600)
        self.assertEqual(self.output.stat().st_mode&0o777,0o700)
        self.assertTrue(all(result[k] is False for k in r.FLAGS))
        self.assertEqual(result["counts"]["liveTransportInvocationsStarted"],0)
        self.assertEqual(result["counts"]["injectedTransportInvocationsStarted"],len(reply.calls))
        return result,reply
    def reset_output(self):
        self.output=Path(self.temp.name)/("evidence-"+str(len(list(Path(self.temp.name).iterdir()))))

    def test_nullable_strict_request_and_no_import_journal_output_or_transport(self):
        with mock.patch.object(r,"helpers",side_effect=AssertionError("nullable helper/journal access")):
            self.assertEqual(r.validate(copy.deepcopy(r.TEMPLATE),self.fixture.path)["status"],"INCOMPLETE")
            self.assertEqual(r.collect(copy.deepcopy(r.TEMPLATE),self.fixture.path,self.output)["status"],"INCOMPLETE")
            for field in ("endpoint","accountKeys","snapshot","cumulativeFeeCapLamports"):
                value=self.value();value[field]="unreviewed"
                with self.assertRaises(r.Stop):r.collect(value,self.fixture.path,self.output)
        self.assertFalse(self.output.exists())
        nullable=Path(self.temp.name)/"nullable.json";nullable.write_bytes(r.encode(r.TEMPLATE))
        nonexistent=Path(self.temp.name)/"never-created-parent"
        for arguments in (["--validate",str(nullable),"--journal",str(nonexistent)],
            ["--collect",str(nullable),"--journal",str(nonexistent),"--output",str(self.output)]):
            captured=io.BytesIO()
            with mock.patch.object(r,"helpers",side_effect=AssertionError("nullable CLI helper import")):
                with mock.patch.object(r.sys,"stdout",SimpleNamespace(buffer=captured)):status=r.main(arguments)
            self.assertEqual(status,0);self.assertEqual(json.loads(captured.getvalue())["status"],"INCOMPLETE")
            self.assertFalse(nonexistent.exists());self.assertFalse(self.output.exists())
        for raw in (b'{"stage":null,"stage":null}',b'{"x":NaN}',b'{"x":1.5}',b'['*33+b'0'+b']'*33,b' '*65537):
            nullable.write_bytes(raw)
            with self.assertRaises(r.Stop):r.read_input(nullable)
        for field,bad in (("minimumContextSlot",True),("minimumContextSlot","01"),("minimumContextSlot",str(r.U64+1)),("stage","EXECUTE")):
            value=self.value();value[field]=bad
            with self.assertRaises(r.Stop):r.request_shape(value)

    def test_actual_pinned_pre_and_post_composition_receipts_context_and_unchanged_ledger(self):
        for stage in ("PRE_VOTE","POST_VOTE"):
            result,reply=self.run_collect(stage)
            self.assertEqual(result["status"],"VERIFIED_COLLECTED_STORAGE_AND_CLOCK_EVIDENCE_ONLY")
            self.assertEqual([q["method"] for q in reply.calls],["getGenesisHash","getMultipleAccounts"])
            self.assertEqual(result["nodeProcessInvocations"],5)
            self.assertEqual(result["storageReport"]["nodeProcessInvocations"],3)
            self.assertFalse(result["storageReport"]["timelockReleased"])
            self.assertEqual(result["storageReport"]["evidence"]["transaction"]["compactSha256"],self.fixture.seal["compactSha256"])
            self.assertEqual(result["inheritedLedgerReference"],{"actualFeeLamports":"5000","reservedFeeLamports":"5000","cumulativeFeeCapLamports":"25000"})
            self.assertEqual(result["transportMode"],"OFFLINE_INJECTED_TEST");self.assertFalse(result["fixedOriginTlsResponseReceived"])
            timing=result["clockEvidence"]
            self.assertEqual(timing["epochStartTimestamp"],"-44");self.assertEqual(timing["epoch"],str(r.U64))
            if stage=="POST_VOTE":self.assertTrue(timing["timelockArithmeticSatisfiedByCollectedClock"]);self.assertEqual(timing["elapsedSeconds"],"3600")
            else:self.assertIsNone(timing["timelockArithmeticSatisfiedByCollectedClock"])
            for index in (1,2):
                http=json.loads((self.output/f"http-{index:02}.http.json").read_bytes())
                started=json.loads((self.output/f"http-{index:02}.started.json").read_bytes())
                self.assertIsInstance(http["responseCompletedUtc"],str);self.assertIsInstance(started["requestStartedUtc"],str)
            self.reset_output()

    def test_clock_all_lengths_identity_context_and_full_u64_i64_domains(self):
        storage={"evidence":{"proposal":{"timestamp":"0"}}}
        for size in range(81):
            if size==40:continue
            value=clock();value["dataBase64"]=base64.b64encode((bytes(80))[:size]).decode()
            with self.assertRaises(r.Stop):r.clock_evidence(value,9007199254741001,0,"PRE_VOTE",storage,0)
        for change in ({"exists":False},{"owner":r.CLOCK},{"executable":True},{"lamports":"0"},{"dataBase64":"AB=="}):
            value=clock();value.update(change)
            with self.assertRaises(r.Stop):r.clock_evidence(value,9007199254741001,0,"PRE_VOTE",storage,0)
        for now in (r.I64_MIN,-1,0,1,r.I64_MAX):
            evidence=r.clock_evidence(clock(now,r.U64),r.U64,r.U64,"PRE_VOTE",storage,0)
            self.assertEqual(evidence["unixTimestamp"],str(now));self.assertEqual(evidence["slot"],str(r.U64))
        with self.assertRaisesRegex(r.Stop,"clock_context_or_slot_stale"):r.clock_evidence(clock(slot=10),11,11,"PRE_VOTE",storage,0)
        with self.assertRaisesRegex(r.Stop,"clock_context_or_slot_stale"):r.clock_evidence(clock(slot=12),11,0,"PRE_VOTE",storage,0)

    def test_native_checked_signed_subtraction_threshold_monotonicity_and_overflow(self):
        # Native contract oracle: delay is an inclusive threshold; i64 subtraction
        # must be representable even though Python integers do not overflow.
        for stamp in (r.I64_MIN,-1,0,1,r.I64_MAX-7776001):
            for delay in (0,1,3600,7776000):
                for delta in (-1,0,delay-1,delay,delay+1):
                    now=stamp+delta
                    if not r.I64_MIN<=now<=r.I64_MAX:continue
                    storage={"evidence":{"proposal":{"timestamp":str(stamp)}}}
                    if delta<delay:
                        with self.assertRaisesRegex(r.Stop,"timelock_not_satisfied"):r.clock_evidence(clock(now),9007199254741001,0,"POST_VOTE",storage,delay)
                    else:
                        result=r.clock_evidence(clock(now),9007199254741001,0,"POST_VOTE",storage,delay)
                        self.assertEqual(result["elapsedSeconds"],str(delta));self.assertTrue(result["timelockArithmeticSatisfiedByCollectedClock"])
        for now,stamp in ((r.I64_MIN,r.I64_MAX),(r.I64_MAX,r.I64_MIN)):
            with self.assertRaisesRegex(r.Stop,"timelock_i64_subtraction_overflow"):
                r.clock_evidence(clock(now),9007199254741001,0,"POST_VOTE",{"evidence":{"proposal":{"timestamp":str(stamp)}}},0)

    def test_strict_rpc_envelopes_duplicate_ids_nonfinite_and_unused_float(self):
        valid={"jsonrpc":"2.0","id":2,"result":{"unusedUiAmount":0.5}}
        self.assertIn("unusedUiAmount",r.rpc_result(self.c,json.dumps([valid]).encode(),2))
        for mutate in (lambda x:x.update(id=True),lambda x:x.update(id=1),lambda x:x.update(id="2"),lambda x:x.update(jsonrpc="1.0"),lambda x:x.update(error={"code":-1})):
            value=copy.deepcopy(valid);mutate(value)
            with self.assertRaises(r.Stop):r.rpc_result(self.c,json.dumps([value]).encode(),2)
        for raw in (b'[]',json.dumps([valid,valid]).encode(),b'[{"jsonrpc":"2.0","id":2,"id":2,"result":null}]',b'[{"jsonrpc":"2.0","id":2,"result":NaN}]'):
            with self.assertRaises((r.Stop,self.c.Stop)):r.rpc_result(self.c,raw,2)
        for bad in (True,-1,1.5,r.U64+1):
            with self.assertRaises(self.c.Stop):self.c.u64(bad)

    def test_wrong_genesis_redirect_and_rpc_error_keep_raw_one_read_no_retry(self):
        variants=[({"mutate":lambda m,x:x.update(result="wrong")},"genesis_mismatch"),({"status":302},"redirect_forbidden"),
            ({"mutate":lambda m,x:(x.pop("result"),x.update(error={"code":-32000}))},"rpc_error")]
        for options,reason in variants:
            result,reply=self.run_collect(**options)
            self.assertEqual(result["status"],"STOP");self.assertEqual(result["reason"],reason);self.assertEqual(len(reply.calls),1)
            self.assertTrue((self.output/"http-01.response.bin").exists());self.reset_output()

    def test_partial_transport_failure_preserves_bytes_counters_and_no_later_read(self):
        reply=Replies(self.fixture,fail_at=2,failure_type=self.c.TransportFailure,output=self.output)
        # Reuse actual verified module objects so exception class identity matches.
        with mock.patch.object(r,"helpers",return_value=(self.c,self.s,self.j)):
            result=r.collect(self.value(),self.fixture.path,self.output,_transport=reply)
        self.assertEqual(result["status"],"STOP");self.assertEqual(result["counts"]["httpAttemptsStarted"],2)
        self.assertEqual(result["counts"]["httpResponsesCompleted"],1);self.assertEqual(result["counts"]["httpTransportFailures"],1)
        self.assertEqual((self.output/"http-02.partial-response.bin").read_bytes(),b'{"partial":')
        self.assertIn("transportFailedUtc",json.loads((self.output/"http-02.failure.json").read_bytes()))
        self.assertEqual(result["counts"]["liveTransportInvocationsStarted"],0)

    def test_missing_account_stale_context_and_stale_clock_are_stops(self):
        def missing(method,response):
            if method=="getMultipleAccounts":response["result"]["value"][1]=None
        def stale_context(method,response):
            if method=="getMultipleAccounts":response["result"]["context"]["slot"]-=1
        def stale_clock(method,response):
            if method=="getMultipleAccounts":response["result"]["value"][3]["data"][0]=clock(slot=9007199254741000)["dataBase64"]
        for mutate,reason in ((missing,"account_identity"),(stale_context,"accounts_context_stale"),(stale_clock,"clock_context_or_slot_stale")):
            result,reply=self.run_collect(mutate=mutate)
            self.assertEqual(result["status"],"STOP");self.assertEqual(result["reason"],reason);self.assertEqual(len(reply.calls),2);self.reset_output()

    def test_collected_native_corruption_stage_votes_and_delay_cannot_be_rebound(self):
        def corrupt(method,response):
            if method=="getMultipleAccounts":
                data=response["result"]["value"][1]["data"];raw=bytearray(base64.b64decode(data[0]));raw[-5]^=1;data[0]=base64.b64encode(raw).decode()
        def too_early(method,response):
            if method=="getMultipleAccounts":response["result"]["value"][3]["data"][0]=clock(now=1234571489)["dataBase64"]
        for stage,mutate,reason in (("PRE_VOTE",corrupt,"stored_native_mismatch"),("POST_VOTE",too_early,"timelock_not_satisfied")):
            result,_=self.run_collect(stage,mutate=mutate)
            self.assertEqual(result["status"],"STOP");self.assertEqual(result["reason"],reason);self.reset_output()
        # An Approved raw snapshot cannot be relabeled PRE_VOTE by the caller.
        reply=Replies(self.fixture,"POST_VOTE",output=self.output)
        result=r.collect(self.value("PRE_VOTE"),self.fixture.path,self.output,_transport=reply)
        self.assertEqual(result["reason"],"proposal_stage_status")
        self.assertEqual(self.before,{p.name:p.read_bytes() for p in self.fixture.path.iterdir()})

    def test_output_exclusive_outside_parent_symlinks_and_reserve_before_http(self):
        reply=Replies(self.fixture)
        for path in (self.fixture.path,self.fixture.path/"forbidden-child"):
            with self.assertRaisesRegex(r.Stop,"output_inside_parent_forbidden"):r.collect(self.value(),self.fixture.path,path,_transport=reply)
        self.output.mkdir()
        with self.assertRaises(FileExistsError):r.collect(self.value(),self.fixture.path,self.output,_transport=reply)
        link=Path(self.temp.name)/"link";link.symlink_to(self.output,target_is_directory=True)
        with self.assertRaises(FileExistsError):r.collect(self.value(),self.fixture.path,link,_transport=reply)
        fresh=Path(self.temp.name)/"no-space"
        with mock.patch.object(r.os,"statvfs",return_value=SimpleNamespace(f_bavail=1,f_frsize=1)):
            with self.assertRaisesRegex(r.Stop,"reserve_2gib_evidence_budget"):r.collect(self.value(),self.fixture.path,fresh,_transport=reply)
        self.assertFalse(fresh.exists());self.assertEqual(reply.calls,[])

    def test_failed_started_persistence_and_uncertain_manifest_never_return_success(self):
        real=self.c.Store.write;reply=Replies(self.fixture,output=self.output)
        def started_fail(store,name,raw):
            if name.endswith(".started.json"):raise OSError("offline started fsync failure")
            return real(store,name,raw)
        with mock.patch.object(r,"helpers",return_value=(self.c,self.s,self.j)):
            with mock.patch.object(self.c.Store,"write",started_fail):result=r.collect(self.value(),self.fixture.path,self.output,_transport=reply)
        self.assertEqual(result["status"],"STOP");self.assertEqual(result["counts"]["httpAttemptsStarted"],0);self.assertEqual(reply.calls,[])
        self.reset_output();reply=Replies(self.fixture,mutate=lambda m,x:x.update(result="wrong"),output=self.output)
        def manifest_fail(store,name,raw):
            receipt=real(store,name,raw)
            if name=="manifest.json":raise OSError("offline uncertain manifest fsync acknowledgement")
            return receipt
        with mock.patch.object(r,"helpers",return_value=(self.c,self.s,self.j)):
            with mock.patch.object(self.c.Store,"write",manifest_fail):
                with self.assertRaises(OSError):r.collect(self.value(),self.fixture.path,self.output,_transport=reply)
        self.assertTrue((self.output/"manifest.json").exists());self.assertEqual(len(reply.calls),1)

    def test_source_pin_real_profile_guard_stale_tail_and_falsey_injected_hook(self):
        with mock.patch.dict(r.PINS,{"tools/startup_public_journal.py":"0"*64}):
            with mock.patch.object(r.importlib.util,"spec_from_file_location",side_effect=AssertionError("unpinned import")):
                with self.assertRaisesRegex(r.Stop,"helper_source_pin"):r.helpers()
        with mock.patch.object(http.client,"HTTPSConnection",side_effect=AssertionError("network forbidden")):
            result=r.validate(self.value(),self.fixture.path);self.assertFalse(result["tlsCollectionProfileAllowed"])
            with self.assertRaisesRegex(r.Stop,"actual_testnet_profile_required_before_collection"):r.collect(self.value(),self.fixture.path,self.output)
            with self.assertRaisesRegex(r.Stop,"offline_transport_callable"):r.collect(self.value(),self.fixture.path,self.output,_transport=False)
            self.assertFalse(self.output.exists())
            reply=Replies(self.fixture,output=self.output)
            class Falsey:
                def __bool__(self):return False
                def __call__(self,raw):return reply(raw)
            accepted=r.collect(self.value(),self.fixture.path,self.output,_transport=Falsey())
            self.assertEqual(accepted["status"],"VERIFIED_COLLECTED_STORAGE_AND_CLOCK_EVIDENCE_ONLY")
            self.assertEqual(accepted["counts"]["liveTransportInvocationsStarted"],0)
        self.reset_output();bad=self.value();bad["expectedTail"]["sequence"]="4"
        with self.assertRaisesRegex(self.j.Stop,"tail_compare_and_swap"):
            with mock.patch.object(r,"helpers",return_value=(self.c,self.s,self.j)):r.collect(bad,self.fixture.path,self.output,_transport=Replies(self.fixture))
        self.assertFalse(self.output.exists());self.assertEqual(self.before,{p.name:p.read_bytes() for p in self.fixture.path.iterdir()})


if __name__=="__main__":unittest.main()
