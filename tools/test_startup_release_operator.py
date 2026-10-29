"""Focused offline operator tests. Existing historical signatures only.

Injected transport is synthetic retrospective response replay, never RPC.
No private key, key creation, signing, simulation or old-suite execution.
"""
import base64
import ast
from contextlib import ExitStack
from contextlib import contextmanager
import copy
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import types
import unittest
from unittest import mock

HERE = Path(__file__).parent
REPO = Path(os.environ.get("PIV1_PUBLIC_CHECK_REPO", "/home/jerem/piv1"))
spec = importlib.util.spec_from_file_location("release_operator", HERE / "startup_release_operator.py")
o = importlib.util.module_from_spec(spec); spec.loader.exec_module(o)
FIXTURE = json.loads((REPO / "clients/cli/startup-public-check.fixture.json").read_bytes())
assert o.sha((REPO / "clients/cli/startup-public-check.fixture.json").read_bytes()) == "0855cd85667a565f4b6184ef2b389c2e4ec4421582f2b52f58b2cf8897852c0d"
CLOCK_RAW=(REPO / "tools/startup_clock_binding.fixture.json").read_bytes()
assert o.sha(CLOCK_RAW)=="1e627e75208408177b6d07042bdfe307d791b0d0502b7c48961b56630c64670b"
CLOCK_FIXTURE=json.loads(CLOCK_RAW)


def phase_from_fixture(c, index):
    t = FIXTURE["transactions"][index]; wire = base64.b64decode(t["signedWireBase64"])
    msg = wire[1+64*t["signatureCount"]:]; n=msg[3]
    keys=[c.b58encode(msg[4+i*32:36+i*32]) for i in range(n)]
    roles=[{"address":key,"signer":i<msg[0],"writable":i<msg[0]-msg[1] if i<msg[0] else i<n-msg[2]} for i,key in enumerate(keys)]
    reviewed={"messageBase64":base64.b64encode(msg).decode(),"messageSha256":t["messageSha256"],"feePayer":keys[0],"requiredSigners":keys[:msg[0]],"accountRoles":roles,"lookupTables":[]}
    life={"recentBlockhash":c.b58encode(msg[4+32*n:36+32*n]),"currentBlockHeight":"100","lastValidBlockHeight":"1000","minimumRemainingBlockHeights":"40"}
    unsigned=bytes([t["signatureCount"]])+bytes(64*t["signatureCount"])+msg
    return reviewed, life, base64.b64encode(unsigned).decode()


class OperatorTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.r,cls.c,cls.s,cls.j=o.helpers()

    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.base=Path(self.tmp.name)/"base"; self.tail=None; self.calls=[]
        c,j=self.c,self.j
        reviewed,life,unsigned=phase_from_fixture(c,0); t=FIXTURE["transactions"][0]
        phase={"id":"historical_fund","packet":{"signedWireBase64":t["signedWireBase64"],"reviewed":reviewed},"lifetime":life,"feeQuoteLamports":t["feeLamports"],"feeCapLamports":t["feeLamports"],"expectedPoststate":FIXTURE["fundPoststate"]["accounts"]}
        frozen={"packageSha256":"a"*64,"payloadSha256":"b"*64,"anchorUnixTimestamp":None,"transactionIndex":CLOCK_FIXTURE["preManifest"]["initializer"]["governance"]["transactionIndex"]}
        segment={"id":"historical_base","input":{"schema":"piv1-startup-public-check-v1","frozen":frozen,"cumulativeFeeCapLamports":"25000","phases":[phase],"trace":[]}}
        fd=j.directory(self.base,True)
        try:
            j.write_file(fd,"lock",j.LOCK_BYTES); meta=os.stat("lock",dir_fd=fd)
            identity={"device":str(meta.st_dev),"inode":str(meta.st_ino),"ctimeNs":str(meta.st_ctime_ns)}
            chain=[]; record=j.make_record(chain,"BEGIN",segment,identity); j.write_file(fd,"00000000.json",j.encode(record)); chain.append(record)
            previous=None
            for name in ("PREPARED","SUBMISSION_START_RECORDED","FINALIZED"):
                obs=None
                if name=="FINALIZED":
                    obs={"status":{"signatureHex":base64.b64decode(t["signedWireBase64"])[1:65].hex(),"slot":t["slot"],"confirmationStatus":"finalized","err":None},"transaction":{"slot":t["slot"],"wireBase64":t["signedWireBase64"],"feeLamports":t["feeLamports"],"err":None},"poststate":{"contextSlot":t["slot"],"accounts":copy.deepcopy(FIXTURE["fundPoststate"]["accounts"])}}
                event={"previousDigest":o.sha(j.encode(previous).rstrip(b"\n")) if previous else None,"phase":phase["id"],"state":name,"packetSha256":t["signedWireSha256"],"signatureHex":base64.b64decode(t["signedWireBase64"])[1:65].hex(),"frozen":frozen,"observation":obs}
                record=j.make_record(chain,"EVENT",event,identity); j.write_file(fd,f"{len(chain):08}.json",j.encode(record)); chain.append(record); previous=event
            parent_tail=j.tail(chain)
        finally: os.close(fd)
        self.base_bytes={p.name:p.read_bytes() for p in self.base.iterdir()}
        reviewed,life,unsigned=phase_from_fixture(c,1); self.tx=FIXTURE["transactions"][1]
        states={key:o.state_digest(c,a) for key,a in FIXTURE["fundPoststate"]["accounts"].items()}
        self.phase={"id":"synthetic_replay_only","kind":"BUFFER_CREATE","initializerIndex":None,"reviewed":reviewed,"feeCapLamports":self.tx["feeLamports"],"minimumRemainingBlockHeights":"40","expectedPrestate":copy.deepcopy(states),"expectedPoststate":copy.deepcopy(states)}
        historical={"cumulativeFeeCapLamports":"25000","reservedFeeLamports":"5000","actualFeeLamports":"5000"}
        proposal={"namespace":"piv1-testnet-startup-release-"+"c"*64,"status":"PROPOSAL_ONLY_NOT_AUTHORIZED","feeCapLamports":"4815000","historicalLedgerReference":{"parentTail":parent_tail,"ledger":copy.deepcopy(historical)}}
        self.manifest={"schema":o.SCHEMA,"packageSha256":"c"*64,"artifact":o.ARTIFACT,"sourcePins":{"readback":o.READBACK_SHA,"checker":j.CHECKER_SHA,"sdk":j.SDK_SHA,"journal":"bfea079a5740fa66e437930d50793a18289beeae1760af4164e986295c8a6d37"},"initializerPreManifestSha256":"d"*64,"parentTail":parent_tail,"historicalLedger":historical,"proposedFeeBudget":proposal,"phases":[self.phase]}
        self.prepared={"id":self.phase["id"],"wireBase64":unsigned,"lifetime":life,"feeQuoteLamports":self.tx["feeLamports"]}

    def call(self,action,data=None,**kw):
        result=o.operate({"schema":o.SCHEMA,"action":action,"expectedTail":self.tail,"data":data},self.base,**kw)
        self.tail=result["tail"]; return result

    def create(self): return self.call("CREATE",self.manifest)

    def ready(self):
        self.create(); self.call("PREPARE",[self.prepared]); self.call("IMPORT",[{"id":self.phase["id"],"wireBase64":self.tx["signedWireBase64"]}])

    def refresh_tail(self):
        records=sorted(o.journal_path(self.base).glob("*.json"))
        self.tail=json.loads(records[-1].read_bytes())["digest"]

    def auth(self):
        raw=o.encode({"schema":"piv1-launcher-release-authorization-v1","manifestSha256":o.sha(o.encode(self.manifest)),"baseParentTail":self.manifest["parentTail"],"operatorSourceSha256":o.sha((HERE/"startup_release_operator.py").read_bytes()),"expiresUnixTimestamp":"18446744073709551615","scope":"EXACT_BOUNDED_TESTNET_RELEASE","feeBudgetNamespace":self.manifest["proposedFeeBudget"]["namespace"]})
        return {"authorization_raw":raw,"authorization_sha256":o.sha(raw)}

    def wire_account(self,a):
        if not a["exists"]: return None
        return {"owner":a["owner"],"executable":a["executable"],"lamports":int(a["lamports"]),"data":[a["dataBase64"],"base64"],"rentEpoch":0,"space":len(base64.b64decode(a["dataBase64"]))}

    def transport(self,raw,*,timeout=False,unknown=False,bad_post=False):
        queries=json.loads(raw); replies=[]
        for q in queries:
            method=q["method"]; self.calls.append(method)
            slot=int(self.tx["slot"]); ctx=slot+5
            if method=="getGenesisHash": value=self.c.GENESIS
            elif method=="getBlockHeight": value=200
            elif method=="getLatestBlockhash": value={"context":{"slot":ctx},"value":{"blockhash":self.prepared["lifetime"]["recentBlockhash"],"lastValidBlockHeight":1000}}
            elif method=="isBlockhashValid": value={"context":{"slot":ctx},"value":True}
            elif method=="getFeeForMessage": value={"context":{"slot":ctx},"value":int(self.tx["feeLamports"])}
            elif method=="sendTransaction":
                # Observe the actual durable record before the transport can receive a send.
                chain=[json.loads(p.read_bytes()) for p in sorted(o.journal_path(self.base).glob("*.json"))]
                self.assertEqual(chain[-1]["kind"],"INTENT")
                self.assertEqual(q["params"][1]["maxRetries"],0)
                self.assertFalse(q["params"][1]["skipPreflight"])
                if timeout: raise self.c.TransportFailure("injected response loss", b'{"partial":')
                value=self.tx["signature"]
            elif method=="getSignatureStatuses": value={"context":{"slot":ctx},"value":[None if unknown else {"slot":slot,"confirmations":None,"err":None,"confirmationStatus":"finalized"}]}
            elif method=="getTransaction": value=None if unknown else {"slot":slot,"version":"legacy","transaction":[self.tx["signedWireBase64"],"base64"],"meta":{"fee":int(self.tx["feeLamports"]),"err":None}}
            elif method=="getMultipleAccounts":
                a=copy.deepcopy(FIXTURE["fundPoststate"]["accounts"])
                if bad_post: a[next(iter(a))]["lamports"]="1"
                value={"context":{"slot":ctx},"value":[self.wire_account(a[key]) for key in q["params"][0]]}
            else: raise AssertionError(method)
            replies.append({"jsonrpc":"2.0","id":q["id"],"result":value})
        return 200,[],o.encode(replies)

    def test_nullable_imports_no_helpers(self):
        with mock.patch.object(o,"helpers",side_effect=AssertionError("unexpected helper")):
            self.assertEqual(o.operate({"schema":o.SCHEMA,"action":None,"expectedTail":None,"data":None},self.base)["status"],"INCOMPLETE")

    def test_create_inherits_ledger_cap_and_preserves_parent(self):
        parent=o.inspect_parent(self.j,self.base,self.manifest["parentTail"])
        bad=copy.deepcopy(self.manifest); bad["proposedFeeBudget"]["feeCapLamports"]="4815001"
        with self.assertRaisesRegex(o.Stop,"proposed_fee_namespace"): o.manifest_shape(self.c,self.j,bad,parent)
        bad=copy.deepcopy(self.manifest); bad["proposedFeeBudget"]["historicalLedgerReference"]["ledger"]["actualFeeLamports"]="0"
        with self.assertRaisesRegex(o.Stop,"proposed_fee_historical_reference"): o.manifest_shape(self.c,self.j,bad,parent)
        bad=copy.deepcopy(self.manifest); del bad["proposedFeeBudget"]
        with self.assertRaisesRegex(o.Stop,"object_fields"): o.manifest_shape(self.c,self.j,bad,parent)
        result=self.create(); self.assertEqual(result["historicalLedger"],self.manifest["historicalLedger"])
        self.assertEqual(result["historicalLedger"]["cumulativeFeeCapLamports"],"25000")
        self.assertEqual(result["proposedFeeLedger"]["actualFeeLamports"],"0")
        self.assertEqual(result["proposedFeeLedger"]["reservedFeeLamports"],"0")
        self.assertEqual(result["proposedFeeLedger"]["feeCapLamports"],"4815000")
        self.assertEqual({p.name:p.read_bytes() for p in self.base.iterdir()},self.base_bytes)
        self.assertEqual(self.call("INSPECT")["tail"],result["tail"])
        with self.assertRaisesRegex(o.Stop,"tail_compare_and_swap"):
            o.operate({"schema":o.SCHEMA,"action":"INSPECT","expectedTail":"0"*64,"data":None},self.base)

    def test_prepare_exact_unsigned_bytes_and_lifetime_refusals(self):
        self.create(); bad=copy.deepcopy(self.prepared); bad["lifetime"]["recentBlockhash"]="11111111111111111111111111111111"
        with self.assertRaisesRegex(o.Stop,"prepared_blockhash_binding"): self.call("PREPARE",[bad])
        bad=copy.deepcopy(self.prepared); bad["lifetime"]["lastValidBlockHeight"]="120"
        with self.assertRaisesRegex(o.Stop,"prepared_lifetime_margin"): self.call("PREPARE",[bad])
        self.call("PREPARE",[self.prepared])
        bad=copy.deepcopy(self.prepared); bad["lifetime"]["lastValidBlockHeight"]="1001"
        with self.assertRaisesRegex(o.Stop,"same_blockhash_lifetime_changed"): self.call("PREPARE",[bad])
        self.assertFalse(self.call("INSPECT")["liveReady"])

    def test_real_historical_signature_import_and_tamper_refusal(self):
        self.create(); self.call("PREPARE",[self.prepared]); wire=bytearray(base64.b64decode(self.tx["signedWireBase64"])); wire[1]^=1
        with self.assertRaisesRegex(o.Stop,"signature_invalid"):
            self.call("IMPORT",[{"id":self.phase["id"],"wireBase64":base64.b64encode(wire).decode()}])
        self.call("IMPORT",[{"id":self.phase["id"],"wireBase64":self.tx["signedWireBase64"]}])

    def test_missing_launcher_authorization_stops_before_transport(self):
        self.ready()
        with self.assertRaisesRegex(o.Stop,"launcher_authorization_required"):
            self.call("SEND",{"id":self.phase["id"]},_transport=self.transport)
        self.assertEqual(self.calls,[])
        with self.assertRaisesRegex(o.Stop,"launcher_authorization_required"):
            self.call("HANDOFF",[self.phase["id"]])

    def test_preintent_refresh_discards_old_signatures_and_freezes_semantics(self):
        self.ready(); refreshed=copy.deepcopy(self.prepared)
        raw=bytearray(base64.b64decode(refreshed["wireBase64"])); offset=1+raw[0]*64; n=raw[offset+3]; bh=offset+4+n*32
        raw[bh]^=1; refreshed["wireBase64"]=base64.b64encode(raw).decode(); refreshed["lifetime"]["recentBlockhash"]=self.c.b58encode(raw[bh:bh+32])
        self.call("PREPARE",[refreshed])
        with self.assertRaisesRegex(o.Stop,"current_signed_phase_required"):
            self.call("SEND",{"id":self.phase["id"]},_transport=self.transport,**self.auth())
        bad=copy.deepcopy(refreshed); raw[-1]^=1; bad["wireBase64"]=base64.b64encode(raw).decode()
        with self.assertRaisesRegex(o.Stop,"prepared_semantics_changed"):
            self.call("PREPARE",[bad])

    def test_send_intent_precedes_one_send_and_exact_finalization_advances(self):
        self.ready(); first=self.call("SEND",{"id":self.phase["id"]},_transport=self.transport,**self.auth())
        self.assertEqual(first["status"],"UNKNOWN_OR_SUBMITTED_RECOVERY_ONLY")
        result=self.call("RECOVER",{"id":self.phase["id"]},_transport=self.transport,**self.auth())
        self.assertEqual(result["status"],"ALL_PHASES_RECONCILED")
        self.assertEqual(result["proposedFeeLedger"]["actualFeeLamports"],self.tx["feeLamports"])
        self.assertEqual(result["historicalLedger"]["actualFeeLamports"],"5000")
        self.assertEqual(self.calls.count("sendTransaction"),1)
        self.assertEqual({p.name:p.read_bytes() for p in self.base.iterdir()},self.base_bytes)

    def test_uncertain_send_blocks_resend_handoff_import_and_allows_original_lookup(self):
        pinned_helpers=mock.patch.object(o,"helpers",return_value=(self.r,self.c,self.s,self.j))
        pinned_helpers.start(); self.addCleanup(pinned_helpers.stop)
        self.ready(); transport=lambda raw:self.transport(raw,timeout=True,unknown=True)
        result=self.call("SEND",{"id":self.phase["id"]},_transport=transport,**self.auth())
        self.assertEqual(result["status"],"UNKNOWN_OR_SUBMITTED_RECOVERY_ONLY")
        for action,data,reason in (("SEND",{"id":self.phase["id"]},"intent_exists_recovery_only"),("HANDOFF",[self.phase["id"]],"unresolved_intent_blocks_handoff"),("IMPORT",[{"id":self.phase["id"],"wireBase64":self.tx["signedWireBase64"]}],"unresolved_intent_blocks_import"),("PREPARE",[self.prepared],"unresolved_intent_blocks_prepare")):
            with self.assertRaisesRegex(o.Stop,reason): self.call(action,data,_transport=transport,**self.auth())
        self.call("RECOVER",{"id":self.phase["id"]},_transport=transport,**self.auth())
        self.assertEqual(self.calls.count("sendTransaction"),1)
        self.assertIn("getSignatureStatuses",self.calls)
        evidence=o.journal_path(self.base).with_name(o.journal_path(self.base).name+".evidence")
        self.assertEqual([p.read_bytes() for p in evidence.rglob("*.partial-response.bin")],[b'{"partial":'])
        # An original receipt can expire without preventing public resolution.
        original=self.auth()
        with mock.patch.object(o.time,"time",return_value=18446744073709551616):
            with self.assertRaisesRegex(o.Stop,"launcher_authorization_expired"):
                self.call("HANDOFF",[self.phase["id"]],**original)
            self.call("RECOVER",{"id":self.phase["id"]},_transport=transport,**original)
        different=json.loads(original["authorization_raw"]); different["expiresUnixTimestamp"]="18446744073709551614"; raw=o.encode(different)
        with self.assertRaisesRegex(o.Stop,"recovery_original_authorization_required"):
            self.call("RECOVER",{"id":self.phase["id"]},_transport=transport,authorization_raw=raw,authorization_sha256=o.sha(raw))

    def test_poststate_mismatch_never_advances_or_changes_expected_state(self):
        self.ready(); self.call("SEND",{"id":self.phase["id"]},_transport=self.transport,**self.auth())
        before=copy.deepcopy(self.manifest)
        with self.assertRaisesRegex(o.Stop,"poststate_mismatch"):
            self.call("RECOVER",{"id":self.phase["id"]},_transport=lambda raw:self.transport(raw,bad_post=True),**self.auth())
        self.refresh_tail()
        self.assertEqual(self.call("INSPECT")["completedPhases"],0)
        self.assertEqual(self.manifest,before)
        evidence=o.journal_path(self.base).with_name(o.journal_path(self.base).name+".evidence")
        retained={str(p):p.read_bytes() for p in evidence.rglob("*") if p.is_file()}
        with mock.patch.object(o,"helpers",return_value=(self.r,self.c,self.s,self.j)), mock.patch.object(self.c,"collect",side_effect=OSError("injected collection failure")):
            with self.assertRaisesRegex(OSError,"injected collection failure"):
                self.call("RECOVER",{"id":self.phase["id"]},_transport=self.transport,**self.auth())
        self.refresh_tail()
        result=self.call("RECOVER",{"id":self.phase["id"]},_transport=self.transport,**self.auth())
        self.assertEqual(result["status"],"ALL_PHASES_RECONCILED")
        self.assertTrue(all(Path(p).read_bytes()==raw for p,raw in retained.items()))
        self.assertEqual(len([p for p in evidence.iterdir() if p.name.endswith("-recover")]),3)

    def test_949_descriptor_capacity_and_small_batch_limit(self):
        self.manifest["phases"]=[{**copy.deepcopy(self.phase),"id":f"write_{i}"} for i in range(949)]
        result=self.create(); self.assertEqual(result["phaseCapacity"],949)
        with self.assertRaisesRegex(o.Stop,"batch_bound"): self.call("PREPARE",[self.prepared]*65)
        self.assertEqual(o.MAX_RECORDS,8192)
        # Capacity is not authority: historical cap and future proposal stay separate.
        self.assertEqual(result["historicalLedger"]["cumulativeFeeCapLamports"],"25000")
        self.assertEqual(result["proposedFeeLedger"]["reservedFeeLamports"],"0")

    def test_terminal_seal_required_and_initializer_causality_replay_guards(self):
        self.manifest["phases"]=[{**copy.deepcopy(self.phase),"id":f"init_{i}","kind":"INITIALIZER","initializerIndex":i,"reviewed":None} for i in range(9)]
        pre=copy.deepcopy(CLOCK_FIXTURE["preManifest"])
        self.manifest["initializerPreManifestSha256"]=o.sha(self.j.encode(pre))
        self.prepared["id"]="init_0"; self.create()
        with self.assertRaisesRegex(o.Stop,"initializer_seal_required"): self.call("PREPARE",[self.prepared])
        with self.assertRaisesRegex(o.Stop,"seal_profile_binding"):
            self.call("BIND_SEAL",{"parentPath":str(self.base),"parentTail":self.manifest["parentTail"]})
        fd=self.j.directory(o.journal_path(self.base))
        try: raw=self.j.read_file(fd,"00000000.json",self.j.MAX_INPUT)
        finally: os.close(fd)
        create=json.loads(raw)
        forged=[create,{"kind":"PRE_VOTE","data":{"id":"init_0"}}]
        with self.assertRaisesRegex(o.Stop,"gate_without_seal"): o.replay(self.j,forged)
        # The SAME base appends the expressly allowed PROFILE/SEAL boundary.
        def boundary(action,tail,value):
            return self.j.apply({"schema":self.j.SCHEMA,"action":action,"expectedTail":tail,"segment":value,"event":None},self.base)
        profile=boundary("FREEZE_INITIALIZER_PROFILE",self.manifest["parentTail"],pre)
        sealed=boundary("SEAL_INITIALIZER_CLOCK",profile["tail"],copy.deepcopy(CLOCK_FIXTURE["clock"]))
        self.assertTrue(all((self.base/name).read_bytes()==raw for name,raw in self.base_bytes.items()))
        added=[json.loads(p.read_bytes())["kind"] for p in sorted(self.base.glob("*.json")) if p.name not in self.base_bytes]
        self.assertEqual(added,["PROFILE","SEAL"])
        result=self.call("BIND_SEAL",{"parentPath":str(self.base),"parentTail":sealed["tail"]},_transport=self.transport)
        self.assertEqual(result["historicalLedger"],self.manifest["historicalLedger"])
        records=[json.loads(p.read_bytes()) for p in sorted(o.journal_path(self.base).glob("*.json"))]
        binding=records[-1]["data"]
        self.assertEqual(len(binding["reviewedPackets"]),9)
        self.assertEqual(binding["baseParentTail"],self.manifest["parentTail"])
        plan=sealed["initializerSeal"]["unsignedInitializerPlan"]
        self.prepared["wireBase64"]=plan["packets"][0]["wireBase64"]
        self.prepared["lifetime"]={"recentBlockhash":pre["initializer"]["transport"]["recentBlockhash"],"currentBlockHeight":"100","lastValidBlockHeight":pre["initializer"]["transport"]["lastValidBlockHeight"],"minimumRemainingBlockHeights":"40"}
        self.call("PREPARE",[self.prepared])
        self.assertEqual(self.call("INSPECT")["completedPhases"],0)

    def test_intent_write_uncertainty_falsey_transport_and_evidence_cap(self):
        pinned_helpers=mock.patch.object(o,"helpers",return_value=(self.r,self.c,self.s,self.j))
        pinned_helpers.start(); self.addCleanup(pinned_helpers.stop)
        self.ready()
        class FalseyTransport:
            def __bool__(self): return False
            def __call__(inner, raw): return self.transport(raw)
        original=self.j.write_file
        def uncertain(fd,name,raw):
            original(fd,name,raw)
            if name.endswith(".json") and json.loads(raw).get("kind")=="INTENT":
                raise OSError("injected fsync uncertainty after complete write")
        with mock.patch.object(self.j,"write_file",side_effect=uncertain), mock.patch.object(self.c,"tls_post",side_effect=AssertionError("actual TLS forbidden")):
            with self.assertRaisesRegex(o.Stop,"commit_uncertain_no_retry_or_adoption"):
                self.call("SEND",{"id":self.phase["id"]},_transport=FalseyTransport(),**self.auth())
        self.assertNotIn("sendTransaction",self.calls)
        self.refresh_tail()
        self.assertEqual(self.call("INSPECT")["status"],"UNKNOWN_OR_SUBMITTED_RECOVERY_ONLY")
        for action,data in (("SEND",{"id":self.phase["id"]}),("HANDOFF",[self.phase["id"]]),("IMPORT",[{"id":self.phase["id"],"wireBase64":self.tx["signedWireBase64"]}])):
            with self.assertRaises(o.Stop): self.call(action,data,_transport=FalseyTransport(),**self.auth())
        with mock.patch.object(o,"MAX_EVIDENCE",o.ACTION_EVIDENCE-1):
            with self.assertRaisesRegex(o.Stop,"evidence_total_bound"):
                self.call("RECOVER",{"id":self.phase["id"]},_transport=FalseyTransport(),**self.auth())
        self.assertEqual(self.call("RECOVER",{"id":self.phase["id"]},_transport=FalseyTransport(),**self.auth())["status"],"ALL_PHASES_RECONCILED")


if __name__=="__main__": unittest.main()


class CandidateOfflineRuntimeBridgeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.a=o.startup_manifest();ref=cls.a.candidate_oracle();_,cls.r,_=cls.a._candidate_helpers()
        cls.manifest=cls.a.candidate_assemble(Path(ref['provenance']['elfPublicPath']).read_bytes())
        cls.slot=int(ref['baselineContextSlot'])+1

    def test_import_only_bridge_uses_real_pinned_sources_and_no_legacy_parent(self):
        import struct
        accounts={self.r.PROGRAM_DATA:{'exists':False,'owner':None,'executable':None,'lamports':None,'dataBase64':None},
            self.r.CLOCK:{'exists':True,'owner':self.r.SYSVAR,'executable':False,'lamports':'1',
                'dataBase64':base64.b64encode(struct.pack('<QqQQq',self.slot,0,0,0,1793016001)).decode()}}
        result=o.candidate_verify_state_handoff(self.manifest,935,'PRE',accounts,context_slot=str(self.slot))
        self.assertTrue(result['runtimePredicateVerification']);self.assertFalse(result['controllerReady'])
        self.assertIsNone(result['runtimeFacts']['lastRuntimePhase'])
        self.assertFalse(o.full_startup(self.manifest))

    def test_legacy_operate_rejects_candidate_request_before_parent_or_journal(self):
        request={'schema':self.a.CANDIDATE_MANIFEST_SCHEMA,'action':'CREATE','expectedTail':None,'data':self.manifest}
        with self.assertRaisesRegex(ValueError,'schema'):o.operate(request,'/never-opened-candidate-runtime-journal')
        with tempfile.TemporaryDirectory() as temporary:
            base=Path(temporary)/'never-opened-legacy-parent'
            request={'schema':o.SCHEMA,'action':'INSPECT','expectedTail':None,'data':None}
            with self.assertRaisesRegex(ValueError,'helper_source_pin'):o.operate(request,base)
            self.assertFalse(base.exists());self.assertFalse(o.journal_path(base).exists())


class CandidatePreparationSealTests(unittest.TestCase):
    """New synthetic directories only; real ELF/planner/pins, no operational parent."""
    @classmethod
    def setUpClass(cls):
        cls.a=o.startup_manifest();cls.ref=cls.a.candidate_oracle();cls.j,_,_=cls.a._candidate_helpers()
        cls.manifest=cls.a.candidate_assemble(Path(cls.ref['provenance']['elfPublicPath']).read_bytes())
        cls.binding=cls.j.candidate_clock_binding(cls.ref['preManifest'],cls.ref['clock'])
        spec=importlib.util.spec_from_file_location('public_squads_raw_oracle',HERE/'test_startup_initializer_storage.py')
        cls.raw=importlib.util.module_from_spec(spec);spec.loader.exec_module(cls.raw)
        pdas=cls.raw.oracle_pdas(cls.ref['preManifest']['initializer']['governance'])
        cls.snapshot=cls.raw.snapshot(cls.binding,pdas,'PRE_VOTE')

    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
        self.path=Path(self.tmp.name)/'host.candidate-preparation'

    def create(self):return o.candidate_prepare_seal(self.path,self.manifest,self.binding)
    def hashes(self):return {p.name:o.sha(p.read_bytes())for p in self.path.iterdir()}

    def test_exact_durable_host_seal_readback_and_actual_raw_squads_bytes(self):
        created=self.create();before=self.hashes()
        read=o.candidate_read_preparation_seal(self.path,created['sealDigest'])
        result=o.candidate_preparation_storage_readback(self.path,created['sealDigest'],self.snapshot,'PRE_VOTE')
        self.assertEqual(created['seal'],read['seal']);self.assertEqual(before,self.hashes())
        self.assertEqual(result['storageEvidence']['transaction']['nativeSha256'],self.binding['nativeSha256'])
        self.assertEqual(result['initializerBindingSha256'],o.sha(o.encode(self.binding)))
        self.assertEqual(result['manifestSha256'],o.sha(o.encode(self.manifest)))
        self.assertEqual(result['snapshotSha256'],o.sha(o.encode(self.snapshot)))
        self.assertEqual(result['snapshotContextSlot'],self.snapshot['contextSlot'])
        self.assertEqual(result['seal']['historicalUncertaintyReference'],self.manifest['historicalUncertainty'])
        self.assertTrue(all(result[k]is False for k in o.CANDIDATE_PREPARATION_FLAGS))
        self.assertTrue(all(result['seal']['assurances'][k]is False for k in self.raw.s.FLAGS))
        self.assertEqual(set(self.path.iterdir()),{self.path/'lock',self.path/'seal.json'})
        self.assertNotIn('runtimeFacts',result['seal']);self.assertNotIn('parentTail',result['seal'])

    def test_duplicate_wrong_digest_partial_and_foreign_namespaces_are_never_repaired(self):
        created=self.create();before=self.hashes()
        with self.assertRaisesRegex(ValueError,'exists_no_retry_or_adoption'):self.create()
        with self.assertRaisesRegex(ValueError,'exact_digest'):o.candidate_read_preparation_seal(self.path,'0'*64)
        self.assertEqual(before,self.hashes())
        for name,lock in [('partial',o.CANDIDATE_PREPARATION_LOCK),('foreign',self.j.LOCK_BYTES)]:
            path=Path(self.tmp.name)/(name+'.candidate-preparation');fd=self.j.directory(path,True)
            try:self.j.write_file(fd,'lock',lock)
            finally:os.close(fd)
            with self.assertRaises(ValueError):o.candidate_read_preparation_seal(path,'0'*64)
            self.assertEqual(set(p.name for p in path.iterdir()),{'lock'})

    def test_extra_hardlink_and_record_flag_scope_field_tampering_refuse_without_mutation(self):
        created=self.create();fd=self.j.directory(self.path)
        try:self.j.write_file(fd,'extra.json',b'{}\n')
        finally:os.close(fd)
        before=self.hashes()
        with self.assertRaisesRegex(ValueError,'closed_namespace'):o.candidate_read_preparation_seal(self.path,created['sealDigest'])
        self.assertEqual(before,self.hashes())
        for name,change in [('flags',lambda r:r['assurances'].__setitem__('authorized',True)),('scope',lambda r:r.__setitem__('scope','EXACT_BOUNDED_TESTNET_RELEASE')),('fields',lambda r:r.__setitem__('runtimeFacts',{}))]:
            path=Path(self.tmp.name)/(name+'.candidate-preparation');o.candidate_prepare_seal(path,self.manifest,self.binding)
            record=json.loads((path/'seal.json').read_bytes());change(record);raw=o.encode(record);(path/'seal.json').write_bytes(raw)
            with self.assertRaises(ValueError):o.candidate_read_preparation_seal(path,o.sha(raw))
            self.assertEqual((path/'seal.json').read_bytes(),raw)
        path=Path(self.tmp.name)/'hardlink.candidate-preparation';r=o.candidate_prepare_seal(path,self.manifest,self.binding)
        os.link(path/'seal.json',Path(self.tmp.name)/'duplicate-public-seal.json')
        with self.assertRaises(ValueError):o.candidate_read_preparation_seal(path,r['sealDigest'])

    def test_manifest_binding_policy_and_no_runtime_or_completed_parent_input(self):
        bad=copy.deepcopy(self.manifest);bad['historicalUncertainty']['resolved']=True
        with self.assertRaises(ValueError):o.candidate_prepare_seal(self.path,bad,self.binding)
        self.assertFalse(self.path.exists())
        bad=copy.deepcopy(self.binding);bad['preManifest']['initializer']['recipients']['htfpRecipient']=self.ref['roles']['g0']
        with self.assertRaises(ValueError):o.candidate_prepare_seal(self.path,self.manifest,bad)
        self.assertFalse(self.path.exists())
        with self.assertRaises(TypeError):o.candidate_prepare_seal(self.path,self.manifest,self.binding,runtimeFacts={})
        self.assertFalse(self.path.exists())

    def test_exact_snapshot_set_native_payload_and_context_fail_read_only(self):
        created=self.create();before=self.hashes()
        for kind in ('extra','native','context'):
            snap=copy.deepcopy(self.snapshot)
            if kind=='extra':snap['accounts'][self.ref['roles']['g0']]={}
            elif kind=='context':snap['contextSlot']='0'
            else:
                address=self.binding['unsignedInitializerPlan']['transaction'];raw=bytearray(base64.b64decode(snap['accounts'][address]['dataBase64']));at=raw.index(b'PIV1IN02');raw[at:at+8]=b'PIV1IN01';snap['accounts'][address]['dataBase64']=base64.b64encode(raw).decode()
            with self.assertRaises(ValueError):o.candidate_preparation_storage_readback(self.path,created['sealDigest'],snap,'PRE_VOTE')
            self.assertEqual(before,self.hashes())

    def test_source_failure_after_durability_and_storage_never_reports_success(self):
        original=o._candidate_preparation_sources
        def fault_at(target):
            def sources():
                a,j,s,domain,check=original()
                def checked(boundary):
                    check(boundary)
                    if boundary==target:raise o.Stop('synthetic_source_drift:'+boundary)
                return a,j,s,domain,checked
            return sources
        with mock.patch.object(o,'_candidate_preparation_sources',side_effect=fault_at('after_durability')):
            with self.assertRaisesRegex(ValueError,'synthetic_source_drift'):self.create()
        before=self.hashes()
        with self.assertRaisesRegex(ValueError,'exists_no_retry_or_adoption'):self.create()
        self.assertEqual(before,self.hashes())
        path=Path(self.tmp.name)/'readback.candidate-preparation';created=o.candidate_prepare_seal(path,self.manifest,self.binding);before={p.name:o.sha(p.read_bytes())for p in path.iterdir()}
        with mock.patch.object(o,'_candidate_preparation_sources',side_effect=fault_at('after_storage')):
            with self.assertRaisesRegex(ValueError,'synthetic_source_drift'):o.candidate_preparation_storage_readback(path,created['sealDigest'],self.snapshot,'PRE_VOTE')
        self.assertEqual(before,{p.name:o.sha(p.read_bytes())for p in path.iterdir()})


class CandidateVerifyStatesIntegrationTests(unittest.TestCase):
    """Actual consuming boundary, public retained bytes and new synthetic seals only."""
    @classmethod
    def setUpClass(cls):
        cls.a=o.startup_manifest();cls.ref=cls.a.candidate_oracle();cls.j,_,cls.c=cls.a._candidate_helpers()
        cls.manifest=cls.a.candidate_assemble(Path(cls.ref['provenance']['elfPublicPath']).read_bytes())
        path=next(Path(p) for p in cls.ref['provenance']['evidence'] if p.endswith('/snapshot60.json'))
        raw=path.read_bytes();assert o.sha(raw)==cls.ref['provenance']['evidence'][str(path)]
        snapshot=json.loads(raw);cls.context=snapshot['contextSlot']
        cls.accounts={row['address']:{k:row[k] for k in ('exists','owner','executable','lamports','dataBase64')} for row in snapshot['accounts']}

    def verify(self,state=None,phase=None,index=0,**context):
        return o.verify_states(self.c,state or {'manifest':self.manifest},phase or self.manifest['phases'][index],
            'PRE',self.accounts,context_slot=self.context,**context)

    def test_real_verify_states_consumes_fixed_manifest_phase_and_complete_account_bytes(self):
        result=self.verify()
        self.assertTrue(result['accountBytesVerified']);self.assertEqual(len(result['checkedAccountKeys']),60)
        self.assertFalse(result['controllerReady']);self.assertFalse(result['completeStateOracle'])
        self.assertFalse(result['fundingObserved']);self.assertFalse(result['executionLineageAuthenticated'])
        self.assertEqual(result['accountDigests'],self.ref['baseline'])

    def test_final_manifest_source_check_refusal_occurs_after_real_verification(self):
        real=o.startup_manifest;calls=[]
        def checked():
            module=real();calls.append(module)
            if len(calls)==2:raise o.Stop('injected_final_manifest_source_refusal')
            return module
        with mock.patch.object(o,'startup_manifest',side_effect=checked):
            with self.assertRaisesRegex(ValueError,'injected_final_manifest_source_refusal'):self.verify()
        self.assertEqual(len(calls),2)

    def test_final_operator_and_jito_read_faults_refuse_without_repository_mutation(self):
        real=Path.read_bytes
        sources=[(Path(o.__file__).absolute(),'operator_source_changed')]
        sources.append((REPO/next(iter(self.a.JITO_SOURCE_PINS)),'jito_source_changed'))
        for target,reason in sources:
            reads=[]
            def changed(path):
                raw=real(path)
                if path.absolute()==target:
                    reads.append(raw)
                    if len(reads)==2:return raw+b'\n# injected read fault only\n'
                return raw
            with self.subTest(source=target.name),mock.patch.object(Path,'read_bytes',changed):
                with self.assertRaisesRegex(ValueError,reason):self.verify()
            self.assertEqual(len(reads),2)
            self.assertEqual(real(target),reads[0])

    def test_caller_masks_phase_types_floor_credit_and_legacy_facts_cannot_supply_policy(self):
        for change in (lambda p:p.update(expectedPrestate={}),lambda p:p.update(index=False),lambda p:p.update(index=0.0)):
            phase=copy.deepcopy(self.manifest['phases'][0]);change(phase)
            with self.assertRaisesRegex(ValueError,'candidate_state_phase'):self.verify(phase=phase)
        for name,value in (('runtimeFacts',{}),('floor','0'),('parentTail',{}),('seal',{})):
            with self.assertRaisesRegex(ValueError,'candidate_state_scope'):
                self.verify(state={'manifest':self.manifest,name:value})
        for name,value in (('funding_mode','CONDITIONAL_H2_CREDIT'),('floor','0'),('clock_binding',{})):
            with self.assertRaisesRegex(ValueError,'candidate_state_context'):self.verify(**{name:value})
        with self.assertRaisesRegex(ValueError,'prefix_no_runtime_or_binding'):
            self.verify(state={'manifest':self.manifest,'candidateRuntime':{}})

    def test_initializer_requires_exact_durable_binding_and_reaches_existing_runtime_order_guard(self):
        with self.assertRaisesRegex(ValueError,'preparation_required'):self.verify(index=937)
        binding=self.j.candidate_clock_binding(self.ref['preManifest'],self.ref['clock'])
        with tempfile.TemporaryDirectory() as temporary:
            path=Path(temporary)/'state.candidate-preparation'
            created=o.candidate_prepare_seal(path,self.manifest,binding)
            before={p.name:o.sha(p.read_bytes()) for p in path.iterdir()}
            state={'manifest':self.manifest,'candidatePreparation':{'path':str(path),'sha256':created['sealDigest']}}
            # A host seal is not an authenticated936 receipt or runtime cursor.
            with self.assertRaisesRegex(ValueError,'runtime_pre_order'):self.verify(state=state,index=937)
            state['candidatePreparation']['sha256']='0'*64
            with self.assertRaisesRegex(ValueError,'exact_digest'):self.verify(state=state,index=937)
            state['candidatePreparation']['sha256']=created['sealDigest']
            state['manifest']=copy.deepcopy(self.manifest);state['manifest']['historicalUncertainty']['resolved']=True
            with self.assertRaisesRegex(ValueError,'seal_manifest'):self.verify(state=state,index=937)
            self.assertEqual(before,{p.name:o.sha(p.read_bytes()) for p in path.iterdir()})

    def test_private_adapter_keeps_full_success_gate_and_fresh_helpers_and_leaves_default(self):
        factory=o._candidate_runtime_pins;checks=[]
        def observed(j):
            real=factory(j)
            def checked():checks.append(j);return real()
            return checked
        default=o.startup_manifest();default_loader=default._candidate_python_helpers
        with mock.patch.object(o,'_candidate_runtime_pins',side_effect=observed):
            a,j,_,_,unchanged=o._candidate_preparation_sources()
            with mock.patch.object(a,'_candidate_python_helpers',wraps=a._candidate_python_helpers)as loads,\
                    mock.patch.object(default,'candidate_verify_accounts',wraps=default.candidate_verify_accounts)as default_verify,\
                    mock.patch.object(o,'startup_manifest',wraps=o.startup_manifest)as source_checks:
                result=self.verify(_candidate_adapter=a)
                self.assertEqual(result['accountDigests'],self.ref['baseline'])
                self.assertEqual(source_checks.call_count,2);default_verify.assert_not_called()
                self.assertGreater(loads.call_count,0);self.assertEqual(len(checks),2)
                before=len(checks);bad=copy.deepcopy(self.accounts);bad.pop(next(iter(bad)))
                with self.assertRaisesRegex(ValueError,'state_exact_coverage'):
                    o.verify_states(self.c,{'manifest':self.manifest},self.manifest['phases'][0],'PRE',bad,
                        context_slot=self.context,_candidate_adapter=a)
                self.assertEqual(len(checks),before)
                before=len(checks)
                with mock.patch.object(j,'NODE',Path('/synthetic-public-runtime-policy-drift')),\
                        self.assertRaisesRegex(ValueError,'runtime_policy_changed'):
                    self.verify(_candidate_adapter=a)
                self.assertEqual(len(checks),before);default_verify.assert_not_called()
                self.assertIs(default._candidate_python_helpers,default_loader)
                expected=self.verify()
                default_verify.assert_called_once();self.assertEqual(result,expected)
                self.assertEqual(len(checks),before)

    def test_private_adapter_retains_first_and_final_real_manifest_source_refusals(self):
        a,_,_,_,unchanged=o._candidate_preparation_sources();read=Path.read_bytes
        target=Path(o.__file__).with_name('startup_manifest.py');retained=read(target)
        for fault_at in (1,2):
            reads=[]
            def changed(path):
                raw=read(path)
                if path==target:
                    reads.append(raw)
                    if len(reads)==fault_at:return raw+b'\n# synthetic public source read fault\n'
                return raw
            with self.subTest(source_check=fault_at),mock.patch.object(Path,'read_bytes',new=changed),\
                    mock.patch.object(a,'candidate_verify_accounts',wraps=a.candidate_verify_accounts)as verify:
                with self.assertRaisesRegex(ValueError,'startup_manifest_source_pin'):self.verify(_candidate_adapter=a)
                self.assertEqual(len(reads),fault_at);self.assertEqual(verify.call_count,fault_at-1)
            self.assertEqual(read(target),retained)

    def test_private_adapter_is_wired_only_from_existing_factory_owners(self):
        tree=ast.parse(Path(o.__file__).read_bytes());routed=[]
        for function in (node for node in tree.body if isinstance(node,ast.FunctionDef)):
            for call in (node for node in ast.walk(function)if isinstance(node,ast.Call)
                    and isinstance(node.func,ast.Name)and node.func.id=='verify_states'):
                private=[keyword.value for keyword in call.keywords if keyword.arg=='_candidate_adapter']
                if private:
                    self.assertEqual(len(private),1);self.assertEqual(ast.dump(private[0]),ast.dump(ast.Name(id='a',ctx=ast.Load())))
                    routed.append(function.name)
        self.assertEqual(routed,['_candidate_prefix_intent','_candidate_prefix_result','_candidate_governed_startup_operate'])


class CandidateFundingIntegrationTests(unittest.TestCase):
    """No real funded positive: canonical Source signatures are never generated."""
    @classmethod
    def setUpClass(cls):
        from tools.test_startup_manifest import CandidateFundingReceiptTests
        cls.host=CandidateFundingReceiptTests;cls.host.setUpClass()
        cls.a=o.startup_manifest();cls.ref=cls.a.candidate_oracle();cls.manifest=cls.host.manifest;cls.c=cls.host.c
        path=next(Path(p) for p in cls.ref['provenance']['evidence'] if p.endswith('/snapshot60.json'))
        raw=path.read_bytes();assert o.sha(raw)==cls.ref['provenance']['evidence'][str(path)]
        snapshot=json.loads(raw);cls.context=snapshot['contextSlot']
        cls.accounts={row['address']:{k:row[k] for k in ('exists','owner','executable','lamports','dataBase64')} for row in snapshot['accounts']}

    def verify(self,receipt,stage='PRE'):
        state={'manifest':self.manifest,'candidateFunding':{
            'receipt':receipt,'expectedReceiptSha256':o.sha(o.encode(receipt))}}
        return o.verify_states(self.c,state,self.manifest['phases'][0],stage,self.accounts,context_slot=self.context)

    def test_actual_state_boundary_revalidates_signature_and_never_credits_unsigned_receipt(self):
        receipt,_,_=self.host.fixture()
        for stage in ('PRE','POST'):
            with self.subTest(stage=stage),self.assertRaisesRegex(ValueError,'signature_or_receipt:zero_signature'):
                self.verify(receipt,stage)
        self.assertEqual(self.accounts[self.ref['roles']['h2']]['lamports'],'151301960')
        self.assertFalse(self.manifest['historicalUncertainty']['resolved'])

    def test_recipient_amount_boolean_and_uncommitted_replacement_cannot_unlock_state(self):
        receipt,_,_=self.host.fixture()
        for offset in (65+36,214):  # H2 key and exact u64 transfer amount in the fixed message.
            bad=copy.deepcopy(receipt);wire=bytearray(base64.b64decode(bad['wireBase64']));wire[offset]^=1
            bad['wireBase64']=base64.b64encode(wire).decode()
            with self.assertRaisesRegex(ValueError,'original_wire'):self.verify(bad)
        state={'manifest':self.manifest,'candidateFunding':True}
        with self.assertRaises(ValueError):
            o.verify_states(self.c,state,self.manifest['phases'][0],'PRE',self.accounts,context_slot=self.context)
        state['candidateFunding']={'receipt':receipt,'expectedReceiptSha256':'0'*64}
        with self.assertRaisesRegex(ValueError,'receipt_digest'):
            o.verify_states(self.c,state,self.manifest['phases'][0],'PRE',self.accounts,context_slot=self.context)


class CandidateFundingRoutePropagationTests(unittest.TestCase):
    """HOST consuming API only; no journal handoff, signed or native positive.

    Positives scope checker responses to source-rechecked cached adapters and
    fresh pinned helper modules, restoring all patched attributes on exit.
    Prefix/runtime expression checks prove argument wiring, not workflow execution.
    """
    @classmethod
    def setUpClass(cls):
        from tools.test_startup_manifest import CandidateSavedFundingAccountVerificationTests
        cls.fixture_type=CandidateSavedFundingAccountVerificationTests;cls.fixture_type.setUpClass()

    def setUp(self):
        self.f=self.fixture_type();self.f.setUp();self.adapters=[];self.boundaries=[];self.node_methods=[]
        self.state={'manifest':self.f.manifest,'candidateFunding':{
            'receipt':self.f.receipt,'expectedReceiptSha256':o.sha(o.encode(self.f.receipt))},
            'candidateFundingRoute':copy.deepcopy(self.f.route)}

    def verify(self,*,positive=False):
        real=o.startup_manifest;configured={};nodes=set()
        with ExitStack() as patches:
            def load():
                a=real() # Rechecks source; the adapter itself can be cached.
                if id(a) not in configured:
                    helpers=a._candidate_helpers
                    if positive:
                        def synthetic_helpers():
                            j,r,c=helpers()
                            if id(j) not in nodes:
                                nodes.add(id(j));self.node_methods.append((j,j._node_request))
                                patches.enter_context(mock.patch.object(j,'_node_request',self.f.checker))
                            return j,r,c
                        patches.enter_context(mock.patch.object(a,'_candidate_helpers',side_effect=synthetic_helpers))
                    check=mock.Mock(wraps=a.candidate_verify_accounts)
                    patches.enter_context(mock.patch.object(a,'candidate_verify_accounts',check));configured[id(a)]=check
                self.adapters.append(a);self.boundaries.append(configured[id(a)]);return a
            patches.enter_context(mock.patch.object(o,'startup_manifest',side_effect=load))
            return o.verify_states(self.f.c,self.state,self.f.manifest['phases'][0],'PRE',self.f.accounts,context_slot=self.f.context)

    def test_private_funding_and_accounts_keep_nested_full_success_postconditions(self):
        # Actual validators and public fixtures; only the existing checker
        # boundary is simulated. Drift is injected after an original returns.
        completed=[];checks=[];fault=[None];real_exec=exec;factory=o._candidate_runtime_pins
        def instrument(code,namespace,**kwargs):
            real_exec(code,namespace,**kwargs)
            if namespace.get('__file__')==str(HERE/'startup_manifest.py'):
                funding=namespace['candidate_verify_funding'];accounts=namespace['candidate_verify_accounts']
                def checked_funding(*args,**kw):
                    result=funding(*args,**kw);completed.append('funding');return result
                def checked_accounts(*args,**kw):
                    result=accounts(*args,**kw);completed.append('accounts');return result
                namespace['candidate_verify_funding']=checked_funding;namespace['candidate_verify_accounts']=checked_accounts
        def runtime(j):
            real=factory(j)
            def checked():
                result=real();checks.append(tuple(completed))
                if fault[0]=='runtime'and completed:raise o.Stop('synthetic_full_runtime_refusal')
                return result
            return checked
        read=Path.read_bytes;target=HERE/'startup_public_journal.py'
        def changed(path):
            raw=read(path)
            return raw+b'\n# synthetic post-success source drift\n'if fault[0]=='source'and completed==['funding','accounts']and path==target else raw
        with mock.patch.object(o,'exec',side_effect=instrument,create=True),mock.patch.object(o,'_candidate_runtime_pins',side_effect=runtime):
            a,_,_,_,unchanged=o._candidate_preparation_sources();load=a._candidate_python_helpers
            with ExitStack()as patches:
                def helpers():
                    repo,modules=load();patches.enter_context(mock.patch.object(modules[0],'_node_request',self.f.checker));return repo,modules
                patches.enter_context(mock.patch.object(a,'_candidate_python_helpers',side_effect=helpers))
                def verify():
                    return a.candidate_verify_accounts(self.f.manifest,a.candidate_phase(0),'PRE',self.f.accounts,
                        context_slot=self.f.context,funding_receipt=self.f.receipt,
                        funding_receipt_sha256=self.state['candidateFunding']['expectedReceiptSha256'],expected_funding_route=self.f.route)
                result=verify();self.assertTrue(result['accountBytesVerified']);self.assertFalse(result['fundingAuthenticated'])
                self.assertEqual(completed,['funding','accounts']);self.assertEqual(checks,[(),('funding',),('funding','accounts')])
                completed.clear();fault[0]='runtime'
                with self.assertRaisesRegex(ValueError,'synthetic_full_runtime_refusal'):verify()
                self.assertEqual(completed,['funding']);self.assertEqual(checks[-1],('funding',))
                completed.clear();fault[0]='source';before=len(checks)
                with mock.patch.object(Path,'read_bytes',new=changed),self.assertRaisesRegex(ValueError,'candidate_helper_source_pin'):verify()
                self.assertEqual(completed,['funding','accounts']);self.assertEqual(len(checks),before+1)
                self.assertEqual(checks[-1],('funding',))

    def test_checker_adapter_and_helpers_restore_after_success_failure_and_before_real_crypto(self):
        adapter=o.startup_manifest();helpers=adapter._candidate_helpers;verify=adapter.candidate_verify_accounts
        def restored():
            self.assertIs(adapter._candidate_helpers,helpers)
            self.assertIs(adapter.candidate_verify_accounts,verify)
            for journal,checker in self.node_methods:self.assertIs(journal._node_request,checker)
        self.assertTrue(self.verify(positive=True)['accountBytesVerified']);restored();self.f.checker.reset_mock()
        self.test_real_checker_zero_and_invalid_signatures_refuse_at_operator_boundary();restored()
        self.f.accounts.pop(self.f.r['rent'])
        with self.assertRaisesRegex(ValueError,'state_exact_coverage'):self.verify(positive=True)
        restored();self.f.checker.assert_not_called()

    def test_mock_checker_v2_consuming_boundary_preserves_complete_bytes_and_false_flags(self):
        result=self.verify(positive=True)
        self.assertEqual(len(self.adapters),2) # Includes unchanged final fresh source check.
        self.assertIs(self.boundaries[0].call_args.kwargs['expected_funding_route'],self.state['candidateFundingRoute'])
        self.f.checker.assert_called_once();self.assertTrue(result['accountBytesVerified'])
        self.assertEqual(len(result['checkedAccountKeys']),60)
        self.assertEqual(result['accountDigests'][self.f.r['h2']]['lamports'],'4583191240')
        for key in ('fundingAuthenticated','durableReceiptLineageAuthenticated','chainStateAuthenticated',
                'executionLineageAuthenticated','completePrerequisitesAuthenticated','fullSnapshotVerified','liveReady'):
            self.assertFalse(result[key])

    def test_mock_checker_v1_omits_optional_keyword_even_for_explicit_none(self):
        receipt=self.state['candidateFunding']['receipt'];receipt.pop('route')
        receipt.update(schema='piv1-candidate-supplied-funding-receipt-v1',endpoint=self.f.c.ENDPOINT)
        self.state['candidateFunding']['expectedReceiptSha256']=o.sha(o.encode(receipt))
        for supplied_none in (False,True):
            if supplied_none:self.state['candidateFundingRoute']=None
            else:self.state.pop('candidateFundingRoute')
            with self.subTest(supplied_none=supplied_none):
                self.assertTrue(self.verify(positive=True)['accountBytesVerified'])
                self.assertNotIn('expected_funding_route',self.boundaries[-2].call_args.kwargs)

    def test_missing_wrong_malformed_and_receipt_selected_route_are_not_policy(self):
        route=copy.deepcopy(self.f.route);wrong=copy.deepcopy(route);wrong['originCommitmentSha256']='3'*64
        for value,reason in ((None,'candidate_funding_scope'),(wrong,'candidate_funding_route_binding'),
                ({},'candidate_funding_route_')):
            self.state['candidateFundingRoute']=value
            with self.subTest(reason=reason),self.assertRaisesRegex(ValueError,reason):self.verify()
        self.state['candidateFundingRoute']=route
        self.state['candidateFunding']['receipt']['route']=wrong
        self.state['candidateFunding']['expectedReceiptSha256']=o.sha(o.encode(self.state['candidateFunding']['receipt']))
        with self.assertRaisesRegex(ValueError,'candidate_funding_route_binding'):self.verify()
        self.f.checker.assert_not_called()

    def test_route_without_complete_pair_refuses_before_account_verifier(self):
        original=copy.deepcopy(self.state['candidateFunding'])
        for funding in (None,{}, {'receipt':original['receipt']},
                {'receipt':None,'expectedReceiptSha256':original['expectedReceiptSha256']},
                {'receipt':original['receipt'],'expectedReceiptSha256':None}):
            self.state['candidateFunding']=funding;self.boundaries=[]
            with self.subTest(funding_fields=None if funding is None else tuple(funding)),self.assertRaisesRegex(ValueError,'object_fields|candidate_funding_commitment_required'):
                self.verify()
            self.boundaries[0].assert_not_called()
        self.f.checker.assert_not_called()

    def test_real_checker_zero_and_invalid_signatures_refuse_at_operator_boundary(self):
        for signature,reason in ((bytes(64),'zero_signature'),(bytes([1])*64,'signature_invalid')):
            receipt=self.f.receipt_fixture(signature)
            self.state['candidateFunding']={'receipt':receipt,'expectedReceiptSha256':o.sha(o.encode(receipt))}
            with self.subTest(reason=reason),self.assertRaisesRegex(ValueError,'signature_or_receipt:'+reason):self.verify()
        self.f.checker.assert_not_called()

    def test_mock_checker_clock_source_h2_and_exact_coverage_guards_remain(self):
        import struct
        original=copy.deepcopy(self.f.accounts)
        for fault,reason in (('clock','state_before_funding_anchor'),('source','state_source_identity'),
                ('h2','state_byte_mismatch'),('coverage','state_exact_coverage')):
            self.f.accounts=copy.deepcopy(original)
            if fault=='coverage':self.f.accounts.pop(self.f.r['rent'])
            elif fault=='clock':
                clock=list(struct.unpack('<QqQQq',base64.b64decode(self.f.accounts[self.f.r['clock']]['dataBase64'])))
                clock[4]-=1;self.f.accounts[self.f.r['clock']]['dataBase64']=base64.b64encode(struct.pack('<QqQQq',*clock)).decode()
            elif fault=='source':self.f.accounts[self.f.r['source']]['owner']=self.f.r['h2']
            else:self.f.accounts[self.f.r['h2']]['lamports']=str(int(self.f.accounts[self.f.r['h2']]['lamports'])+1)
            with self.subTest(fault=fault),self.assertRaisesRegex(ValueError,reason):self.verify(positive=True)

    def source_calls(self,name,target):
        tree=ast.parse(Path(o.__file__).read_bytes());function=next(n for n in tree.body if isinstance(n,ast.FunctionDef)and n.name==name)
        return [n for n in ast.walk(function)if isinstance(n,ast.Call)and
            ((isinstance(n.func,ast.Name)and n.func.id==target)or(isinstance(n.func,ast.Attribute)and n.func.attr==target))]

    def expression(self,node,state):
        return eval(compile(ast.Expression(node),'<actual-route-argument-expression>','eval'),{}, {'state':state})

    def test_source_prefix_pre_post_state_expressions_forward_only_independent_route(self):
        # Evaluate the actual call's state expression only, not prefix workflows.
        for name in ('_candidate_prefix_intent','_candidate_prefix_result'):
            call,=self.source_calls(name,'verify_states')
            for supplied in (None,False,True):
                state={'manifest':self.f.manifest,'funding':self.state['candidateFunding']}
                if supplied is not False:state['fundingRoute']=self.state['candidateFundingRoute'] if supplied else None
                result=self.expression(call.args[1],state)
                self.assertIs(result['candidateFunding'],state['funding'])
                if supplied:self.assertIs(result['candidateFundingRoute'],state['fundingRoute'])
                else:self.assertNotIn('candidateFundingRoute',result)

    def test_source_runtime_pre_post_keyword_expressions_preserve_absent_v1_shape(self):
        # Evaluate only the actual optional keyword expansion; no runtime proof.
        for name in ('_candidate_runtime_intent','_candidate_prefix_result'):
            call,=self.source_calls(name,'candidate_verify_accounts')
            expansion,=[kw.value for kw in call.keywords if kw.arg is None]
            for supplied in (None,False,True):
                state={'funding':self.state['candidateFunding']}
                if supplied is not False:state['fundingRoute']=self.state['candidateFundingRoute'] if supplied else None
                result=self.expression(expansion,state)
                if supplied:self.assertIs(result['expected_funding_route'],state['fundingRoute'])
                else:self.assertEqual(result,{})


class CandidateGovernedFundingHandoffTests(unittest.TestCase):
    """Synthetic durable HOST handoff, never old actual records/live admission.

    Positive crypto responses alone are mocked; chain/raw/account/phase/storage
    validators run. Real signature negatives run with no checker replacement.
    """
    @classmethod
    def setUpClass(cls):
        from tools.test_startup_manifest import CandidateSavedFundingAccountVerificationTests
        cls.fixture_type=CandidateSavedFundingAccountVerificationTests;cls.fixture_type.setUpClass()
        cls.a=o.startup_manifest();cls.j,_,cls.c=cls.a._candidate_helpers()

    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name);self.path=self.root/'fresh.governed-startup';self.tail=None;self.parent_number=0
        self.f=self.fixture_type();self.f.setUp();self.manifest=self.f.manifest
        self.build_parent()

    def build_parent(self,signature=bytes(64)):
        self.parent_number+=1;self.parent=self.root/f'funding-{self.parent_number}.candidate-controller'
        receipt=self.f.receipt_fixture(signature);fd=self.j.directory(self.parent,True)
        try:
            self.j.write_file(fd,'lock',o.CANDIDATE_CONTROLLER_LOCK);meta=os.stat('lock',dir_fd=fd)
            identity={'device':str(meta.st_dev),'inode':str(meta.st_ino),'ctimeNs':str(meta.st_ctime_ns)}
            _,_,_,domain,_=o._candidate_preparation_sources();domain=copy.deepcopy(domain)
            domain.update({'tools/startup_release_operator.py':'a'*64,'tools/startup_manifest.py':'b'*64,'fundingRoute':self.f.route})
            records=[]
            def record(kind,data):
                body={'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'sequence':str(len(records)),
                    'previousDigest':None if not records else records[-1]['digest'],'lockIdentity':identity,'kind':kind,'data':data}
                value={**body,'digest':o.sha(o.encode(body))};records.append(value)
                self.j.write_file(fd,f'{len(records)-1:08}.json',o.encode(value));return value
            first=record('CREATE',{'path':str(self.parent),'manifest':self.manifest,'manifestSha256':o.sha(o.encode(self.manifest)),
                'sourceDomain':domain,'historicalUncertaintyReference':self.manifest['historicalUncertainty']})
            prepared=o._candidate_prepare_funding(self.a,self.j,self.manifest,
                {k:receipt[k]for k in ('feeCapLamports','originalBlockhash','preflight')},first['digest'],domain)
            record('PREPARE_FUNDING',prepared)
            intent=o._candidate_controller_intent(self.a,self.j,self.manifest,
                {k:receipt[k]for k in ('wireBase64','feeCapLamports','originalBlockhash','preflight')})
            record('FUNDING_INTENT',{**intent,'preparationRequestId':prepared['request']['requestId']})
            record('FUNDING_RESULT',{'receipt':receipt,'expectedReceiptSha256':o.sha(o.encode(receipt))})
            root=os.fstat(fd)
            self.anchor={'path':str(self.parent),'directoryIdentity':{'device':str(root.st_dev),'inode':str(root.st_ino)},
                'lockIdentity':identity,'recordSha256':{f'{i:08}.json':o.sha(o.encode(row))for i,row in enumerate(records)},
                'tail':records[-1]['digest'],'sourceDomainSha256':o.sha(o.encode(domain)),
                'historicalSourcePins':{key:domain[key]for key in ('tools/startup_release_operator.py','tools/startup_manifest.py','tools/startup_public_journal.py')},
                'manifestSha256':o.sha(o.encode(self.manifest)),'fundingReceiptSha256':o.sha(o.encode(receipt)),
                'originalWireSha256':intent['identity']['wireSha256'],'originalSignatureHex':intent['identity']['signatureHex'],
                'expectedFundingRoute':copy.deepcopy(self.f.route)}
            self.records=records
        finally:os.close(fd)

    @contextmanager
    def positive_checker(self):
        real=o._candidate_preparation_sources;patched=set();adapters=set()
        def checker(requests):
            return [{'reason':'ALL_DECLARED_PHASES_FINALIZED_WITH_EXPECTED_POSTSTATE' if request['trace'][-1]['state']=='FINALIZED'
                else 'PENDING_OR_UNKNOWN_NO_REPLACEMENT'}for request in requests]
        with ExitStack()as patches:
            def patch_j(j):
                if id(j)not in patched:
                    patched.add(id(j));patches.enter_context(mock.patch.object(j,'_node_request',side_effect=checker))
            def patch_adapter(a):
                if id(a)not in adapters:
                    adapters.add(id(a));helpers=a._candidate_helpers
                    def supplied_helpers():
                        result=helpers();patch_j(result[0]);return result
                    patches.enter_context(mock.patch.object(a,'_candidate_helpers',side_effect=supplied_helpers))
            def sources():
                a,j,s,domain,unchanged=real();patch_j(j);patch_adapter(a)
                return a,j,s,domain,unchanged
            # Direct Journal replay also uses this independently held fixture helper.
            patch_j(self.j)
            # verify_states uses this source-rechecked cached adapter separately.
            patch_adapter(o.startup_manifest())
            patches.enter_context(mock.patch.object(o,'_candidate_preparation_sources',side_effect=sources));yield

    def call(self,action,data=None,*,anchor=None,**private):
        out=o._candidate_governed_startup_operate({'schema':o.CANDIDATE_GOVERNED_SCHEMA,'action':action,'expectedTail':self.tail,'data':data},
            self.path,expected_handoff=self.anchor if anchor is None else anchor,**private)
        self.tail=out['tail'];return out

    def hashes(self,path):return {p.name:o.sha(p.read_bytes())for p in path.iterdir()if p.is_file()}

    def frame(self,queries,results):
        return {name:base64.b64encode(self.c.encode(value)).decode()for name,value in (
            ('requestBase64',[{'jsonrpc':'2.0','id':i+1,'method':method,'params':params}for i,(method,params)in enumerate(queries)]),
            ('responseBase64',[{'jsonrpc':'2.0','id':i+1,'result':value}for i,value in enumerate(results)]),('httpBase64',{'status':200,'headers':[]}))}

    def preparation(self):
        import struct
        receipt=self.records[-1]['data']['receipt'];floor=int(self.c.strict_json(base64.b64decode(receipt['poststate']['responseBase64']))[0]['result']['context']['slot'])
        origin=floor+1;context=floor+2;config={'commitment':'finalized'};hash_value=self.f.r['vault0']
        review=o._candidate_prefix_review(self.a,self.c,self.manifest,0,hash_value);count=len(review['requiredSigners'])
        accounts=copy.deepcopy(self.f.accounts);clock=list(struct.unpack('<QqQQq',base64.b64decode(accounts[self.f.r['clock']]['dataBase64'])))
        clock[0]=context;clock[4]+=2;accounts[self.f.r['clock']]['dataBase64']=base64.b64encode(struct.pack('<QqQQq',*clock)).decode()
        keys=self.a.candidate_expected(self.manifest,0,'PRE',funding_mode='CONDITIONAL_H2_CREDIT')['requiredSnapshotKeys']
        values=[None if not accounts[key]['exists']else {'owner':accounts[key]['owner'],'executable':accounts[key]['executable'],
            'lamports':int(accounts[key]['lamports']),'data':[accounts[key]['dataBase64'],'base64']}for key in keys]
        fee=int(self.manifest['phases'][0]['historicalFeeProjectionLamports'])
        return {'wireBase64':base64.b64encode(bytes([count])+bytes(64*count)+base64.b64decode(review['messageBase64'])).decode(),
            'feeCapLamports':str(fee),'originalBlockhash':self.frame([('getGenesisHash',[]),('getLatestBlockhash',[{**config,'minContextSlot':floor}])],
                [self.c.GENESIS,{'context':{'slot':origin},'value':{'blockhash':hash_value,'lastValidBlockHeight':300}}]),
            'preflight':self.frame([('getFeeForMessage',[review['messageBase64'],config]),
                ('getMultipleAccounts',[keys,{'encoding':'base64',**config,'minContextSlot':origin}]),
                ('getBlockHeight',[config]),('isBlockhashValid',[hash_value,config])],
                [{'context':{'slot':context},'value':fee},{'context':{'slot':context},'value':values},100,{'context':{'slot':context},'value':True}])}

    def test_durable_distinct_create_reopen_verify_and_actual_first_prepare_keep_old_bytes(self):
        before=self.hashes(self.parent);cached=o.startup_manifest();helpers=cached._candidate_helpers
        with self.positive_checker(),mock.patch.object(o,'public_rpc_transport',side_effect=AssertionError('HOST must not collect RPC')):
            created=self.call('CREATE_HANDOFF',self.manifest)
            self.assertEqual(created['startupCursor'],0);self.assertFalse(created['originalPending']);self.assertFalse(created['liveReady'])
            self.assertEqual(self.call('INSPECT')['tail'],created['tail'])
            verified=self.call('VERIFY',{'accounts':self.f.accounts,'contextSlot':self.f.context})['accountVerification']
            self.assertTrue(verified['accountBytesVerified']);self.assertFalse(verified['fundingAuthenticated'])
            prepared=self.call('PREPARE_PHASE',self.preparation());self.assertTrue(prepared['phasePreparationRetained'])
            self.assertTrue(self.call('INSPECT')['phasePreparationRetained'])
            _,journal_helper,_,_,_=o._candidate_preparation_sources()
            journal=o.Journal(journal_helper,self.path,_governed=True,_expected_handoff=self.anchor)
            try:
                state=o._candidate_governed_replay(journal_helper,journal.chain,self.path,self.anchor)
                self.assertEqual(state['phasePreparation']['request']['operatorTail'],created['tail'])
                for key in ('intent','pending','runtimeFacts','initializerBinding','phaseSignAttempt'):self.assertIsNone(state[key])
            finally:journal.close()
        self.assertIs(cached._candidate_helpers,helpers)
        with self.assertRaisesRegex(RuntimeError,'checker_scope_unwind'),self.positive_checker():
            raise RuntimeError('checker_scope_unwind')
        self.assertIs(cached._candidate_helpers,helpers)
        with self.assertRaisesRegex(ValueError,'zero_signature'):self.call('INSPECT')
        self.assertEqual(before,self.hashes(self.parent))
        seed=json.loads((self.path/'00000000.json').read_bytes())['data']
        self.assertNotIn('fundingProof',seed);self.assertNotEqual(seed['sourceDomain']['tools/startup_release_operator.py'],'a'*64)

    def test_real_zero_invalid_signature_parent_never_creates_startup_journal(self):
        for signature,reason in ((bytes(64),'zero_signature'),(bytes([1])*64,'signature_invalid')):
            if any(signature):self.build_parent(signature)
            with self.subTest(reason=reason),self.assertRaisesRegex(ValueError,reason):self.call('CREATE_HANDOFF',self.manifest)
            self.assertFalse(self.path.exists())

    def test_independent_route_source_tail_and_original_identity_are_enforced_before_create(self):
        for fault,reason in (('route','source_domain'),('source','source_domain'),('tail','parent_tail'),('wire','original_signature'),('signature','original_signature')):
            anchor=copy.deepcopy(self.anchor)
            if fault=='route':anchor['expectedFundingRoute']['originCommitmentSha256']='3'*64
            elif fault=='source':anchor['sourceDomainSha256']='c'*64
            elif fault=='tail':anchor['tail']='c'*64
            elif fault=='wire':anchor['originalWireSha256']='c'*64
            else:anchor['originalSignatureHex']='01'*64
            with self.subTest(fault=fault),self.positive_checker(),self.assertRaisesRegex(ValueError,reason):self.call('CREATE_HANDOFF',self.manifest,anchor=anchor)
            self.assertFalse(self.path.exists())

    def test_parent_gap_raw_change_lock_and_directory_substitution_refuse(self):
        raw=(self.parent/'00000002.json').read_bytes();(self.parent/'00000002.json').rename(self.parent/'preserved-record')
        with self.assertRaisesRegex(ValueError,'parent_namespace'):self.call('CREATE_HANDOFF',self.manifest)
        (self.parent/'preserved-record').rename(self.parent/'00000002.json')
        (self.parent/'00000002.json').write_bytes(raw+b' ')
        with self.assertRaisesRegex(ValueError,'parent_record_sha'):self.call('CREATE_HANDOFF',self.manifest)
        (self.parent/'00000002.json').write_bytes(raw)
        for key in ('lockIdentity','directoryIdentity'):
            anchor=copy.deepcopy(self.anchor);anchor[key]['inode']=str(int(anchor[key]['inode'])+1)
            with self.subTest(identity=key),self.assertRaisesRegex(ValueError,'parent_lock|parent_directory'):self.call('CREATE_HANDOFF',self.manifest,anchor=anchor)
        self.assertFalse(self.path.exists())

    def test_symlink_ancestor_cannot_create_inside_consumed_parent(self):
        alias=self.root/'alias';alias.symlink_to(self.root,target_is_directory=True)
        nested=alias/self.parent.name/'child.governed-startup';before=self.hashes(self.parent);names=set(self.parent.iterdir())
        with self.positive_checker(),self.assertRaises(OSError):
            o._candidate_governed_startup_operate({'schema':o.CANDIDATE_GOVERNED_SCHEMA,'action':'CREATE_HANDOFF','expectedTail':None,'data':self.manifest},
                nested,expected_handoff=self.anchor)
        self.assertEqual(before,self.hashes(self.parent));self.assertEqual(names,set(self.parent.iterdir()))
        self.assertFalse((self.parent/'child.governed-startup').exists())

    def test_reopen_requires_same_external_anchor_and_current_source_domain(self):
        with self.positive_checker():self.call('CREATE_HANDOFF',self.manifest)
        before=self.hashes(self.path);changed=copy.deepcopy(self.anchor);changed['tail']='c'*64
        with self.assertRaisesRegex(ValueError,'seed_binding'):self.call('INSPECT',anchor=changed)
        real=o._candidate_preparation_sources
        def changed_sources():
            a,j,s,domain,guard=real();domain['tools/startup_release_operator.py']='c'*64;return a,j,s,domain,guard
        with mock.patch.object(o,'_candidate_preparation_sources',side_effect=changed_sources),self.assertRaisesRegex(ValueError,'seed_binding'):self.call('INSPECT')
        self.assertEqual(before,self.hashes(self.path))

    def test_hold_after_prepare_and_bad_supplied_lifetime_never_replaces_original(self):
        with self.positive_checker():
            self.call('CREATE_HANDOFF',self.manifest);data=self.preparation();bad=copy.deepcopy(data)
            rows=self.c.strict_json(base64.b64decode(bad['preflight']['responseBase64']));rows[2]['result']=261
            bad['preflight']['responseBase64']=base64.b64encode(self.c.encode(rows)).decode();before=self.hashes(self.path)
            with self.assertRaisesRegex(ValueError,'declared_preflight'):self.call('PREPARE_PHASE',bad)
            self.assertEqual(before,self.hashes(self.path));self.call('PREPARE_PHASE',data);before=self.hashes(self.path)
            with self.assertRaisesRegex(ValueError,'funding_or_hold'):self.call('PREPARE_PHASE',data)
            self.assertEqual(before,self.hashes(self.path));self.assertTrue(self.call('INSPECT')['phasePreparationRetained'])

    def test_partial_commit_is_retained_and_reopen_refuses_without_repair(self):
        with self.positive_checker():
            self.call('CREATE_HANDOFF',self.manifest);write=os.write;target=str(self.path/'00000001.json')
            def interrupted(fd,raw):
                if os.readlink(f'/proc/self/fd/{fd}')==target:
                    write(fd,raw[:23]);raise OSError('synthetic partial journal write')
                return write(fd,raw)
            with mock.patch.object(os,'write',side_effect=interrupted),self.assertRaisesRegex(ValueError,'commit_uncertain_no_retry_or_adoption'):
                self.call('PREPARE_PHASE',self.preparation())
            before=self.hashes(self.path);self.assertEqual((self.path/'00000001.json').stat().st_size,23)
            with self.assertRaises(ValueError):self.call('INSPECT')
            self.assertEqual(before,self.hashes(self.path))

    def test_modes_public_entry_and_all_live_or_funding_actions_are_closed(self):
        request={'schema':o.CANDIDATE_GOVERNED_SCHEMA,'action':'CREATE_HANDOFF','expectedTail':None,'data':self.manifest}
        with self.assertRaisesRegex(ValueError,'schema'):o.operate(request,self.path)
        for action in ('FUNDING_INTENT','FUNDING_RESULT','PREPARE_FUNDING'):
            with self.subTest(action=action),self.assertRaisesRegex(ValueError,'governed_action'):self.call(action,{})
        for action in ('PREPARE_PHASE','SIGN_PHASE_ATTEMPT','PHASE_INTENT'):
            with self.subTest(capture=action),self.assertRaisesRegex(ValueError,'supplied_data_only'):self.call(action)
        with self.assertRaisesRegex(ValueError,'journal_selector'):o.Journal(self.j,self.path,_candidate=True,_governed=True,_expected_handoff=self.anchor)
        with self.assertRaisesRegex(ValueError,'external_anchor_required'):o.Journal(self.j,self.path,_governed=True)
        with self.assertRaises(TypeError):o._candidate_governed_startup_operate(request,self.path)
        with self.assertRaises(TypeError):o._candidate_governed_startup_operate(request,self.path,expected_handoff=self.anchor,_transport=True)
        for key,reason in (('_native_profile_path','native_fresh_action_only'),('_submit_original','submission_private_scope')):
            with self.subTest(argument=key),self.assertRaisesRegex(ValueError,reason):o._candidate_governed_startup_operate(request,self.path,expected_handoff=self.anchor,**{key:True})
        self.assertFalse(self.path.exists())


class CandidateGovernedLifecycleTests(unittest.TestCase):
    """Fresh synthetic namespaces; checker and network/native boundaries only.

    The synthetic checker does not establish signatures or on-chain success.
    All durable replay, authorization, account, Clock, fee and original-byte
    validators execute. No historical actual journal or private input is read.
    """
    # Reuse fixture construction only; select each regression exactly once.
    setUpClass=classmethod(CandidateGovernedFundingHandoffTests.setUpClass.__func__)
    setUp=CandidateGovernedFundingHandoffTests.setUp
    build_parent=CandidateGovernedFundingHandoffTests.build_parent
    positive_checker=CandidateGovernedFundingHandoffTests.positive_checker
    call=CandidateGovernedFundingHandoffTests.call
    hashes=CandidateGovernedFundingHandoffTests.hashes
    frame=CandidateGovernedFundingHandoffTests.frame
    preparation=CandidateGovernedFundingHandoffTests.preparation

    def state(self):
        journal=o.Journal(self.j,self.path,_governed=True,_expected_handoff=self.anchor)
        try:return journal.replay()
        finally:journal.close()

    def sign_data(self,state,expiry='18446744073709551615'):
        view=o._candidate_signing_view(self.a,state)
        receipt={'schema':'piv1-launcher-release-authorization-v2','manifestSha256':view['manifestSha256'],'baseParentTail':None,
            'operatorSourceSha256':o.sha(Path(o.__file__).read_bytes()),'expiresUnixTimestamp':expiry,'scope':'EXACT_BOUNDED_TESTNET_RELEASE',
            'feeBudgetNamespace':view['manifest']['proposedFeeBudget']['namespace'],'signerBackendSha256':o.SIGNER_BACKEND_SHA,'signerProfileSha256':'c'*64}
        raw=o.encode(receipt)
        return {'authorizationRawBase64':base64.b64encode(raw).decode(),'authorizationSha256':o.sha(raw)}

    @contextmanager
    def boundaries(self,*,unknown=False,fault=None):
        import struct
        prepared=self.preparation();pre=self.c.strict_json(base64.b64decode(prepared['preflight']['responseBase64']))
        before=copy.deepcopy(pre[1]['result']);calls=[];native=[];sent=[];active={}
        stable=o.Journal.stable
        def track(journal):active['journal']=journal;return stable(journal)
        def invoke(j,payload,*,_raw_capture):
            native.append(copy.deepcopy(payload))
            wire=base64.b64decode(payload['packets'][0]['wireBase64']);wire=wire[:1]+bytes([1])*(64*wire[0])+wire[1+64*wire[0]:]
            response={'schema':'piv1-startup-signing-response-v1','requestId':payload['request']['requestId'],
                'packets':[{'id':payload['packets'][0]['id'],'wireBase64':base64.b64encode(wire).decode()}]}
            literal=b' '+json.dumps(response,indent=1).encode()+b' \n';_raw_capture(literal,True)
            if fault is not None:fault('native',active['journal'])
            return response
        def finalized():
            journal=active['journal'];state=journal.replay();pending=state['pending'];slot=int(before['context']['slot'])+2
            post=copy.deepcopy(before);post['context']['slot']=slot+1
            keys=self.a.candidate_expected(self.manifest,0,'POST',funding_mode='CONDITIONAL_H2_CREDIT')['requiredSnapshotKeys']
            expected=self.a.candidate_expected(self.manifest,0,'POST',funding_mode='CONDITIONAL_H2_CREDIT')['accounts'];r=self.f.r
            rows=dict(zip(keys,post['value']))
            for role in ('g0','h2'):rows[r[role]]['lamports']=int(expected[r[role]]['lamports'])
            raw=struct.pack('<IB',1,1)+self.c.b58decode(r['g0'],32)+bytes(892352)
            rows[r['loaderBuffer']]={'owner':self.a.LOADER,'executable':False,'lamports':int(expected[r['loaderBuffer']]['lamports']),
                'data':[base64.b64encode(raw).decode(),'base64']}
            clock=list(struct.unpack('<QqQQq',base64.b64decode(rows[r['clock']]['data'][0])));clock[0]=slot+1;clock[4]+=2
            rows[r['clock']]['data'][0]=base64.b64encode(struct.pack('<QqQQq',*clock)).decode();post['value']=[rows[k]for k in keys]
            roles=[v['address']for v in pending['reviewed']['accountRoles']]
            balances=lambda values:[0 if values[k]is None else int(values[k]['lamports'])for k in roles]
            pre_accounts={key:None if not val['exists']else val for key,val in o._candidate_result_pre(pending).items()}
            tx={'slot':slot,'version':'legacy','transaction':[pending['input']['wireBase64'],'base64'],
                'meta':{'err':None,'fee':int(pending['feeQuoteLamports']),'preBalances':balances(pre_accounts),'postBalances':balances(rows)}}
            status={'context':{'slot':slot+1},'value':[None if unknown else {'slot':slot,'err':None,'confirmationStatus':'finalized'}]}
            return status,tx,post
        def transport(c,raw):
            batch=c.strict_json(raw);methods=[item['method']for item in batch];calls.append(batch)
            if methods==['getGenesisHash','getLatestBlockhash']:
                values=[row['result']for row in c.strict_json(base64.b64decode(prepared['originalBlockhash']['responseBase64']))]
            elif methods==['getFeeForMessage','getMultipleAccounts','getBlockHeight','isBlockhashValid']:
                values=[row['result']for row in pre]
            elif methods==['getGenesisHash','getFeeForMessage','getMultipleAccounts','isBlockhashValid','getBlockHeight']:
                values=[c.GENESIS,pre[0]['result'],before,pre[3]['result'],pre[2]['result']]
            elif methods==['sendTransaction']:
                self.assertEqual(len(sent),0);sent.append(batch[0]['params'][0]);values=[c.b58encode(base64.b64decode(sent[0])[1:65])]
                self.assertEqual(batch[0]['params'][1]['maxRetries'],0)
            elif methods==['getGenesisHash']:values=[c.GENESIS]
            elif methods==['getSignatureStatuses','getTransaction']:
                status,tx,_=finalized();values=[status,tx]
                original=c.b58encode(base64.b64decode(active['journal'].chain[-1]['data']['wireBase64'])[1:65])
                self.assertEqual(batch[0]['params'][0],[original]);self.assertEqual(batch[1]['params'][0],original)
            elif methods==['getMultipleAccounts']:values=[finalized()[2]]
            else:raise AssertionError('unexpected synthetic boundary: '+str(methods))
            body=c.encode([{'jsonrpc':'2.0','id':i+1,'result':value}for i,value in enumerate(values)])
            if fault is not None:fault('transport',active['journal'])
            return 200,[],body
        with mock.patch.object(o.Journal,'stable',track),mock.patch.object(o,'public_rpc_transport',side_effect=transport),mock.patch.object(o,'invoke_signer',side_effect=invoke):
            yield calls,native,sent,active

    def prepare_captured(self):
        self.call('CREATE_HANDOFF',self.manifest)
        return self.call('PREPARE_PHASE',{'feeCapLamports':self.manifest['phases'][0]['historicalFeeProjectionLamports']})

    def test_governed_capture_native_original_send_result_uses_actual_phase_orchestration(self):
        old=self.hashes(self.parent)
        with self.positive_checker(),self.boundaries()as(calls,native,sent,_):
            prepared=self.prepare_captured();state=self.state();data=self.sign_data(state)
            out=self.call('SIGN_PHASE_ATTEMPT',data,_native_profile_path='/not-read/public-fixture',_submit_original=True)
            self.assertEqual(out['startupCursor'],1);self.assertFalse(out['originalPending']);self.assertEqual(len(native),1);self.assertEqual(len(sent),1)
            self.assertEqual(base64.b64decode(out['nativeSignerResponse']['stdoutBase64'])[:1],b' ')
            self.assertEqual(out['nativeSignerResponse']['stdoutSha256'],o.sha(base64.b64decode(out['nativeSignerResponse']['stdoutBase64'])))
            self.assertTrue(prepared['phaseInputCapture']['unsignedPreparationVerified']);self.assertTrue(out['signAttemptInputCapture']['currentInputVerified'])
            self.assertEqual([json.loads((self.path/f'{i:08}.json').read_bytes())['kind']for i in range(5)],
                ['CREATE_HANDOFF','PREPARE_PHASE','SIGN_PHASE_ATTEMPT','PHASE_INTENT','PHASE_RESULT'])
            self.assertFalse(out['nativeDispatcherEnabled']);self.assertFalse(out['externalSigningAuthorized']);self.assertFalse(out['liveReady'])
            self.assertFalse(self.call('INSPECT')['originalPending'])
            self.assertEqual(sum(batch[0]['method']=='sendTransaction'for batch in calls),1)
        self.assertEqual(old,self.hashes(self.parent))

    def test_native_unknown_holds_reopen_never_resends_and_original_only_recovery_advances(self):
        with self.positive_checker(),self.boundaries(unknown=True)as(_,native,sent,_):
            self.prepare_captured();data=self.sign_data(self.state())
            with self.assertRaisesRegex(ValueError,'original_unknown_hold')as caught:
                self.call('SIGN_PHASE_ATTEMPT',data,_native_profile_path='/not-read/public-fixture',_submit_original=True)
            self.assertEqual(caught.exception.phaseTransportCapture['status'],'ORIGINAL_UNKNOWN_HOLD_NO_RETRY')
            self.assertTrue(caught.exception.nativeSignerResponse['completeStdout']);self.assertEqual((len(native),len(sent)),(1,1))
            self.tail=json.loads((self.path/'00000003.json').read_bytes())['digest'];before=self.hashes(self.path)
            self.assertTrue(self.call('INSPECT')['originalPending'])
            with self.assertRaisesRegex(ValueError,'original_boundary_only|prepared_funding_or_hold'):
                self.call('SIGN_PHASE_ATTEMPT',data,_native_profile_path='/not-read/public-fixture',_submit_original=True)
            with self.assertRaisesRegex(ValueError,'original_unknown_hold'):self.call('PHASE_RESULT')
            self.assertEqual(before,self.hashes(self.path));self.assertEqual((len(native),len(sent)),(1,1))
        with self.positive_checker(),self.boundaries()as(calls,native,sent,_):
            recovered=self.call('PHASE_RESULT');self.assertEqual(recovered['startupCursor'],1)
            self.assertEqual((native,sent),([],[]));self.assertFalse(recovered['signingOrSendPerformed'])
            self.assertEqual([batch[0]['method']for batch in calls],['getGenesisHash','getSignatureStatuses','getMultipleAccounts'])

    def test_fixed_journal_dispatch_default_and_independent_governed_anchor(self):
        oldpath=self.root/'unchanged.candidate-controller'
        o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'CREATE','expectedTail':None,'data':self.manifest},oldpath)
        old=o.Journal(self.j,oldpath,_candidate=True)
        try:
            with mock.patch.object(o,'_candidate_controller_replay',return_value={'fixedOld':True})as replay:
                self.assertEqual(old.replay(),{'fixedOld':True});replay.assert_called_once_with(self.j,old.chain,oldpath)
        finally:old.close()
        fixture_checker=self.j._node_request
        with self.positive_checker():
            self.assertIsInstance(self.j._node_request,mock.Mock)
            self.call('CREATE_HANDOFF',self.manifest)
            journal=o.Journal(self.j,self.path,_governed=True,_expected_handoff=self.anchor)
            try:
                with mock.patch.object(o,'_candidate_controller_replay',side_effect=AssertionError('old replay forbidden')):
                    self.assertEqual(journal.replay()['cursor'],0)
                journal.expected_handoff['tail']='e'*64
                with self.assertRaisesRegex(ValueError,'anchor_changed'):journal.replay()
                with self.assertRaisesRegex(ValueError,'anchor_changed'):journal.stable()
            finally:journal.close()
        self.assertIs(self.j._node_request,fixture_checker)

    def test_current_capture_parent_raw_anchor_and_retained_state_faults_preserve_evidence(self):
        # Four demonstrated callback fault classes, not a combinatorial matrix.
        for fault_name,reason in (('parent','parent_record_sha'),('anchor','anchor_changed'),('state','retained_changed'),('raw','raw_changed')):
            self.path=self.root/(fault_name+'.governed-startup');self.tail=None
            with self.positive_checker(),self.boundaries():self.prepare_captured()
            before=self.hashes(self.path);parent_raw=(self.parent/'00000003.json').read_bytes()
            def fault(kind,journal):
                if kind!='transport':return
                if fault_name=='parent':(self.parent/'00000003.json').write_bytes(parent_raw+b' ')
                elif fault_name=='anchor':journal.expected_handoff['tail']='e'*64
                elif fault_name=='state':journal.chain[0]['data']['manifestSha256']='e'*64
                else:(self.path/'00000000.json').write_bytes((self.path/'00000000.json').read_bytes()+b' ')
            with self.positive_checker(),self.boundaries(fault=fault),self.assertRaisesRegex(ValueError,reason)as caught:
                self.call('VERIFY_CURRENT_PHASE')
            self.assertFalse(caught.exception.publicCapture['tlsReceiptValidated'])
            self.assertIn('responseBase64',caught.exception.publicCapture['publicFrame'])
            if fault_name!='raw':self.assertEqual(before,self.hashes(self.path))
            (self.parent/'00000003.json').write_bytes(parent_raw)

    def test_current_source_fault_after_callback_keeps_raw_without_new_record(self):
        with self.positive_checker(),self.boundaries():self.prepare_captured()
        changed=[False];real=None
        def sources():
            a,j,s,domain,unchanged=real()
            def guard(boundary):
                unchanged(boundary)
                if changed[0]:raise ValueError('synthetic_governed_source_fault')
            return a,j,s,domain,guard
        def fault(kind,journal):
            if kind=='transport':changed[0]=True
        before=self.hashes(self.path)
        with self.positive_checker(),self.boundaries(fault=fault):
            real=o._candidate_preparation_sources
            with mock.patch.object(o,'_candidate_preparation_sources',side_effect=sources):
                with self.assertRaisesRegex(ValueError,'synthetic_governed_source_fault')as caught:self.call('VERIFY_CURRENT_PHASE')
        self.assertEqual(before,self.hashes(self.path));self.assertIn('responseBase64',caught.exception.publicCapture['publicFrame'])
        self.assertFalse(caught.exception.publicCapture['tlsReceiptValidated'])

    def test_successful_capture_then_final_parent_failure_keeps_both_input_evidence_types(self):
        actual=o._candidate_operate_phase;parent_raw=(self.parent/'00000003.json').read_bytes()
        for action,capture_name,flag,index in (('PREPARE_PHASE','phaseInputCapture','unsignedPreparationVerified',3),
                ('SIGN_PHASE_ATTEMPT','signAttemptInputCapture','currentInputVerified',4)):
            self.path=self.root/(action.lower()+'.governed-startup');self.tail=None;captured={}
            with self.positive_checker(),self.boundaries():
                self.call('CREATE_HANDOFF',self.manifest)
                if action=='SIGN_PHASE_ATTEMPT':
                    self.call('PREPARE_PHASE',{'feeCapLamports':self.manifest['phases'][0]['historicalFeeProjectionLamports']})
                    data=self.sign_data(self.state())
                else:data={'feeCapLamports':self.manifest['phases'][0]['historicalFeeProjectionLamports']}
                def completed(*args,**kwargs):
                    result=actual(*args,**kwargs);captured['evidence']=copy.deepcopy(result[index]);captured['files']=self.hashes(self.path)
                    (self.parent/'00000003.json').write_bytes(parent_raw+b' ')
                    return result
                with mock.patch.object(o,'_candidate_operate_phase',side_effect=completed):
                    with self.assertRaisesRegex(ValueError,'parent_record_sha')as caught:self.call(action,data)
                evidence=getattr(caught.exception,capture_name);self.assertFalse(evidence[flag])
                self.assertEqual(evidence['input'],captured['evidence']['input']);self.assertEqual(captured['files'],self.hashes(self.path))
                if action=='PREPARE_PHASE':self.assertEqual(evidence['frames'],captured['evidence']['frames'])
                else:
                    self.assertEqual(evidence['currentCapture']['publicFrame'],captured['evidence']['currentCapture']['publicFrame'])
                    self.assertFalse(evidence['currentCapture']['publicCapture']['tlsReceiptValidated'])
                self.assertEqual(len(captured['files']),3 if action=='PREPARE_PHASE'else 4)
                (self.parent/'00000003.json').write_bytes(parent_raw)
                self.tail=json.loads((self.path/('00000001.json'if action=='PREPARE_PHASE'else '00000002.json')).read_bytes())['digest']
                retained=self.call('INSPECT');self.assertTrue(retained['phasePreparationRetained'])
                self.assertEqual(retained['phaseSignAttemptRetained'],action=='SIGN_PHASE_ATTEMPT')
                self.assertEqual(captured['files'],self.hashes(self.path))

    def test_uncertain_sign_commit_preserves_hold_and_never_invokes_native(self):
        with self.positive_checker(),self.boundaries()as(_,native,sent,_):
            self.prepare_captured();data=self.sign_data(self.state());write=os.write;target=str(self.path/'00000002.json')
            def interrupted(fd,raw):
                if os.readlink(f'/proc/self/fd/{fd}')==target:
                    write(fd,raw[:23]);raise OSError('synthetic sign commit uncertainty')
                return write(fd,raw)
            with mock.patch.object(os,'write',side_effect=interrupted),self.assertRaisesRegex(ValueError,'commit_uncertain_no_retry_or_adoption'):
                self.call('SIGN_PHASE_ATTEMPT',data,_native_profile_path='/not-read/public-fixture',_submit_original=True)
            self.assertEqual((native,sent),([],[]));self.assertEqual((self.path/'00000002.json').stat().st_size,23)
            before=self.hashes(self.path)
            with self.assertRaises(ValueError):self.call('INSPECT')
            self.assertEqual(before,self.hashes(self.path))

    def test_native_postcallback_parent_fault_retains_raw_and_reservation_without_send(self):
        with self.positive_checker(),self.boundaries():
            self.prepare_captured();data=self.sign_data(self.state())
        raw=(self.parent/'00000003.json').read_bytes()
        def fault(kind,journal):
            if kind=='native':(self.parent/'00000003.json').write_bytes(raw+b' ')
        with self.positive_checker(),self.boundaries(fault=fault)as(_,native,sent,_):
            with self.assertRaisesRegex(ValueError,'parent_record_sha')as caught:
                self.call('SIGN_PHASE_ATTEMPT',data,_native_profile_path='/not-read/public-fixture',_submit_original=True)
            self.assertTrue(caught.exception.nativeSignerResponse['completeStdout']);self.assertEqual((len(native),sent),(1,[]))
        (self.parent/'00000003.json').write_bytes(raw)
        self.tail=json.loads((self.path/'00000002.json').read_bytes())['digest']
        with self.positive_checker(),self.boundaries()as(_,native,sent,_):
            self.assertTrue(self.call('INSPECT')['phaseSignAttemptRetained'])
            with self.assertRaisesRegex(ValueError,'prepared_funding_or_hold|original_boundary_only'):
                self.call('SIGN_PHASE_ATTEMPT',data,_native_profile_path='/not-read/public-fixture',_submit_original=True)
            self.assertEqual((native,sent),([],[]))


class CandidateGovernedBufferPhaseTests(unittest.TestCase):
    """One phase of the private Buffer route; all positive I/O is simulated."""
    setUpClass=classmethod(CandidateGovernedFundingHandoffTests.setUpClass.__func__)
    setUp=CandidateGovernedFundingHandoffTests.setUp
    build_parent=CandidateGovernedFundingHandoffTests.build_parent
    positive_checker=CandidateGovernedFundingHandoffTests.positive_checker
    call=CandidateGovernedFundingHandoffTests.call
    hashes=CandidateGovernedFundingHandoffTests.hashes
    frame=CandidateGovernedFundingHandoffTests.frame
    preparation=CandidateGovernedFundingHandoffTests.preparation
    state=CandidateGovernedLifecycleTests.state
    boundaries=CandidateGovernedLifecycleTests.boundaries

    def buffer_request(self):
        digest=o.sha(o.encode(self.manifest))
        authorization={'schema':'piv1-launcher-release-authorization-v2','manifestSha256':digest,'baseParentTail':None,
            'operatorSourceSha256':o.sha(Path(o.__file__).read_bytes()),'expiresUnixTimestamp':'18446744073709551615',
            'scope':'EXACT_BOUNDED_TESTNET_RELEASE','feeBudgetNamespace':'piv1-testnet-startup-release-'+digest,
            'signerBackendSha256':o.SIGNER_BACKEND_SHA,'signerProfileSha256':'c'*64}
        raw=o.encode(authorization)
        return {'schema':o.CANDIDATE_GOVERNED_SCHEMA,'expectedTail':self.tail,
            'feeCapLamports':self.manifest['phases'][0]['historicalFeeProjectionLamports'],
            'authorizationRawBase64':base64.b64encode(raw).decode(),'authorizationSha256':o.sha(raw)}

    def buffer(self,*,submit=True):
        result=o._candidate_governed_buffer_phase(self.buffer_request(),self.path,expected_handoff=self.anchor,
            _native_profile_path='/not-read/public-fixture',_submit_original=submit)
        self.tail=result['tail'];return result

    def test_one_full_replay_before_original_and_new_record_only_reduction_matches_old_replay(self):
        with self.positive_checker(),self.boundaries()as(calls,native,sent,active):
            self.call('CREATE_HANDOFF',self.manifest);events=[]
            full=o._candidate_governed_replay;reduce=o._candidate_replay_phase_record;transport=o.public_rpc_transport
            def replay(*args):events.append('FULL');return full(*args)
            def reduced(*args):events.append(args[4]['kind']);return reduce(*args)
            def observed(c,raw):
                methods=[row['method']for row in c.strict_json(raw)];events.append(tuple(methods))
                return transport(c,raw)
            with mock.patch.object(o,'_candidate_governed_replay',side_effect=replay),\
                    mock.patch.object(o,'_candidate_replay_phase_record',side_effect=reduced),\
                    mock.patch.object(o,'public_rpc_transport',side_effect=observed):
                result=self.buffer()
                self.assertEqual(events.count('FULL'),1)
                original=events.index(('getGenesisHash','getLatestBlockhash'))
                self.assertLess(events.index('FULL'),original)
                first_current=events.index(('getGenesisHash','getFeeForMessage','getMultipleAccounts','isBlockhashValid','getBlockHeight'))
                self.assertEqual(events.count('PREPARE_PHASE'),1);self.assertNotIn('PREPARE_PHASE',events[first_current:])
                for kind in ('SIGN_PHASE_ATTEMPT','PHASE_INTENT','PHASE_RESULT'):self.assertEqual(events.count(kind),1)
                self.assertIsNone(active['journal']._buffer_context)
            self.assertEqual((result['startupCursor'],len(native),len(sent)),(1,1,1))
            self.assertFalse(result['originalPending']);self.assertFalse(result['liveReady'])
            self.assertTrue(result['phaseInputCapture']['unsignedPreparationVerified'])
            self.assertTrue(result['signAttemptInputCapture']['currentInputVerified'])
            records=[json.loads((self.path/f'{i:08}.json').read_bytes())for i in range(5)]
            self.assertEqual([row['kind']for row in records],['CREATE_HANDOFF','PREPARE_PHASE','SIGN_PHASE_ATTEMPT','PHASE_INTENT','PHASE_RESULT'])
            old=full(self.j,records,self.path,self.anchor)
            self.assertEqual(old['cursor'],result['startupCursor']);self.assertIsNone(old['pending'])
            self.assertEqual(old['lastSlot'],result['phaseTransportCapture']['recovery']['summary']['slot'])

    def test_private_snapshot_has_no_mutable_alias_and_fixed_kind_guard_stops_before_capture(self):
        with self.positive_checker():
            self.call('CREATE_HANDOFF',self.manifest)
            journal=o.Journal(self.j,self.path,_governed=True,_expected_handoff=self.anchor)
            try:
                journal._begin_buffer_phase();first=journal.replay();first['manifest']['phases'][0]['index']=935
                self.assertEqual(journal.replay()['manifest']['phases'][0]['index'],0)
                with self.assertRaisesRegex(ValueError,'operation_scope'):journal._begin_buffer_phase()
                self.assertTrue(journal._buffer_failed)
                with self.assertRaisesRegex(ValueError,'operation_consumed'):journal.append('PREPARE_PHASE',{})
            finally:journal.close()
        # Evaluate the actual fixed guard with the real phase selector. This is
        # boundary evidence for Write/935, not a fabricated 934-Write execution.
        tree=ast.parse(Path(o.__file__).read_bytes());journal=next(n for n in tree.body if isinstance(n,ast.ClassDef)and n.name=='Journal')
        begin=next(n for n in journal.body if isinstance(n,ast.FunctionDef)and n.name=='_begin_buffer_phase')
        call,=[n for n in ast.walk(begin)if isinstance(n,ast.Call)and isinstance(n.func,ast.Name)and n.func.id=='need'
            and len(n.args)==2 and isinstance(n.args[1],ast.Constant)and n.args[1].value=='candidate_buffer_phase_only']
        predicate=compile(ast.Expression(call.args[0]),'<actual-buffer-kind-boundary>','eval')
        for index,allowed in ((0,True),(1,True),(934,True),(935,False),(936,False)):
            self.assertIs(eval(predicate,{'type':type,'int':int},{'index':index,'a':self.a}),allowed)
        capture=next(n for n in ast.walk(begin)if isinstance(n,ast.Assign)and any(isinstance(t,ast.Attribute)and t.attr=='_buffer_context'for t in n.targets))
        self.assertLess(call.lineno,capture.lineno)

    def test_native_callback_raw_sidecar_snapshot_and_anchor_faults_preserve_hold(self):
        for name,reason in (('raw','raw_changed'),('sidecar','payload_hash'),('snapshot','prefix_changed'),('anchor','anchor_changed')):
            self.path=self.root/(name+'.governed-startup');self.tail=None
            with self.positive_checker():self.call('CREATE_HANDOFF',self.manifest)
            def fault(kind,journal):
                if kind!='native':return
                if name=='raw':
                    path=self.path/'00000001.json';path.write_bytes(path.read_bytes()+b' ')
                elif name=='sidecar':
                    ref=journal.chain[1]['data']['evidence'];path=self.path.with_name(self.path.name+'.evidence')/ref['name']/'payload.json'
                    raw=bytearray(path.read_bytes());raw[-2]^=1;path.write_bytes(bytes(raw))
                elif name=='snapshot':
                    context=list(journal._buffer_context);state=journal._buffer_decode_state(context[5]);state['cursor']=1;context[5]=o.encode(state)
                    journal._buffer_context=tuple(context)
                else:journal.expected_handoff['tail']='e'*64
            with self.subTest(fault=name),self.positive_checker(),self.boundaries(fault=fault)as(_,native,sent,active):
                with self.assertRaisesRegex(ValueError,reason)as caught:self.buffer()
                self.assertEqual((len(native),sent),(1,[]));self.assertTrue(caught.exception.nativeSignerResponse['completeStdout'])
                self.assertTrue(active['journal']._buffer_failed);self.assertIsNone(active['journal']._buffer_context)
                self.assertTrue((self.path/'00000002.json').exists());self.assertFalse((self.path/'00000003.json').exists())

    def test_internal_aggregate_clone_exceeds_record_bound_without_relaxing_external_limits(self):
        raw=o.encode({'payload':'A'*(self.j.MAX_INPUT+1)})
        self.assertGreater(len(raw),self.j.MAX_INPUT)
        copied=o.Journal._buffer_decode_state(raw)
        self.assertEqual(len(copied['payload']),self.j.MAX_INPUT+1)
        with self.assertRaisesRegex(ValueError,'json_bound'):self.j.parse(raw)
        with mock.patch.object(o,'MAX_JOURNAL',len(raw)-1),self.assertRaisesRegex(ValueError,'state_bound'):
            o.Journal._buffer_decode_state(raw)

    def test_source_fault_after_original_callback_retains_raw_before_any_new_commit(self):
        with self.positive_checker():self.call('CREATE_HANDOFF',self.manifest)
        changed=[False];factory=None;before=self.hashes(self.path)
        def sources():
            a,j,s,domain,unchanged=factory()
            def guard(boundary):
                unchanged(boundary)
                if changed[0]:raise ValueError('synthetic_buffer_source_fault')
            return a,j,s,domain,guard
        def fault(kind,journal):
            if kind=='transport':changed[0]=True
        with self.positive_checker(),self.boundaries(fault=fault)as(_,native,sent,active):
            factory=o._candidate_preparation_sources
            with mock.patch.object(o,'_candidate_preparation_sources',side_effect=sources):
                with self.assertRaisesRegex(ValueError,'synthetic_buffer_source_fault')as caught:self.buffer()
                self.assertEqual((native,sent),([],[]));self.assertEqual(before,self.hashes(self.path))
                self.assertIn('responseBase64',caught.exception.phaseInputCapture['frames']['originalBlockhash'])
                self.assertFalse(caught.exception.phaseInputCapture['unsignedPreparationVerified']);self.assertTrue(active['journal']._buffer_failed)

    def test_uncertain_reservation_commit_never_invokes_native_or_reuses_context(self):
        with self.positive_checker(),self.boundaries()as(_,native,sent,active):
            self.call('CREATE_HANDOFF',self.manifest);write=os.write;target=str(self.path/'00000002.json')
            def interrupted(fd,raw):
                if os.readlink(f'/proc/self/fd/{fd}')==target:
                    write(fd,raw[:23]);raise OSError('synthetic buffer reservation uncertainty')
                return write(fd,raw)
            with mock.patch.object(os,'write',side_effect=interrupted),self.assertRaisesRegex(ValueError,'commit_uncertain_no_retry_or_adoption'):
                self.buffer()
            self.assertEqual((native,sent),([],[]));self.assertTrue(active['journal']._buffer_failed)
            self.assertIsNone(active['journal']._buffer_context);self.assertEqual((self.path/'00000002.json').stat().st_size,23)
            before=self.hashes(self.path)
            with self.assertRaises(ValueError):self.call('INSPECT')
            self.assertEqual(before,self.hashes(self.path))

    def test_durable_transition_context_raw_and_source_faults_consume_before_continuation(self):
        for fault,reason in (('context','candidate_buffer_commit_changed'),('raw','candidate_controller_commit_readback'),
                ('source','synthetic_post_write_source')):
            self.path=self.root/(fault+'.governed-startup');self.tail=None
            with self.positive_checker():self.call('CREATE_HANDOFF',self.manifest)
            changed=[False];fsync=os.fsync;factory=None;target=str(self.path/'00000001.json')
            def sources():
                a,j,s,domain,unchanged=factory()
                def check(boundary):
                    unchanged(boundary)
                    if changed[0]and fault=='source':raise o.Stop('synthetic_post_write_source')
                return a,j,s,domain,check
            def durable(fd):
                fsync(fd)
                if not changed[0]and os.readlink(f'/proc/self/fd/{fd}')==target:
                    changed[0]=True;journal=active['journal']
                    if fault=='context':journal._buffer_context=tuple(list(journal._buffer_context))
                    elif fault=='raw':
                        path=Path(target);path.write_bytes(path.read_bytes()+b' ')
            with self.subTest(fault=fault),self.positive_checker(),self.boundaries()as(_,native,sent,active):
                factory=o._candidate_preparation_sources
                with mock.patch.object(o,'_candidate_preparation_sources',side_effect=sources),mock.patch.object(os,'fsync',side_effect=durable):
                    with self.assertRaisesRegex(ValueError,'commit_uncertain_no_retry_or_adoption')as caught:self.buffer()
                self.assertIn(reason,str(caught.exception.__cause__))
                self.assertTrue(changed[0]);self.assertEqual((native,sent),([],[]));self.assertTrue(active['journal']._buffer_failed)
                self.assertIsNone(active['journal']._buffer_context)
                self.assertTrue(Path(target).exists());self.assertFalse((self.path/'00000002.json').exists())
                before=self.hashes(self.path)
                with self.assertRaisesRegex(ValueError,'operation_consumed'):active['journal'].append('PREPARE_PHASE',{})
                self.assertEqual(before,self.hashes(self.path))

    def test_authorization_expiry_during_reservation_durability_blocks_fresh_native_gate(self):
        with self.positive_checker(),self.boundaries()as(_,native,sent,active):
            self.call('CREATE_HANDOFF',self.manifest);request=self.buffer_request()
            authorization=json.loads(base64.b64decode(request['authorizationRawBase64']));expires=int(o.time.time())+60
            authorization['expiresUnixTimestamp']=str(expires);raw=o.encode(authorization)
            request.update(authorizationRawBase64=base64.b64encode(raw).decode(),authorizationSha256=o.sha(raw))
            now=[expires-1];fsync=os.fsync;target=str(self.path/'00000002.json')
            def durable(fd):
                fsync(fd)
                if os.readlink(f'/proc/self/fd/{fd}')==target:now[0]=expires
            with mock.patch.object(o.time,'time',side_effect=lambda:now[0]),mock.patch.object(os,'fsync',side_effect=durable):
                with self.assertRaisesRegex(ValueError,'launcher_authorization_expired'):
                    o._candidate_governed_buffer_phase(request,self.path,expected_handoff=self.anchor,
                        _native_profile_path='/not-read/public-fixture',_submit_original=True)
            self.assertEqual(now[0],expires);self.assertEqual((native,sent),([],[]));self.assertTrue(active['journal']._buffer_failed)
            self.assertTrue(Path(target).exists());self.assertFalse((self.path/'00000003.json').exists())

    def test_unknown_original_reopen_is_hold_and_existing_original_recovery_only(self):
        with self.positive_checker(),self.boundaries(unknown=True)as(_,native,sent,_):
            self.call('CREATE_HANDOFF',self.manifest)
            with self.assertRaisesRegex(ValueError,'original_unknown_hold'):self.buffer()
            self.assertEqual((len(native),len(sent)),(1,1));self.tail=json.loads((self.path/'00000003.json').read_bytes())['digest']
            before=self.hashes(self.path)
        with self.positive_checker(),self.boundaries()as(calls,native,sent,_):
            with self.assertRaisesRegex(ValueError,'buffer_original_hold'):self.buffer()
            self.assertEqual((calls,native,sent),([],[],[]));self.assertEqual(before,self.hashes(self.path))
            result=self.call('PHASE_RESULT');self.assertEqual(result['startupCursor'],1)
            self.assertEqual((native,sent),([],[]));self.assertEqual([batch[0]['method']for batch in calls],
                ['getGenesisHash','getSignatureStatuses','getMultipleAccounts'])


class CandidateFundingControllerTests(unittest.TestCase):
    """New temporary host chains; no signed funding positive or signature mocks."""
    @classmethod
    def setUpClass(cls):
        from tools.test_startup_manifest import CandidateFundingReceiptTests
        cls.host=CandidateFundingReceiptTests;cls.host.setUpClass()
        cls.a=o.startup_manifest();cls.manifest=cls.host.manifest;_,_,cls.c=cls.a._candidate_helpers()
        cls.j=cls.a._candidate_helpers()[0]

    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
        self.path=Path(self.tmp.name)/'funding.candidate-controller';self.tail=None

    def call(self,action,data=None,**kwargs):
        result=o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':action,'expectedTail':self.tail,'data':data},self.path,**kwargs)
        self.tail=result['tail'];return result

    def hashes(self):return {p.name:o.sha(p.read_bytes()) for p in self.path.iterdir()}
    def intent(self,receipt):return {k:receipt[k] for k in ('wireBase64','feeCapLamports','originalBlockhash','preflight')}

    def test_actual_operate_create_exact_readback_cas_and_host_scope(self):
        created=self.call('CREATE',self.manifest);before=self.hashes()
        self.assertEqual(self.call('INSPECT')['tail'],created['tail']);self.assertEqual(before,self.hashes())
        self.assertTrue(all(created[k] is False for k in o.CANDIDATE_PREPARATION_FLAGS if k!='journalMutationPerformed'))
        self.assertTrue(created['journalMutationPerformed']);self.assertTrue(created['hostControllerJournalMutationPerformed'])
        self.assertFalse(created['operationalJournalAccessed']);self.assertFalse(self.call('INSPECT')['journalMutationPerformed'])
        self.assertFalse(created['startupPhaseProgressionImplemented']);self.assertFalse(created['originalPacketSignatureVerified'])
        record=json.loads((self.path/'00000000.json').read_bytes())
        self.assertEqual(record['schema'],o.CANDIDATE_CONTROLLER_SCHEMA)
        self.assertEqual(record['data']['sourceDomain']['tools/startup_release_operator.py'],o.sha(Path(o.__file__).read_bytes()))
        self.assertEqual(record['data']['historicalUncertaintyReference'],self.manifest['historicalUncertainty'])
        with self.assertRaisesRegex(ValueError,'tail_compare_and_swap'):
            o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'INSPECT','expectedTail':None,'data':None},self.path)
        with self.assertRaises(FileExistsError):self.call('CREATE',self.manifest)
        self.assertEqual(before,self.hashes())

    def test_original_host_intent_persists_hold_and_refuses_replacement_or_progression(self):
        self.call('CREATE',self.manifest);receipt,_,_=self.host.fixture()
        held=self.call('FUNDING_INTENT',self.intent(receipt));before=self.hashes()
        self.assertEqual(held['status'],'UNKNOWN_HOLD_ORIGINAL_FUNDING_ONLY')
        self.assertEqual(held['originalFundingIdentity']['signatureHex'],'00'*64)
        self.assertEqual(self.call('INSPECT')['originalFundingIdentity'],held['originalFundingIdentity'])
        with self.assertRaisesRegex(ValueError,'original_intent_only'):self.call('FUNDING_INTENT',self.intent(receipt))
        with self.assertRaisesRegex(ValueError,'funding_unknown_hold'):self.call('VERIFY',{'accounts':{},'contextSlot':'0'})
        with self.assertRaisesRegex(ValueError,'object_fields'):
            self.call('VERIFY',{'accounts':{},'contextSlot':'0','candidateFunding':{'receipt':receipt}})
        self.assertEqual(before,self.hashes());self.assertIsNone(held['fundingReceiptSha256'])

    def test_result_without_intent_and_real_crypto_failures_never_append_or_credit(self):
        self.call('CREATE',self.manifest);receipt,_,_=self.host.fixture()
        data={'receipt':receipt,'expectedReceiptSha256':o.sha(o.encode(receipt))};before=self.hashes()
        with self.assertRaisesRegex(ValueError,'result_requires_original_intent'):self.call('FUNDING_RESULT',data)
        self.assertEqual(before,self.hashes())
        for signature,reason in ((bytes(64),'zero_signature'),(bytes([1])*64,'signature_invalid')):
            self.path=Path(self.tmp.name)/(reason+'.candidate-controller');self.tail=None
            self.call('CREATE',self.manifest);receipt,_,_=self.host.fixture(signature)
            self.call('FUNDING_INTENT',self.intent(receipt));before=self.hashes();tail=self.tail
            with self.assertRaisesRegex(ValueError,'signature_or_receipt:'+reason):
                self.call('FUNDING_RESULT',{'receipt':receipt,'expectedReceiptSha256':o.sha(o.encode(receipt))})
            self.assertEqual(self.tail,tail);self.assertEqual(before,self.hashes())
            self.assertEqual(self.call('INSPECT')['status'],'UNKNOWN_HOLD_ORIGINAL_FUNDING_ONLY')
            bad=copy.deepcopy(receipt);bad['feeCapLamports']='10001'
            with self.assertRaisesRegex(ValueError,'original_result_only'):
                self.call('FUNDING_RESULT',{'receipt':bad,'expectedReceiptSha256':o.sha(o.encode(bad))})
            self.assertEqual(before,self.hashes())

    def test_source_domain_and_partial_or_extra_namespaces_refuse_without_repair(self):
        self.call('CREATE',self.manifest);path=self.path/'00000000.json';record=json.loads(path.read_bytes())
        record['data']['sourceDomain']['tools/startup_release_operator.py']='0'*64
        record['digest']=o.sha(o.encode({k:v for k,v in record.items() if k!='digest'}));path.write_bytes(o.encode(record))
        self.tail=record['digest']  # Pass CAS to exercise the independent source-domain guard.
        before=self.hashes()
        with self.assertRaisesRegex(ValueError,'create_binding'):self.call('INSPECT')
        self.assertEqual(before,self.hashes())
        self.path=Path(self.tmp.name)/'partial.candidate-controller';self.tail=None;fd=self.j.directory(self.path,True)
        try:self.j.write_file(fd,'lock',o.CANDIDATE_CONTROLLER_LOCK);self.j.write_file(fd,'00000000.json',b'{')
        finally:os.close(fd)
        before=self.hashes()
        with self.assertRaises(ValueError):self.call('INSPECT')
        self.assertEqual(before,self.hashes())
        self.path=Path(self.tmp.name)/'extra.candidate-controller';self.tail=None;self.call('CREATE',self.manifest)
        (self.path/'extra.json').write_bytes(b'{}\n');before=self.hashes()
        with self.assertRaisesRegex(ValueError,'operator_namespace'):self.call('INSPECT')
        self.assertEqual(before,self.hashes())

    def test_candidate_directory_identity_and_closed_journal_mode(self):
        self.call('CREATE',self.manifest);before=self.hashes()
        with self.assertRaisesRegex(ValueError,'journal_selector'):o.Journal(self.j,self.path,_candidate=1)
        journal=o.Journal(self.j,self.path,_candidate=True)
        try:
            with self.assertRaisesRegex(ValueError,'signer_attempt_forbidden'):
                journal.append('FUNDING_RESULT',{},expected_signer_attempt={})
            moved=self.path.with_name('preserved-original');self.path.rename(moved);self.path.mkdir(mode=0o700)
            with self.assertRaisesRegex(ValueError,'directory_changed'):journal.stable()
            self.assertEqual(before,{p.name:o.sha(p.read_bytes()) for p in moved.iterdir()})
        finally:journal.close()

    def test_append_detaches_caller_data_before_real_prospective_replay(self):
        self.call('CREATE',self.manifest);receipt,_,_=self.host.fixture()
        data=o._candidate_controller_intent(self.a,self.j,self.manifest,self.intent(receipt))
        real=o._candidate_controller_replay
        def after_capture(j,records,path):
            data['identity']['signatureHex']='11'*64  # Only this synthetic caller object changes.
            return real(j,records,path)
        journal=o.Journal(self.j,self.path,_candidate=True)
        try:
            with mock.patch.object(o,'_candidate_controller_replay',side_effect=after_capture):
                record=journal.append('FUNDING_INTENT',data)
            self.tail=record['digest']
            self.assertEqual(record['data']['identity']['signatureHex'],'00'*64)
            self.assertEqual(journal.chain[-1],record)
            self.assertEqual((self.path/'00000001.json').read_bytes(),o.encode(record))
        finally:journal.close()
        self.assertEqual(self.call('INSPECT')['originalFundingIdentity']['signatureHex'],'00'*64)

    def test_live_auth_signer_transport_cli_and_output_overrides_refuse_before_create(self):
        for kwargs in ({'live':True},{'live_sign':True},{'authorization_raw':b'{}'},
                {'authorization_sha256':'0'*64},{'_transport':lambda x:x},{'_signer':lambda x:x},{'output_mode':'COMMIT_RECEIPT'}):
            with self.assertRaisesRegex(ValueError,'offline_only'):self.call('CREATE',self.manifest,**kwargs)
            self.assertFalse(self.path.exists())
        with mock.patch.object(o,'__name__','__main__'):
            with self.assertRaisesRegex(ValueError,'import_only'):self.call('CREATE',self.manifest)
        self.assertFalse(self.path.exists())


class LegacyJournalDefaultTests(unittest.TestCase):
    """Secure legacy record plumbing only, not operational manifest/parent admission."""
    def test_real_default_append_replay_readback_and_lock_without_legacy_helper_bypass(self):
        j=o.startup_manifest()._candidate_helpers()[0]  # Real current pinned source/primitives.
        with tempfile.TemporaryDirectory() as temporary:
            base=Path(temporary)/'synthetic-never-opened-parent'
            manifest={'schema':o.SCHEMA,'historicalLedger':{},'phases':[]}
            data={'manifest':manifest,'baseParentPath':str(base),'historicalLedger':{},'contextFloor':'0','lastFinalizedSlot':'0'}
            journal=o.Journal(j,base,True)
            try:
                self.assertFalse(journal.candidate);self.assertEqual(journal.path,o.journal_path(base))
                record=journal.append('CREATE',data);tail=journal.tail()
                self.assertEqual(record['schema'],o.SCHEMA)
                self.assertEqual(j.read_file(journal.fd,'lock',128),o.LOCK)
                self.assertEqual(j.read_file(journal.fd,'00000000.json',j.MAX_INPUT),o.encode(record))
                self.assertEqual(o.replay(j,journal.chain)['manifest'],manifest)
                before={p.name:p.read_bytes() for p in journal.path.iterdir()}
                with self.assertRaisesRegex(ValueError,'operator_record_kind'):journal.append('CREATE',data)
                self.assertEqual(before,{p.name:p.read_bytes() for p in journal.path.iterdir()})
            finally:journal.close()
            journal=o.Journal(j,base)
            try:
                self.assertEqual(journal.tail(),tail);self.assertEqual(o.replay(j,journal.chain)['cursor'],0)
                self.assertFalse(base.exists())
                (journal.path/'lock').write_bytes(b'synthetic-invalid-lock\n')
                with self.assertRaisesRegex(ValueError,'stable_lock'):journal.stable()
            finally:journal.close()


class CandidatePrefixProgressionTests(unittest.TestCase):
    """Pure geometry/storage/semantics and actual unfunded-controller refusals only."""
    @classmethod
    def setUpClass(cls):
        from tools.test_startup_manifest import CandidateFundingReceiptTests
        cls.host=CandidateFundingReceiptTests;cls.host.setUpClass()
        cls.a=o.startup_manifest();cls.manifest=cls.host.manifest;cls.j,_,cls.c=cls.a._candidate_helpers()

    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
        self.path=Path(self.tmp.name)/'prefix.candidate-controller';self.tail=None

    def call(self,action,data=None):
        out=o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':action,'expectedTail':self.tail,'data':data},self.path)
        self.tail=out['tail'];return out

    def test_fixed_payload_geometry_keeps_existing_journal_evidence_and_reserve_caps(self):
        self.assertEqual(o.CANDIDATE_PREFIX_RECORDS,1873)
        self.assertLess(o._candidate_prefix_remaining(0),4*1024**3)
        self.assertEqual(o._candidate_prefix_remaining(935),0)
        self.assertEqual(o.MAX_JOURNAL,128*1024**2);self.assertEqual(o.MAX_EVIDENCE,8*1024**3)
        self.assertEqual(self.j.RESERVE,2*1024**3)
        sample={'evidence':{'name':'00001872-result','bytes':2*1024**2,'sha256':'f'*64,'rootIdentity':{'device':'1','inode':'2'}},
            'derived':{'index':934,'slot':'999999999','wireSha256':'f'*64,'clock':self.manifest['baselineContextSlot']}}
        self.assertLess(2*1870*len(o.encode(sample))+len(o.encode(self.manifest)),o.MAX_JOURNAL)
        for value in (True,-1,935):
            with self.assertRaises(ValueError):o._candidate_prefix_bound(value)

    def test_actual_fixed_prefix_reviews_allow_only_original_hash_substitution(self):
        blockhash=self.a.candidate_oracle()['roles']['vault0']  # Synthetic public32B hash, never live.
        for index in (0,1,934):
            review=o._candidate_prefix_review(self.a,self.c,self.manifest,index,blockhash)
            self.assertEqual(o.semantic(review),o.semantic(self.manifest['phases'][index]['reviewed']))
            self.assertEqual(review['requiredSigners'],self.manifest['phases'][index]['reviewed']['requiredSigners'])
        self.assertEqual(self.manifest['phases'][934]['writeBytes'],404)

    def test_prefix_raw_frame_accepts_full_buffer_and_rejects_funding_sized_or_changed_queries(self):
        queries=[('getMultipleAccounts',[[],{'encoding':'base64','commitment':'finalized'}])]
        result={'dataBase64':base64.b64encode(bytes(892389)).decode()}
        request=[{'jsonrpc':'2.0','id':1,'method':queries[0][0],'params':queries[0][1]}]
        frame={k:base64.b64encode(self.c.encode(v)).decode() for k,v in
            (('requestBase64',request),('responseBase64',[{'jsonrpc':'2.0','id':1,'result':result}]),('httpBase64',{'status':200,'headers':[]}))}
        self.assertEqual(o._candidate_prefix_frame(self.c,frame,queries,1),[result])
        with self.assertRaisesRegex(ValueError,'base64_bound'):self.a._candidate_funding_frame(self.c,frame,queries)
        bad=copy.deepcopy(frame);bad['requestBase64']=base64.b64encode(b'[]\n').decode()
        with self.assertRaisesRegex(ValueError,'raw_request'):o._candidate_prefix_frame(self.c,bad,queries,1)

    def test_real_store_reference_hash_and_orphan_stop_are_not_funded_progression(self):
        self.call('CREATE',self.manifest);journal=o.Journal(self.j,self.path,_candidate=True)
        try:
            raw=o.encode({'publicSyntheticPayload':'storage-only-not-a-phase-proof'})
            ref=o._candidate_prefix_store(self.c,self.j,journal,{'cursor':0},'PHASE_INTENT',raw)
            self.assertEqual(o._candidate_prefix_load(self.j,self.path,ref,'1','PHASE_INTENT',0),json.loads(raw))
            self.assertEqual(o._candidate_prefix_inventory(self.j,self.path,[ref]),len(raw))
            before=(self.path.with_name(self.path.name+'.evidence')/ref['name']/'payload.json').read_bytes()
        finally:journal.close()
        with self.assertRaisesRegex(ValueError,'orphan_uncertain_stop'):self.call('INSPECT')
        self.assertEqual((self.path.with_name(self.path.name+'.evidence')/ref['name']/'payload.json').read_bytes(),before)

    def test_named_child_replacement_during_parse_refuses_after_real_read(self):
        self.call('CREATE',self.manifest);journal=o.Journal(self.j,self.path,_candidate=True)
        try:
            raw=o.encode({'synthetic':'read-fault-only'})
            ref=o._candidate_prefix_store(self.c,self.j,journal,{'cursor':0},'PHASE_INTENT',raw)
            root=self.path.with_name(self.path.name+'.evidence');child=root/ref['name'];saved=root/'preserved-original'
            real=self.j.parse
            def changed(data):
                value=real(data);child.rename(saved);child.symlink_to(saved,target_is_directory=True);return value
            with mock.patch.object(self.j,'parse',side_effect=changed):
                with self.assertRaisesRegex(ValueError,'evidence_directory_changed'):
                    o._candidate_prefix_load(self.j,self.path,ref,'1','PHASE_INTENT',0)
            self.assertEqual((saved/'payload.json').read_bytes(),raw)
        finally:journal.close()

    def test_actual_operate_phase_actions_cannot_bypass_original_funding_hold(self):
        created=self.call('CREATE',self.manifest)
        self.assertTrue(created['loaderPrefixProgressionImplemented']);self.assertEqual(created['loaderPrefixCursor'],0)
        self.assertFalse(created['startupPhaseProgressionImplemented'])
        for action in ('PHASE_INTENT','PHASE_RESULT'):
            with self.assertRaisesRegex(ValueError,'prefix_funding_required'):self.call(action,{})
        receipt,_,_=self.host.fixture();intent={k:receipt[k] for k in ('wireBase64','feeCapLamports','originalBlockhash','preflight')}
        self.call('FUNDING_INTENT',intent);tail=self.tail
        with self.assertRaisesRegex(ValueError,'prefix_funding_required'):self.call('PHASE_INTENT',{'cursor':934,'funding':True})
        self.assertEqual(self.tail,tail);self.assertFalse(self.path.with_name(self.path.name+'.evidence').exists())
        self.assertEqual(self.call('INSPECT')['loaderPrefixCursor'],0)


class CandidateRuntimeProgressionTests(unittest.TestCase):
    """Actual pinned unsigned decoding; synthetic metadata, never funded progression."""
    @classmethod
    def setUpClass(cls):
        from tools.test_startup_manifest import CandidateAccountVerificationTests
        cls.host=CandidateAccountVerificationTests;cls.host.setUpClass()
        cls.a=cls.host.a;cls.ref=cls.host.ref;cls.j,_,cls.c=cls.a._candidate_helpers();cls.manifest=cls.host.manifest
        cls.binding=cls.j.candidate_clock_binding(cls.ref['preManifest'],cls.ref['clock'])
        cls.packets=[]
        for i in range(935,947):
            if 937<=i<=945:
                plan=cls.binding['unsignedInitializerPlan'];packet=plan['packets'][i-937]
                wire=packet['wireBase64'];tables=[{'address':plan['alt']['address'],'addresses':plan['alt']['addresses']}]if packet['version']==0 else []
            else:
                review=cls.manifest['phases'][i]['reviewed'];count=len(review['requiredSigners'])
                wire=base64.b64encode(bytes([count])+bytes(64*count)+base64.b64decode(review['messageBase64'])).decode();tables=[]
            cls.packets.append({'wireBase64':wire,'lookupTables':tables})
        cls.decoded=o.describe(cls.j,cls.packets,_runtime_metadata=True)

    def pending(self,index):
        decoded=self.decoded[index-935]
        return {'reviewed':decoded['reviewed'],'metadata':decoded,'binding':self.binding,
            'preAccounts':copy.deepcopy(self.host.original)}

    def test_real_default_decode_equivalence_and_closed_metadata_mode(self):
        self.assertEqual(o.describe(self.j,self.packets),[x['reviewed']for x in self.decoded])
        for index in (935,936,946):self.assertEqual(self.decoded[index-935]['reviewed'],self.manifest['phases'][index]['reviewed'])
        for value in (1,'true',None):
            with self.assertRaisesRegex(ValueError,'runtime_metadata_mode'):o.describe(self.j,[],_runtime_metadata=value)
        review,metadata=o._candidate_runtime_template(self.a,self.j,self.manifest,945,self.binding,self.ref['roles']['vault0'])
        self.assertEqual(o.semantic(review),o.semantic(metadata['reviewed']))
        self.assertEqual(review['requiredSigners'],metadata['reviewed']['requiredSigners'])
        self.assertEqual(metadata['lookups'],self.decoded[10]['lookups'])

    def test_source_derived_bootstrap_governance_and_execute_trace_geometry(self):
        r=self.ref['roles'];system='11111111111111111111111111111111';token='TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'
        for index,count in ((935,1),(937,1),(939,2),(940,1)):
            pending=self.pending(index);trace=o._candidate_runtime_cpi(self.a,self.c,index,pending)[0]['instructions'];roles=[x['address']for x in pending['reviewed']['accountRoles']]
            self.assertEqual(len(trace),count)
            create=self.c.b58decode(trace[0]['data'],52);self.assertEqual(int.from_bytes(create[:4],'little'),0)
            if index==935:self.assertEqual([roles[n]for n in trace[0]['accounts']],[r['h2'],r['programData'],r['loaderBuffer']])
            if index==939:
                self.assertEqual(int.from_bytes(create[12:20],'little'),102)
                self.assertEqual(int.from_bytes(self.c.b58decode(trace[1]['data'],12)[4:],'little'),int(self.ref['rents']['1546'])-int(self.ref['rents']['102']))
        pending=self.pending(945);roles=[x['address']for x in pending['reviewed']['accountRoles']]
        trace=o._candidate_runtime_cpi(self.a,self.c,945,pending)[0]['instructions']
        self.assertEqual(len(trace),41);self.assertEqual(sum(roles[ix['programIdIndex']]==system for ix in trace),38)
        self.assertEqual([roles[ix['programIdIndex']]for ix in trace[-2:]],[token,token])
        self.assertEqual(trace[0]['data'],self.c.b58encode(bytes.fromhex(self.binding['unsignedInitializerPlan']['native']['hex'])))
        self.assertTrue(all(ix['stackHeight']==3 for ix in trace[1:]))
        child_accounts={roles[n]for ix in trace[1:]for n in ix['accounts']}
        self.assertTrue(child_accounts.isdisjoint({r['htfpRecipient'],r['teamRecipient']}))
        for index in (936,938,941,942,943,944,946):self.assertEqual(o._candidate_runtime_cpi(self.a,self.c,index,self.pending(index)),[])

    def test_execute_exact_loaded_and_token_metadata_from_raw_integer_accounts(self):
        pending=self.pending(945);before=pending['preAccounts'];after=copy.deepcopy(before)
        token='TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA';plan=self.binding['unsignedInitializerPlan']
        mint=next(x['address']for x in plan['native']['roles']if x['role']=='mint');authority=plan['pivAuthority']
        for address in self.ref['targetAddresses'][14:]:
            raw=bytearray(165);raw[:32]=self.c.b58decode(mint,32);raw[32:64]=self.c.b58decode(authority,32);raw[108]=1
            after[address]={'exists':True,'owner':token,'executable':False,'lamports':self.ref['rents']['165'],'dataBase64':base64.b64encode(raw).decode()}
        roles=[x['address']for x in pending['reviewed']['accountRoles']]
        def tokens(accounts):
            out=[]
            for n,address in enumerate(roles):
                acc=accounts[address]
                if acc['exists']and acc['owner']==token:
                    raw=base64.b64decode(acc['dataBase64'])
                    if len(raw)==165:out.append({'accountIndex':n,'mint':mint,'owner':self.c.b58encode(raw[32:64]),'programId':token,
                        'uiTokenAmount':{'amount':str(int.from_bytes(raw[64:72],'little')),'decimals':9,'uiAmount':None,'uiAmountString':'0'}})
            return out
        meta={'loadedAddresses':{'writable':self.ref['targetAddresses'],'readonly':[]},'innerInstructions':o._candidate_runtime_cpi(self.a,self.c,945,pending),
            'preTokenBalances':tokens(before),'postTokenBalances':tokens(after)}
        o._candidate_runtime_metadata(self.a,self.c,945,pending,meta,after)
        self.assertEqual(len(meta['postTokenBalances'])-len(meta['preTokenBalances']),2)
        floating=copy.deepcopy(meta);floating['postTokenBalances'][-1]['uiTokenAmount']['uiAmount']=0.0
        parsed=self.c.strict_json(json.dumps(floating).encode())
        self.assertIs(type(parsed['postTokenBalances'][-1]['uiTokenAmount']['uiAmount']),self.c.Decimal)
        o._candidate_runtime_metadata(self.a,self.c,945,pending,parsed,after)
        for field in ('loaded','height','outer','amount','owner','decimals'):
            bad=copy.deepcopy(meta)
            if field=='loaded':bad['loadedAddresses']['writable'].reverse()
            elif field=='height':bad['innerInstructions'][0]['instructions'][0]['stackHeight']=True
            elif field=='outer':bad['innerInstructions'][0]['index']=True
            elif field=='amount':bad['postTokenBalances'][-1]['uiTokenAmount']['amount']='1'
            elif field=='owner':bad['postTokenBalances'][-1]['owner']=self.ref['roles']['g0']
            else:bad['postTokenBalances'][-1]['uiTokenAmount']['decimals']=True
            with self.assertRaises(ValueError):o._candidate_runtime_metadata(self.a,self.c,945,pending,bad,after)

    def test_suffix_prestate_rejects_clock_before_retained_post_before_any_quote(self):
        floor=int(self.host.context)+1;keys=sorted(self.ref['baseline']);values=[]
        for key in keys:
            account=self.host.original[key]
            values.append(None if not account['exists']else {'owner':account['owner'],'executable':account['executable'],'lamports':int(account['lamports']),
                'data':[account['dataBase64'],'base64'],'rentEpoch':0})
        query={'jsonrpc':'2.0','id':1,'method':'getMultipleAccounts','params':[keys,{'encoding':'base64','commitment':'finalized','minContextSlot':floor}]}
        frame={k:base64.b64encode(self.c.encode(v)).decode()for k,v in (('requestBase64',[query]),
            ('responseBase64',[{'jsonrpc':'2.0','id':1,'result':{'context':{'slot':floor},'value':values}}]),('httpBase64',{'status':200,'headers':[]}))}
        state={'cursor':935,'funding':{},'pending':None,'lastClock':{'contextSlot':str(floor),'unixTimestamp':str(int.from_bytes(base64.b64decode(self.ref['clock']['dataBase64'])[32:40],'little',signed=True))}}
        data={'wireBase64':'','feeCapLamports':'1','prestate':frame,'originalBlockhash':{},'preflight':{}}
        with self.assertRaisesRegex(ValueError,'startup_clock_context'):o._candidate_runtime_intent(self.a,self.j,state,data)

    def test_full_geometry_and_actual_unfunded_controller_no_runtime_override(self):
        self.assertEqual(o.CANDIDATE_STARTUP_RECORDS,1897);self.assertEqual(o._candidate_startup_remaining(947),0)
        self.assertLess(o._candidate_startup_remaining(0)+o.ACTION_EVIDENCE,o.MAX_EVIDENCE)
        for index in range(935,947):self.assertEqual(o._candidate_startup_bound(index),16*1024**2)
        for index in (True,947,-1):
            with self.assertRaises(ValueError):o._candidate_startup_bound(index)
        with self.assertRaises(ValueError):o._candidate_prefix_bound(935)
        with tempfile.TemporaryDirectory()as tmp:
            path=Path(tmp)/'runtime.candidate-controller'
            create=o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'CREATE','expectedTail':None,'data':self.manifest},path)
            self.assertTrue(create['runtimeSuffixProgressionImplemented']);self.assertEqual(create['startupCursor'],0)
            self.assertTrue(all(create[k]is False for k in o.CANDIDATE_PREPARATION_FLAGS if k!='journalMutationPerformed'))
            with self.assertRaisesRegex(ValueError,'prefix_funding_required'):
                o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'PHASE_INTENT','expectedTail':create['tail'],
                    'data':{'cursor':937,'runtimeFacts':{},'initializerBinding':self.binding,'funding':True}},path)
            self.assertFalse(path.with_name(path.name+'.evidence').exists())


class CandidateFundingPreparationTests(unittest.TestCase):
    """Real durable unsigned positives; real crypto refusals, no signing seam."""
    @classmethod
    def setUpClass(cls):
        from tools.test_startup_manifest import CandidateFundingReceiptTests
        cls.host=CandidateFundingReceiptTests;cls.host.setUpClass()
        cls.a=o.startup_manifest();cls.manifest=cls.host.manifest;cls.ref=cls.a.candidate_oracle();cls.c=cls.host.c

    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
        self.path=Path(self.tmp.name)/'prepare.candidate-controller';self.tail=None
        self.receipt,_,_=self.host.fixture();self.data={k:self.receipt[k]for k in ('feeCapLamports','originalBlockhash','preflight')}
        self.call('CREATE',self.manifest)

    def call(self,action,data=None,**kwargs):
        out=o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':action,'expectedTail':self.tail,'data':data},self.path,**kwargs)
        self.tail=out['tail'];return out
    def hashes(self):return {p.name:o.sha(p.read_bytes())for p in self.path.iterdir()}

    def test_actual_unsigned_prepare_exact_durable_readback_and_host_only_scope(self):
        previous=self.tail;out=self.call('PREPARE_FUNDING',self.data);prepared=out['fundingPreparation'];request=prepared['request'];before=self.hashes()
        self.assertEqual(self.call('INSPECT')['fundingPreparation'],prepared);self.assertEqual(before,self.hashes())
        wire=base64.b64decode(prepared['packet']['wireBase64']);self.assertEqual(len(wire),215);self.assertEqual(wire[:65],b'\1'+bytes(64))
        self.assertEqual(wire[65:],base64.b64decode(self.ref['sourceFundingReviewed']['messageBase64']))
        self.assertEqual(request['requiredSigners'],[self.ref['roles']['source']]);self.assertEqual(request['operatorTail'],previous)
        self.assertEqual(request['requestId'],o.sha(o.encode({k:v for k,v in request.items()if k!='requestId'})))
        create=json.loads((self.path/'00000000.json').read_bytes());self.assertEqual(request['sourceDomainSha256'],o.sha(o.encode(create['data']['sourceDomain'])))
        self.assertEqual(out['status'],'UNSIGNED_FUNDING_PREPARATION_RETAINED_HOST_ONLY');self.assertIsNone(out['originalFundingIdentity'])
        self.assertEqual(prepared['packet']['feeCapLamports'],'10000');self.assertEqual(prepared['packet']['feeQuoteLamports'],'5000')
        self.assertEqual(out['historicalUncertaintyReference'],self.manifest['historicalUncertainty'])
        self.assertTrue(all(out[k]is False for k in o.CANDIDATE_PREPARATION_FLAGS if k!='journalMutationPerformed'))
        for name in ('signerInvoked','signingAttemptPerformed','externalSigningAuthorized'):self.assertFalse(out[name])
        self.assertEqual(o.CANDIDATE_CONTROLLER_RECORDS,3792);self.assertEqual(o.CANDIDATE_STARTUP_RECORDS,1897)
        self.assertEqual(set(self.path.iterdir()),{self.path/'lock',self.path/'00000000.json',self.path/'00000001.json'})

    def test_prepared_zero_and_invalid_signatures_reach_real_checker_without_append(self):
        prepared=self.call('PREPARE_FUNDING',self.data)['fundingPreparation'];before=self.hashes();tail=self.tail
        for signature,reason in ((bytes(64),'zero_signature'),(bytes([1])*64,'signature_invalid')):
            wire=base64.b64decode(prepared['packet']['wireBase64']);wire=wire[:1]+signature+wire[65:]
            with self.assertRaisesRegex(ValueError,'signed_packet_or_receipt_rejected:'+reason):
                self.call('FUNDING_INTENT',{'requestId':prepared['request']['requestId'],'wireBase64':base64.b64encode(wire).decode()})
            self.assertEqual(before,self.hashes());self.assertEqual(self.tail,tail)
        self.assertEqual(self.call('INSPECT')['fundingPreparation'],prepared)
        with self.assertRaisesRegex(ValueError,'result_requires_original_intent'):
            self.call('FUNDING_RESULT',{'receipt':self.receipt,'expectedReceiptSha256':o.sha(o.encode(self.receipt))})
        self.assertEqual(before,self.hashes())

    def test_prepare_checks_actual_accounts_clock_context_and_sufficiency(self):
        before=self.hashes();base=int(self.ref['baselineContextSlot'])
        for change,reason in (('source_owner','preparation_system'),('h2','preparation_sufficiency'),('source_amount','preparation_sufficiency'),('clock','startup_clock_context'),('quote_context','preparation_context')):
            data=copy.deepcopy(self.data);replies=json.loads(base64.b64decode(data['preflight']['responseBase64']));accounts=replies[1]['result']['value']
            if change=='source_owner':accounts[0]['owner']=self.ref['roles']['source']
            elif change=='h2':accounts[1]['lamports']+=1
            elif change=='source_amount':accounts[0]['lamports']=4431889280+4999
            elif change=='clock':
                raw=bytearray(base64.b64decode(accounts[2]['data'][0]));raw[:8]=(base+9).to_bytes(8,'little');accounts[2]['data'][0]=base64.b64encode(raw).decode()
            else:replies[0]['result']['context']['slot']=base+9
            data['preflight']['responseBase64']=base64.b64encode(self.c.encode(replies)).decode()
            with self.assertRaisesRegex(ValueError,reason):self.call('PREPARE_FUNDING',data)
            self.assertEqual(before,self.hashes())
        # Shared Source has no invented R0 floor; this is unsigned host arithmetic.
        data=copy.deepcopy(self.data);replies=json.loads(base64.b64decode(data['preflight']['responseBase64']))
        replies[1]['result']['value'][0]['lamports']=4431889280+5000+1
        data['preflight']['responseBase64']=base64.b64encode(self.c.encode(replies)).decode()
        prepared=self.call('PREPARE_FUNDING',data)['fundingPreparation']
        self.assertEqual(prepared['accounts'][self.ref['roles']['source']]['lamports'],'4431894281')

    def test_original_request_replacement_and_caller_overrides_refuse(self):
        prepared=self.call('PREPARE_FUNDING',self.data)['fundingPreparation'];before=self.hashes()
        with self.assertRaisesRegex(ValueError,'once_before_intent'):self.call('PREPARE_FUNDING',self.data)
        with self.assertRaisesRegex(ValueError,'preparation_request'):
            self.call('FUNDING_INTENT',{'requestId':'0'*64,'wireBase64':prepared['packet']['wireBase64']})
        for extra in ('feeCapLamports','originalBlockhash','preflight','funding','authorization'):
            with self.assertRaisesRegex(ValueError,'object_fields'):
                self.call('FUNDING_INTENT',{'requestId':prepared['request']['requestId'],'wireBase64':prepared['packet']['wireBase64'],extra:self.data.get(extra,True)})
        raw_intent={k:self.receipt[k]for k in ('wireBase64','feeCapLamports','originalBlockhash','preflight')}
        with self.assertRaisesRegex(ValueError,'object_fields'):self.call('FUNDING_INTENT',raw_intent)
        changed=bytearray(base64.b64decode(prepared['packet']['wireBase64']));changed[-1]^=1
        with self.assertRaisesRegex(ValueError,'original_wire'):
            self.call('FUNDING_INTENT',{'requestId':prepared['request']['requestId'],'wireBase64':base64.b64encode(changed).decode()})
        self.assertEqual(before,self.hashes())

    def test_existing_unverified_intent_cannot_be_upgraded_or_reclassified(self):
        intent={k:self.receipt[k]for k in ('wireBase64','feeCapLamports','originalBlockhash','preflight')}
        held=self.call('FUNDING_INTENT',intent);before=self.hashes()
        self.assertEqual(held['fundingIntentOrigin'],'DIRECT_UNVERIFIED_HOST_INTENT');self.assertIsNone(held['fundingPreparation'])
        with self.assertRaisesRegex(ValueError,'once_before_intent'):self.call('PREPARE_FUNDING',self.data)
        self.assertEqual(before,self.hashes());self.assertEqual(self.call('INSPECT')['originalFundingIdentity'],held['originalFundingIdentity'])

    def test_retained_preparation_commitment_and_tail_are_independent_guards(self):
        self.call('PREPARE_FUNDING',self.data);before=self.hashes()
        with self.assertRaisesRegex(ValueError,'tail_compare_and_swap'):
            o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'INSPECT','expectedTail':None,'data':None},self.path)
        self.assertEqual(before,self.hashes());path=self.path/'00000001.json';record=json.loads(path.read_bytes())
        record['data']['packet']['feeCapLamports']='10001';record['digest']=o.sha(o.encode({k:v for k,v in record.items()if k!='digest'}))
        path.write_bytes(o.encode(record));self.tail=record['digest'];tampered=self.hashes()
        with self.assertRaisesRegex(ValueError,'preparation_binding'):self.call('INSPECT')
        self.assertEqual(tampered,self.hashes())


class CandidateStartupPreparationTests(unittest.TestCase):
    """Host storage/unsigned semantics and crypto refusals, never funded preparation."""
    @classmethod
    def setUpClass(cls):
        from tools.test_startup_manifest import CandidateFundingReceiptTests
        cls.host=CandidateFundingReceiptTests;cls.host.setUpClass();cls.a=o.startup_manifest();cls.manifest=cls.host.manifest
        cls.j,_,cls.c=cls.a._candidate_helpers()

    def component(self):
        # Minimal private-helper fixture only; operate never accepts this state.
        phase=self.manifest['phases'][0];review=copy.deepcopy(phase['reviewed']);count=len(review['requiredSigners'])
        message=base64.b64decode(review['messageBase64']);wire=base64.b64encode(bytes([count])+bytes(64*count)+message).decode()
        lifetime={'recentBlockhash':self.c.b58encode(message[4+32*message[3]:36+32*message[3]]),
            'currentBlockHeight':'100','lastValidBlockHeight':'300','minimumRemainingBlockHeights':'40'}
        pending={'input':{'wireBase64':wire},'reviewed':review,'lifetime':lifetime,'feeQuoteLamports':phase['historicalFeeProjectionLamports'],
            'identity':{'index':0,'wireSha256':o.sha(base64.b64decode(wire)),'messageSha256':review['messageSha256'],'signatureHex':'00'*64}}
        packet={'id':phase['id'],'wireBase64':wire,'lifetime':lifetime,'feeQuoteLamports':pending['feeQuoteLamports'],'reviewed':review}
        request={'phaseIndex':0,'requestId':'a'*64,'feeCapLamports':pending['feeQuoteLamports']}
        return {'manifest':self.manifest,'cursor':0,'pending':None,'phasePreparation':{'pending':pending,'packet':packet,'request':request}}

    def test_real_zero_and_invalid_signatures_refuse_component_consumption_without_mutation(self):
        state=self.component();before=o.encode(state);packet=state['phasePreparation']['packet'];wire=base64.b64decode(packet['wireBase64']);count=wire[0]
        self.assertEqual(set(packet),{'id','wireBase64','lifetime','feeQuoteLamports','reviewed'})
        for signature,reason in ((bytes(64),'zero_signature'),(bytes([1])*64,'signature_invalid')):
            signed=wire[:1]+signature*count+wire[1+64*count:]
            with self.assertRaisesRegex(ValueError,'signed_packet_or_receipt_rejected:'+reason):
                o._candidate_consume_phase_preparation(self.a,self.j,state,{'requestId':'a'*64,'wireBase64':base64.b64encode(signed).decode()})
            self.assertEqual(before,o.encode(state))

    def test_request_message_cursor_and_override_guards_precede_crypto(self):
        state=self.component();wire=state['phasePreparation']['packet']['wireBase64']
        with self.assertRaisesRegex(ValueError,'request_or_hold'):
            o._candidate_consume_phase_preparation(self.a,self.j,state,{'requestId':'b'*64,'wireBase64':wire})
        changed=bytearray(base64.b64decode(wire));changed[-1]^=1
        with self.assertRaisesRegex(ValueError,'preparation_message'):
            o._candidate_consume_phase_preparation(self.a,self.j,state,{'requestId':'a'*64,'wireBase64':base64.b64encode(changed).decode()})
        for extra in ('feeCapLamports','runtimeFacts','clock','initializerBinding','preflight','cursor'):
            with self.assertRaisesRegex(ValueError,'object_fields'):
                o._candidate_consume_phase_preparation(self.a,self.j,state,{'requestId':'a'*64,'wireBase64':wire,extra:True})
        state['cursor']=1
        with self.assertRaisesRegex(ValueError,'request_or_hold'):
            o._candidate_consume_phase_preparation(self.a,self.j,state,{'requestId':'a'*64,'wireBase64':wire})

    def test_prepare_rejects_existing_preparation_pending_intent_and_nonzero_wire(self):
        state=self.component();unsigned=state['phasePreparation']['packet']['wireBase64']
        state.update(funding={},phasePreparation=None)
        nonzero=bytearray(base64.b64decode(unsigned));nonzero[1]=1
        with self.assertRaisesRegex(ValueError,'zero_signatures'):
            o._candidate_prepare_phase(self.a,self.j,state,{'wireBase64':base64.b64encode(nonzero).decode()},None,{})
        for key in ('pending','phasePreparation'):
            held=copy.deepcopy(state);held[key]={'host':'unresolved'}
            with self.assertRaisesRegex(ValueError,'funding_or_hold'):
                o._candidate_prepare_phase(self.a,self.j,held,{'wireBase64':unsigned},None,{})

    def test_real_prepare_sidecar_retains_raw_once_and_orphan_uncertainty(self):
        with tempfile.TemporaryDirectory()as tmp:
            path=Path(tmp)/'startup.candidate-controller'
            created=o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'CREATE','expectedTail':None,'data':self.manifest},path)
            journal=o.Journal(self.j,path,_candidate=True)
            try:
                raw=o.encode({'storageComponentOnly':base64.b64encode(bytes(892389)).decode()})
                ref=o._candidate_prefix_store(self.c,self.j,journal,{'cursor':0},'PREPARE_PHASE',raw)
                self.assertEqual(ref['name'],'00000001-prepare')
                self.assertEqual(o.encode(o._candidate_prefix_load(self.j,path,ref,'1','PREPARE_PHASE',0)),raw)
                self.assertEqual(o._candidate_prefix_inventory(self.j,path,[ref]),len(raw))
                with self.assertRaisesRegex(ValueError,'evidence_reference'):o._candidate_prefix_load(self.j,path,ref,'1','PHASE_INTENT',0)
            finally:journal.close()
            with self.assertRaisesRegex(ValueError,'orphan_uncertain_stop'):
                o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'INSPECT','expectedTail':created['tail'],'data':None},path)
            self.assertEqual((path.with_name(path.name+'.evidence')/ref['name']/'payload.json').read_bytes(),raw)

    def test_actual_unfunded_prepare_refuses_without_evidence_or_caller_credit(self):
        with tempfile.TemporaryDirectory()as tmp:
            path=Path(tmp)/'unfunded.candidate-controller';created=o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'CREATE','expectedTail':None,'data':self.manifest},path)
            before={p.name:o.sha(p.read_bytes())for p in path.iterdir()}
            for data in ({},{'cursor':937,'funding':True,'runtimeFacts':{},'initializerBinding':{}}):
                with self.assertRaisesRegex(ValueError,'prefix_funding_required'):
                    o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'PREPARE_PHASE','expectedTail':created['tail'],'data':data},path)
            self.assertEqual(before,{p.name:o.sha(p.read_bytes())for p in path.iterdir()});self.assertFalse(path.with_name(path.name+'.evidence').exists())
            self.assertTrue(created['startupPreparationImplemented']);self.assertIsNone(created['phasePreparation'])

    def test_compact_record_geometry_keeps_two_full_slots_and_existing_caps(self):
        self.assertEqual(o.CANDIDATE_CONTROLLER_RECORDS,3792);self.assertEqual(o.CANDIDATE_STARTUP_RECORDS,1897)
        self.assertEqual(o._candidate_startup_remaining(0),4353687552)
        self.assertEqual((o.MAX_JOURNAL,o.MAX_EVIDENCE,self.j.RESERVE),(128*1024**2,8*1024**3,2*1024**3))
        state=self.component();record={'requestId':'a'*64,'wireBase64':state['phasePreparation']['packet']['wireBase64'],'derived':state['phasePreparation']['pending']['identity']}
        self.assertLess(947*len(o.encode(record))+o.CANDIDATE_CONTROLLER_RECORDS*16384,o.MAX_JOURNAL)
        self.assertNotIn('evidence',record);self.assertNotIn('preflight',record)


class CandidateCurrentPhaseTests(unittest.TestCase):
    """Observed PRE0/component Clock seams; no funded937/946 or controller positive."""
    @classmethod
    def setUpClass(cls):
        from tools.test_startup_manifest import CandidateAccountVerificationTests
        cls.host=CandidateAccountVerificationTests;cls.host.setUpClass();cls.a=cls.host.a;cls.ref=cls.host.ref;cls.manifest=cls.host.manifest
        cls.j,cls.rules,cls.c=cls.a._candidate_helpers()
        cls.pre=cls.a.candidate_verify_accounts(cls.manifest,cls.a.candidate_phase(0),'PRE',cls.host.original,context_slot=cls.host.context)['clock']
        cls.binding=cls.j.candidate_clock_binding(cls.ref['preManifest'],cls.ref['clock'])
        packet=cls.binding['unsignedInitializerPlan']['packets'][0]
        cls.review937,=o.describe(cls.j,[{'wireBase64':packet['wireBase64'],'lookupTables':[]}])

    def fixture(self,index=0,seconds=1):
        # Only PRE0 map is a real verified full PRE;937/946 select Clock rules
        # over this unchanged observed-only map, not valid funded phase states.
        r=self.ref['roles'];keys=sorted(self.host.original)
        if index==1:keys=sorted(r[k]for k in ('g0','h2','pivProgram','programData','loaderBuffer','loaderProgram','computeBudget','rent','clock','instructions'))
        accounts={k:copy.deepcopy(self.host.original[k])for k in keys};review=self.review937 if index==937 else self.manifest['phases'][index]['reviewed']
        fee=self.manifest['phases'][index]['historicalFeeProjectionLamports'];floor=int(self.host.context)
        pending={'preAccounts':accounts,'preClock':copy.deepcopy(self.pre),'preContext':str(floor),'reviewed':review,
            'input':{'feeCapLamports':fee},'feeQuoteLamports':fee,'lifetime':{'recentBlockhash':self.ref['preManifest']['initializer']['transport']['recentBlockhash'],
            'currentBlockHeight':'100','lastValidBlockHeight':'300','minimumRemainingBlockHeights':'40'}}
        state={'manifest':self.manifest,'cursor':index,'phasePreparation':{'pending':pending},'pending':None,
            'lastClock':copy.deepcopy(self.pre),'lastSlot':str(floor),'initializerBinding':copy.deepcopy(self.binding)if index>=937 else None,'runtimeFacts':None}
        current=copy.deepcopy(accounts);raw=bytearray(base64.b64decode(current[r['clock']]['dataBase64']))
        raw[:8]=(floor+1).to_bytes(8,'little');raw[32:40]=(int(self.pre['unixTimestamp'])+seconds).to_bytes(8,'little',signed=True)
        current[r['clock']]['dataBase64']=base64.b64encode(raw).decode();values=[]
        for key in keys:
            acc=current[key];values.append(None if not acc['exists']else {'owner':acc['owner'],'executable':acc['executable'],'lamports':int(acc['lamports']),'data':[acc['dataBase64'],'base64']})
        config={'commitment':'finalized','minContextSlot':floor};queries=[('getGenesisHash',[]),('getFeeForMessage',[review['messageBase64'],config]),
            ('getMultipleAccounts',[keys,{'encoding':'base64',**config}]),('isBlockhashValid',[pending['lifetime']['recentBlockhash'],config]),('getBlockHeight',[config])]
        replies=[self.c.GENESIS,{'context':{'slot':floor+1},'value':int(fee)},{'context':{'slot':floor+1},'value':values},{'context':{'slot':floor+1},'value':True},101]
        return state,queries,replies

    def frame(self,queries,replies):
        return {'current':{k:base64.b64encode(self.c.encode(v)).decode()for k,v in (('requestBase64',[{'jsonrpc':'2.0','id':i+1,'method':m,'params':p}for i,(m,p)in enumerate(queries)]),
            ('responseBase64',[{'jsonrpc':'2.0','id':i+1,'result':v}for i,v in enumerate(replies)]),('httpBase64',{'status':200,'headers':[]}))}}
    def check(self,state,queries,replies):return o._candidate_verify_current_phase(self.a,self.j,state,self.frame(queries,replies))

    def test_actual60_observed_pre_component_accepts_later_clock_and_source_without_refresh(self):
        state,q,v=self.fixture();before=o.sha(o.encode(state));source=q[2][1][0].index(self.ref['roles']['source']);v[2]['value'][source]['lamports']+=7
        out=self.check(state,q,v);self.assertEqual(len(out['checkedAccountKeys']),60);self.assertGreater(int(out['clock']['slot']),int(self.pre['slot']))
        self.assertEqual(o.sha(o.encode(state)),before);self.assertFalse(out['executionPrestateRefreshed']);self.assertFalse(out['nativeAdmissionQualified'])
        self.assertTrue(all(out[k]is False for k in o.CANDIDATE_PREPARATION_FLAGS));self.assertIn('NO_NEW_FLOOR',out['monotonicityScope'])
        self.assertEqual(out['accountDigests'][self.ref['roles']['source']]['lamports'],str(int(self.host.original[self.ref['roles']['source']]['lamports'])+7))

    def test_frozen937_anchor_and946_period_components_remain_separate_from_current_clock(self):
        state,q,v=self.fixture(937);frozen=o.sha(o.encode(state['initializerBinding']));out=self.check(state,q,v)
        self.assertEqual(o.sha(o.encode(state['initializerBinding'])),frozen);self.assertNotEqual(out['clock']['dataBase64'],state['initializerBinding']['clock']['dataBase64'])
        state,q,v=self.fixture(946);self.check(state,q,v)
        state,q,v=self.fixture(946,self.rules.PERIOD)
        with self.assertRaisesRegex(ValueError,'heartbeat_ambiguous_period'):self.check(state,q,v)

    def test_existing_jito_mutability_and_security_predicates_are_used_unchanged(self):
        state,q,v=self.fixture();roles=self.a.oracle()['roles'];keys=q[2][1][0]
        row=v[2]['value'][keys.index(roles['stakePool'])];raw=bytearray(base64.b64decode(row['data'][0]));raw[258:266]=(int.from_bytes(raw[258:266],'little')+1).to_bytes(8,'little');row['data'][0]=base64.b64encode(raw).decode()
        out=self.check(state,q,v);self.assertNotEqual(out['accountDigests'][roles['stakePool']]['dataSha256'],self.ref['baseline'][roles['stakePool']]['dataSha256'])
        raw[1]^=1;row['data'][0]=base64.b64encode(raw).decode()
        with self.assertRaisesRegex(ValueError,'jito_pool_security_policy'):self.check(state,q,v)

    def test_current_context_clock_accounts_fee_hash_and_original_lifetime_refuse(self):
        for change,reason in (('context','identity_or_context'),('clock','startup_clock_context'),('static','static_account_changed'),('fee','fee_profile_or_cap'),('valid','original_lifetime'),('height','original_lifetime'),('margin','original_lifetime'),('hash','raw_request')):
            state,q,v=self.fixture();keys=q[2][1][0]
            if change=='context':v[1]['context']['slot']-=2
            elif change=='clock':
                row=v[2]['value'][keys.index(self.ref['roles']['clock'])];raw=bytearray(base64.b64decode(row['data'][0]));raw[:8]=(int(self.host.context)-1).to_bytes(8,'little');row['data'][0]=base64.b64encode(raw).decode()
            elif change=='static':v[2]['value'][keys.index(self.ref['roles']['g0'])]['lamports']+=1
            elif change=='fee':v[1]['value']+=1
            elif change=='valid':v[3]['value']=False
            elif change=='height':v[4]=99
            elif change=='margin':v[4]=261
            else:q[3][1][0]=self.ref['roles']['vault0']
            with self.assertRaisesRegex(ValueError,reason):self.check(state,q,v)
        state,q,v=self.fixture(1);v[2]['value'].append(None)
        with self.assertRaisesRegex(ValueError,'prefix_accounts'):self.check(state,q,v)
        state,q,v=self.fixture();key=self.a.large_key();row=v[2]['value'][q[2][1][0].index(key)];raw=bytearray(base64.b64decode(row['data'][0]));raw[-1]^=1;row['data'][0]=base64.b64encode(raw).decode()
        with self.assertRaisesRegex(ValueError,'large_account_complete_image'):self.check(state,q,v)

    def test_actual_unfunded_readonly_action_refuses_without_journal_or_evidence_mutation(self):
        with tempfile.TemporaryDirectory()as tmp:
            path=Path(tmp)/'current.candidate-controller';out=o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'CREATE','expectedTail':None,'data':self.manifest},path)
            before={p.name:o.sha(p.read_bytes())for p in path.iterdir()}
            with self.assertRaisesRegex(ValueError,'current_funding_required'):
                o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'VERIFY_CURRENT_PHASE','expectedTail':out['tail'],'data':{'current':{}}},path)
            self.assertEqual(before,{p.name:o.sha(p.read_bytes())for p in path.iterdir()});self.assertFalse(path.with_name(path.name+'.evidence').exists())


class CandidateSignerBoundaryTests(unittest.TestCase):
    """Public projection/current/storage components, not funded native attempts."""
    @classmethod
    def setUpClass(cls):
        cls.host=CandidateCurrentPhaseTests;cls.host.setUpClass();cls.a=cls.host.a;cls.j=cls.host.j;cls.c=cls.host.c;cls.manifest=cls.host.manifest

    def fixture(self,expiry='18446744073709551615'):
        host=self.host(methodName='test_actual60_observed_pre_component_accepts_later_clock_and_source_without_refresh')
        state,q,v=host.fixture();pending=state['phasePreparation']['pending'];pending['reviewed']=copy.deepcopy(pending['reviewed']);review=pending['reviewed'];count=len(review['requiredSigners'])
        wire=base64.b64encode(bytes([count])+bytes(64*count)+base64.b64decode(review['messageBase64'])).decode()
        pending['input']['wireBase64']=wire;pending['identity']={'index':0,'wireSha256':o.sha(base64.b64decode(wire)),'messageSha256':review['messageSha256'],'signatureHex':'00'*64}
        packet={'id':self.manifest['phases'][0]['id'],'wireBase64':wire,**{k:pending[k]for k in ('lifetime','feeQuoteLamports','reviewed')}}
        state['phasePreparation'].update(packet=packet,request={'phaseIndex':0,'requestId':'a'*64,'feeCapLamports':pending['input']['feeCapLamports']},recordDigest='b'*64)
        state['phaseSignAttempt']=None;state['controllerPath']='/tmp/piv1-public-sign-component.candidate-controller';view=o._candidate_signing_view(self.a,state)
        receipt={'schema':'piv1-launcher-release-authorization-v2','manifestSha256':view['manifestSha256'],'baseParentTail':None,
            'operatorSourceSha256':o.sha(Path(o.__file__).read_bytes()),'expiresUnixTimestamp':expiry,'scope':'EXACT_BOUNDED_TESTNET_RELEASE',
            'feeBudgetNamespace':view['manifest']['proposedFeeBudget']['namespace'],'signerBackendSha256':o.SIGNER_BACKEND_SHA,'signerProfileSha256':'c'*64}
        raw=o.encode(receipt);data={'authorizationRawBase64':base64.b64encode(raw).decode(),'authorizationSha256':o.sha(raw),'current':host.frame(q,v)['current']}
        return state,data

    def attempt(self,state,data,**kw):return o._candidate_sign_phase_attempt(self.a,self.j,state,data,'d'*64,**kw)
    def response(self,state):return {'schema':'piv1-startup-signing-response-v1','requestId':state['phaseSignAttempt']['identity']['request']['requestId'],
        'packets':[{'id':state['phasePreparation']['packet']['id'],'wireBase64':state['phasePreparation']['packet']['wireBase64']}]}

    def test_existing_public_contract_projection_binds_prepared_source_nullparent_and_lifetime(self):
        state,data=self.fixture();before=o.sha(o.encode(state));attempt=self.attempt(state,data);request=attempt['identity']['request'];packet=state['phasePreparation']['packet']
        self.assertEqual(request['schema'],'piv1-startup-signing-request-v1');self.assertEqual(request['baseParentPath'],state['controllerPath']);self.assertIsNone(request['baseParentTail']);self.assertEqual(request['operatorTail'],'d'*64)
        self.assertEqual(request['packets'][0]['preparedRecordDigest'],'b'*64);self.assertEqual(request['packets'][0]['preparedSha256'],o.sha(o.encode(packet)))
        self.assertEqual(request['requestId'],o.sha(o.encode({k:v for k,v in request.items()if k!='requestId'})))
        self.assertNotEqual(request['requestId'],state['phasePreparation']['request']['requestId']);self.assertEqual(request['authorizationSha256'],data['authorizationSha256'])
        self.assertEqual(attempt['identity']['status'],'HOST_SIGNER_BOUNDARY_RESERVED_NOT_INVOKED');self.assertFalse(attempt['identity']['nativeAdmissionQualified'])
        self.assertEqual(o.sha(o.encode(state)),before);self.assertEqual(set(packet),{'id','wireBase64','lifetime','feeQuoteLamports','reviewed'})
        foreign=copy.deepcopy(state);foreign['controllerPath']='/tmp/piv1-foreign-sign-component.candidate-controller'
        self.assertNotEqual(self.attempt(foreign,data)['identity']['request']['requestId'],request['requestId'])

    def test_public_authorization_declaration_parent_source_profile_and_namespace_refuse(self):
        for change,reason in (('baseParentTail','authorization_binding'),('operatorSourceSha256','authorization_binding'),('signerProfileSha256','digest'),('feeBudgetNamespace','fee_namespace_binding')):
            state,data=self.fixture();receipt=json.loads(base64.b64decode(data['authorizationRawBase64']))
            receipt[change]={'sequence':'0','digest':'0'*64}if change=='baseParentTail'else 'invalid'
            raw=o.encode(receipt);data.update(authorizationRawBase64=base64.b64encode(raw).decode(),authorizationSha256=o.sha(raw))
            with self.assertRaisesRegex(ValueError,reason):self.attempt(state,data)
        state,data=self.fixture();state['phasePreparation']['packet']['reviewed']['requiredSigners']=[self.a.candidate_oracle()['roles']['source']]
        with self.assertRaisesRegex(ValueError,'backend_identity'):self.attempt(state,data)

    def test_original_backend_response_mapping_and_real_crypto_refuse_without_mutation(self):
        state,data=self.fixture();state['phaseSignAttempt']={**self.attempt(state,data),'recordDigest':'e'*64};before=o.sha(o.encode(state))
        for signature,reason in ((bytes(64),'zero_signature'),(bytes([1])*64,'signature_invalid')):
            response=self.response(state);wire=base64.b64decode(response['packets'][0]['wireBase64']);response['packets'][0]['wireBase64']=base64.b64encode(wire[:1]+signature*wire[0]+wire[1+64*wire[0]:]).decode()
            with self.assertRaisesRegex(ValueError,'signed_packet_or_receipt_rejected:'+reason):o._candidate_original_signer_response(self.a,self.j,state,response)
            self.assertEqual(o.sha(o.encode(state)),before)
        response=self.response(state);response['requestId']=state['phasePreparation']['request']['requestId']
        with self.assertRaisesRegex(ValueError,'original_response_only'):o._candidate_original_signer_response(self.a,self.j,state,response)
        response=self.response(state);response['packets'][0]['id']='other'
        with self.assertRaisesRegex(ValueError,'response_packet'):o._candidate_original_signer_response(self.a,self.j,state,response)
        with self.assertRaisesRegex(ValueError,'object_fields'):o._candidate_original_signer_response(self.a,self.j,state,{'requestId':'a'*64,'wireBase64':'AA=='})

    def test_boundary_is_once_only_and_expired_replay_never_admits_new_response(self):
        state,data=self.fixture();state['phaseSignAttempt']={**self.attempt(state,data),'recordDigest':'e'*64}
        with self.assertRaisesRegex(ValueError,'original_boundary_only'):self.attempt(state,data)
        state,data=self.fixture('0')
        with self.assertRaisesRegex(ValueError,'authorization_expired'):self.attempt(state,data)
        historical=self.attempt(state,data,historical=True);state['phaseSignAttempt']={**historical,'recordDigest':'e'*64}
        with self.assertRaisesRegex(ValueError,'authorization_expired'):o._candidate_original_signer_response(self.a,self.j,state,self.response(state))
        state,data=self.fixture();data['historical']=True
        with self.assertRaisesRegex(ValueError,'object_fields'):self.attempt(state,data)

    def test_real_sign_slot_readback_and_orphan_stop_preserve_raw_without_adoption(self):
        with tempfile.TemporaryDirectory()as tmp:
            path=Path(tmp)/'sign.candidate-controller';created=o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'CREATE','expectedTail':None,'data':self.manifest},path)
            state,data=self.fixture();raw=o.encode(data);journal=o.Journal(self.j,path,_candidate=True)
            try:
                ref=o._candidate_prefix_store(self.c,self.j,journal,{'cursor':0},'SIGN_PHASE_ATTEMPT',raw)
                self.assertEqual(ref['name'],'00000001-sign');self.assertEqual(o.encode(o._candidate_prefix_load(self.j,path,ref,'1','SIGN_PHASE_ATTEMPT',0)),raw)
            finally:journal.close()
            with self.assertRaisesRegex(ValueError,'orphan_uncertain_stop'):o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'INSPECT','expectedTail':created['tail'],'data':None},path)
            self.assertEqual((path.with_name(path.name+'.evidence')/ref['name']/'payload.json').read_bytes(),raw)

    def test_actual_unfunded_boundary_and_native_route_refuse_with_fixed_geometry(self):
        with tempfile.TemporaryDirectory()as tmp:
            path=Path(tmp)/'blocked.candidate-controller';created=o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'CREATE','expectedTail':None,'data':self.manifest},path)
            request={'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'SIGN_PHASE_ATTEMPT','expectedTail':created['tail'],'data':{}};before={p.name:o.sha(p.read_bytes())for p in path.iterdir()}
            with self.assertRaisesRegex(ValueError,'prefix_funding_required'):o.operate(request,path)
            with self.assertRaisesRegex(ValueError,'candidate_controller_offline_only'):o.operate(request,path,live_sign=True)
            self.assertFalse(created['nativeDispatcherEnabled']);self.assertFalse(created['signerInvoked']);self.assertEqual(before,{p.name:o.sha(p.read_bytes())for p in path.iterdir()})
        self.assertEqual(o.CANDIDATE_CONTROLLER_RECORDS,3792);self.assertEqual(o._candidate_evidence_remaining(0),6530531328)
        self.assertLess(o._candidate_evidence_remaining(0)+o.ACTION_EVIDENCE,o.MAX_EVIDENCE)
        self.assertEqual((o.MAX_JOURNAL,o.MAX_EVIDENCE,self.j.RESERVE),(128*1024**2,8*1024**3,2*1024**3))


class CandidateExecutionPrestateTests(unittest.TestCase):
    """Retained supplied-PRE components only; no funded/signed/native positive."""
    @classmethod
    def setUpClass(cls):
        cls.host=CandidateCurrentPhaseTests;cls.host.setUpClass();cls.a=cls.host.a;cls.j=cls.host.j;cls.c=cls.host.c;cls.ref=cls.host.ref
    def fixture(self,index=0,seconds=1):
        host=self.host(methodName='test_actual60_observed_pre_component_accepts_later_clock_and_source_without_refresh')
        state,q,v=host.fixture(index,seconds);data=host.frame(q,v);report=o._candidate_verify_current_phase(self.a,self.j,state,data)
        raw={'current':data['current']};state['phaseSignAttempt']={'input':raw,'recordDigest':'e'*64,'identity':{'inputSha256':o.sha(o.encode(raw)),
            'currentProofSha256':o.sha(o.encode(report)),'request':{'operatorSourceSha256':o.sha(Path(o.__file__).read_bytes())}}}
        return state,q,v,host
    def test_exact_retained_current_derives_execution_accounts_without_rebinding_original_pre(self):
        state,q,v,host=self.fixture(937);before=o.sha(o.encode(state));execution=o._candidate_execution_pre(self.a,self.j,state)
        pending=copy.deepcopy(state['phasePreparation']['pending']);pending['executionPre']=execution
        self.assertEqual(o.sha(o.encode(state)),before);self.assertEqual(o._candidate_result_pre(pending),execution['accounts'])
        self.assertNotEqual(execution['clock']['dataBase64'],pending['preClock']['dataBase64']);self.assertEqual(state['initializerBinding'],host.binding)
        self.assertNotIn('executionPre',state['phasePreparation']['pending']);self.assertEqual(execution['attemptRecordDigest'],'e'*64)
        for value in (1,'true',None):
            with self.assertRaisesRegex(ValueError,'current_accounts_mode'):o._candidate_verify_current_phase(self.a,self.j,state,host.frame(q,v),_with_accounts=value)
        self.assertNotIn('accounts',o._candidate_verify_current_phase(self.a,self.j,state,host.frame(q,v)))
    def test_retained_input_proof_and_later_current_context_time_cannot_regress(self):
        state,q,v,host=self.fixture();execution=o._candidate_execution_pre(self.a,self.j,state)
        for name,reason in (('inputSha256','execution_original_attempt'),('currentProofSha256','execution_current_commitment')):
            changed=copy.deepcopy(state);changed['phaseSignAttempt']['identity'][name]='0'*64
            with self.assertRaisesRegex(ValueError,reason):o._candidate_execution_pre(self.a,self.j,changed)
        state['pending']=copy.deepcopy(state['phasePreparation']['pending']);state['pending']['executionPre']=execution
        floor=int(execution['contextSlot'])
        for _,params in q[1:4]:params[-1]['minContextSlot']=floor
        with self.assertRaisesRegex(ValueError,'current_identity_or_context'):
            stale=copy.deepcopy(v);stale[2]['context']['slot']=floor-1;o._candidate_verify_current_phase(self.a,self.j,state,host.frame(q,stale))
        row=v[2]['value'][q[2][1][0].index(self.ref['roles']['clock'])];raw=bytearray(base64.b64decode(row['data'][0]));raw[32:40]=int(host.pre['unixTimestamp']).to_bytes(8,'little',signed=True);row['data'][0]=base64.b64encode(raw).decode()
        with self.assertRaisesRegex(ValueError,'current_clock_regression'):o._candidate_verify_current_phase(self.a,self.j,state,host.frame(q,v))
    def test_execute_real_roles_require_current_reserve_bank_and_manager_token_pre(self):
        plan=self.host.binding['unsignedInitializerPlan'];packet=plan['packets'][8]
        metadata,=o.describe(self.j,[{'wireBase64':packet['wireBase64'],'lookupTables':[{'address':plan['alt']['address'],'addresses':plan['alt']['addresses']}]}],_runtime_metadata=True)
        review=metadata['reviewed'];roles=[x['address']for x in review['accountRoles']];protocol=self.ref['preManifest']['initializer']['protocol'];manager=protocol['managerFee'];reserve=protocol['reserve']
        self.assertEqual((roles.index(reserve),roles.index(manager)),(14,16));self.assertFalse(review['accountRoles'][14]['writable']);self.assertFalse(review['accountRoles'][16]['writable'])
        before=copy.deepcopy(self.host.host.original);current=copy.deepcopy(before);current[reserve]['lamports']=str(int(current[reserve]['lamports'])+7)
        raw=bytearray(base64.b64decode(current[manager]['dataBase64']));amount=int.from_bytes(raw[64:72],'little')+1;raw[64:72]=amount.to_bytes(8,'little');current[manager]['dataBase64']=base64.b64encode(raw).decode()
        self.a.jito_digests(self.c,self.a.candidate_phase(945),current,self.host.pre,_candidate=True)
        mint=self.a.oracle()['roles']['poolMint'];token=[{'accountIndex':16,'mint':mint,'owner':self.c.b58encode(raw[32:64]),'programId':current[manager]['owner'],
            'uiTokenAmount':{'amount':str(amount),'decimals':base64.b64decode(current[mint]['dataBase64'])[44]}}]
        pending={'reviewed':review,'metadata':metadata,'binding':self.host.binding,'preAccounts':before,'executionPre':{'accounts':current}}
        meta={'loadedAddresses':{'writable':plan['alt']['addresses'],'readonly':[]},'innerInstructions':o._candidate_runtime_cpi(self.a,self.c,945,pending),'preTokenBalances':token,'postTokenBalances':token}
        o._candidate_runtime_metadata(self.a,self.c,945,pending,meta,current)
        self.assertNotEqual([int(before[k]['lamports'])if before[k]['exists']else 0 for k in roles],[int(o._candidate_result_pre(pending)[k]['lamports'])if current[k]['exists']else 0 for k in roles])
        del pending['executionPre']
        with self.assertRaisesRegex(ValueError,'runtime_token_effects'):o._candidate_runtime_metadata(self.a,self.c,945,pending,meta,current)
    def test_execution_inclusion_quote_and_post_clock_floors_preserve_old_noattempt(self):
        state,q,v,host=self.fixture();pending=copy.deepcopy(state['phasePreparation']['pending']);pending['executionPre']=o._candidate_execution_pre(self.a,self.j,state)
        e=pending['executionPre'];e['quoteContextSlot']=str(int(e['contextSlot'])+2);floor=int(e['quoteContextSlot'])
        with self.assertRaisesRegex(ValueError,'execution_inclusion_floor'):o._candidate_execution_floor(pending,floor-1)
        clock=copy.deepcopy(e['clock']);clock.update(slot=str(floor),contextSlot=str(floor));o._candidate_execution_floor(pending,floor,clock)
        for key in ('slot','contextSlot','unixTimestamp'):
            bad=copy.deepcopy(clock);bad[key]=str(int(e['clock'][key])-1)
            with self.assertRaisesRegex(ValueError,'execution_post_clock'):o._candidate_execution_floor(pending,floor,bad)
        old=pending['preAccounts'];del pending['executionPre'];self.assertIs(o._candidate_result_pre(pending),old);o._candidate_execution_floor(pending,0,{'not':'used'})

    def test_new_proposal_timestamp_floor_is_only_existing_create_fourthvote_and_execute(self):
        state,q,v,host=self.fixture(seconds=10);pending=copy.deepcopy(state['phasePreparation']['pending']);pending['executionPre']=o._candidate_execution_pre(self.a,self.j,state)
        old=int(pending['preClock']['unixTimestamp']);later=int(pending['executionPre']['clock']['unixTimestamp']);self.assertGreater(later,old+1)
        facts={'proposal':{'timestamp':str(old+1),'status':5}};before=o.sha(o.encode(pending))
        for index in (940,941,942,943,944,945,946):
            phase=self.a.candidate_phase(index);new=self.host.rules.proposal_stage(phase,'POST',facts)[2]
            self.assertEqual(new,index in (940,944,945))
            if new:
                with self.assertRaisesRegex(ValueError,'execution_proposal_timestamp'):o._candidate_execution_proposal_floor(self.host.rules,pending,phase,facts)
            else:o._candidate_execution_proposal_floor(self.host.rules,pending,phase,facts)
            exact=copy.deepcopy(facts);exact['proposal']['timestamp']=str(later);o._candidate_execution_proposal_floor(self.host.rules,pending,phase,exact)
        self.assertEqual(o.sha(o.encode(pending)),before);del pending['executionPre']
        for index in (940,944,945):o._candidate_execution_proposal_floor(self.host.rules,pending,self.a.candidate_phase(index),facts)


class CandidatePublicCurrentCaptureTests(unittest.TestCase):
    """Injected raw callback components, never TLS/funded/native positives."""
    @classmethod
    def setUpClass(cls):
        cls.host=CandidateCurrentPhaseTests;cls.host.setUpClass();cls.a=cls.host.a;cls.j=cls.host.j;cls.c=cls.host.c;cls.manifest=cls.host.manifest
    def fixture(self):
        host=self.host(methodName='test_actual60_observed_pre_component_accepts_later_clock_and_source_without_refresh')
        state,q,v=host.fixture();frame=host.frame(q,v)['current'];return state,q,v,frame,host
    def test_single_exact_batch_captures_same_bytes_with_injected_not_tls_scope(self):
        state,q,v,frame,host=self.fixture();before=o.sha(o.encode(state));calls=[];body=self.c.b64(frame['responseBase64'],self.c.MAX_JSON)
        def transport(raw):
            calls.append(raw);self.assertEqual(raw,self.c.b64(frame['requestBase64'],65536));return 200,[['content-type','application/json']],body
        out=o._candidate_capture_current_phase(self.a,self.j,state,_transport=transport)
        self.assertEqual(len(calls),1);self.assertEqual([x['method']for x in self.c.strict_json(calls[0])],['getGenesisHash','getFeeForMessage','getMultipleAccounts','isBlockhashValid','getBlockHeight'])
        self.assertEqual(self.c.b64(out['publicFrame']['responseBase64'],self.c.MAX_JSON),body);self.assertEqual(out['publicFrame']['requestBase64'],frame['requestBase64'])
        http=self.c.strict_json(self.c.b64(out['publicFrame']['httpBase64'],16384));self.assertEqual(http['status'],200);self.assertEqual(http['headers'],[['content-type','application/json']])
        self.assertEqual(out['publicCapture']['frameSha256'],o.sha(self.c.encode(out['publicFrame'])));self.assertFalse(out['publicCapture']['tlsReceiptValidated']);self.assertIn('NOT_TLS',out['publicCapture']['transportMode'])
        self.assertEqual(o.sha(o.encode(state)),before);self.assertFalse(out['nativeAdmissionQualified']);self.assertFalse(out['executionPrestateRefreshed'])
        timing=out['publicCapture'];self.assertGreaterEqual(timing['transportElapsedSeconds'],0);self.assertGreaterEqual(timing['captureAndValidationElapsedSeconds'],timing['transportElapsedSeconds']);self.assertIn('SNAPSHOT_HELPER_ONLY',timing['timingScope'])
    def test_http_stale_and_rpc_refusals_keep_exact_body_and_no_retry(self):
        real=o.bounded_transport;active={}
        def observed(c,transport):
            active['collector']=c;return real(c,transport)
        for change,reason in (('http','prefix_http'),('clock','startup_clock_context'),('rpc','prefix_rpc_envelope'),('partial','synthetic_transport_failure')):
            state,q,v,frame,host=self.fixture();status=503 if change=='http'else 200;calls=[]
            if change=='clock':
                row=v[2]['value'][q[2][1][0].index(self.host.ref['roles']['clock'])];raw=bytearray(base64.b64decode(row['data'][0]));raw[:8]=(int(host.host.context)-1).to_bytes(8,'little');row['data'][0]=base64.b64encode(raw).decode()
            replies=[{'jsonrpc':'2.0','id':i+1,'result':value}for i,value in enumerate(v)]
            if change=='rpc':replies[0]['id']=2
            body=self.c.encode(replies)
            def transport(raw):
                calls.append(raw)
                if change=='partial':raise active['collector'].TransportFailure('synthetic_transport_failure',b'partial-public-body')
                return status,[],body
            with mock.patch.object(o,'bounded_transport',side_effect=observed):
                with self.assertRaisesRegex(Exception,reason)as caught:o._candidate_capture_current_phase(self.a,self.j,state,_transport=transport)
            evidence=caught.exception.publicCapture;self.assertEqual(len(calls),1);self.assertEqual(base64.b64decode(evidence['requestBase64']),calls[0]);self.assertFalse(evidence['tlsReceiptValidated'])
            self.assertGreaterEqual(evidence['transportElapsedSeconds'],0);self.assertGreaterEqual(evidence['captureAndValidationElapsedSeconds'],evidence['transportElapsedSeconds'])
            if change=='partial':self.assertIsNone(evidence['publicFrame']);self.assertEqual(base64.b64decode(evidence['partialResponseBase64']),b'partial-public-body')
            else:self.assertEqual(base64.b64decode(evidence['publicFrame']['responseBase64']),body)
    def test_callback_state_mutation_and_injected_source_refusal_preserve_returned_frame(self):
        real=o._candidate_preparation_sources
        for change,reason in (('state','public_retained_state_changed'),('source','injected_public_source_refusal')):
            state,q,v,frame,host=self.fixture();body=base64.b64decode(frame['responseBase64'])
            def sources():
                a,j,s,domain,unchanged=real()
                def check(boundary):
                    unchanged(boundary)
                    if change=='source'and boundary=='public_capture_after_callback':raise ValueError('injected_public_source_refusal')
                return a,j,s,domain,check
            def transport(raw):
                if change=='state':state['lastSlot']=str(int(state['lastSlot'])+1)
                return 200,[],body
            with mock.patch.object(o,'_candidate_preparation_sources',side_effect=sources):
                with self.assertRaisesRegex(ValueError,reason)as caught:o._candidate_capture_current_phase(self.a,self.j,state,_transport=transport)
            self.assertEqual(base64.b64decode(caught.exception.publicCapture['publicFrame']['responseBase64']),body)
    def test_private_state_uses_exact_bytes_while_report_digests_and_mutation_refusal_stay(self):
        digest=o.sha
        for mutate in (False,True):
            state,q,v,frame,host=self.fixture();body=base64.b64decode(frame['responseBase64']);snapshots=[self.j.encode(state)]
            def public_digest(raw):
                if raw in snapshots:raise AssertionError('private_state_must_use_exact_bytes')
                return digest(raw)
            def transport(raw):
                if mutate:
                    state['lastSlot']=str(int(state['lastSlot'])+1);snapshots.append(self.j.encode(state))
                return 200,[],body
            with self.subTest(mutation=mutate),mock.patch.object(o,'sha',new=public_digest):
                if mutate:
                    with self.assertRaisesRegex(ValueError,'public_retained_state_changed')as caught:
                        o._candidate_capture_current_phase(self.a,self.j,state,_transport=transport)
                    self.assertEqual(base64.b64decode(caught.exception.publicCapture['publicFrame']['responseBase64']),body)
                else:
                    out=o._candidate_capture_current_phase(self.a,self.j,state,_transport=transport)
                    self.assertEqual(out['inputSha256'],digest(o.encode({'current':out['publicFrame']})))
                    self.assertEqual(out['publicCapture']['frameSha256'],digest(self.c.encode(out['publicFrame'])))
    def test_actual_unfunded_null_capture_and_public_overrides_refuse_before_callback(self):
        with tempfile.TemporaryDirectory()as tmp:
            path=Path(tmp)/'public.candidate-controller';created=o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'CREATE','expectedTail':None,'data':self.manifest},path)
            request={'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'VERIFY_CURRENT_PHASE','expectedTail':created['tail'],'data':None};before={p.name:o.sha(p.read_bytes())for p in path.iterdir()}
            with mock.patch.object(o,'public_rpc_transport',side_effect=AssertionError('unexpected_public_callback'))as transport:
                with self.assertRaisesRegex(ValueError,'current_funding_required'):o.operate(request,path)
                for override in ({'_transport':lambda raw:None},{'live':True},{'live_sign':True},{'_signer':lambda raw:None}):
                    with self.assertRaisesRegex(ValueError,'controller_offline_only'):o.operate(request,path,**override)
                transport.assert_not_called()
            self.assertEqual(before,{p.name:o.sha(p.read_bytes())for p in path.iterdir()});self.assertFalse(path.with_name(path.name+'.evidence').exists())


class CandidateSignerFreshnessTests(unittest.TestCase):
    """Real temporary CREATE guards plus supplied component state; never funded."""
    @classmethod
    def setUpClass(cls):
        cls.host=CandidateSignerBoundaryTests;cls.host.setUpClass();cls.a=cls.host.a;cls.j=cls.host.j;cls.c=cls.host.c
    def fixture(self,expiry='18446744073709551615'):
        host=self.host(methodName='test_existing_public_contract_projection_binds_prepared_source_nullparent_and_lifetime');state,data=host.fixture(expiry)
        tmp=tempfile.TemporaryDirectory();self.addCleanup(tmp.cleanup);path=Path(tmp.name)/'fresh.candidate-controller'
        o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'CREATE','expectedTail':None,'data':state['manifest']},path)
        op=o.Journal(self.j,path,_candidate=True);self.addCleanup(op.close);state['controllerPath']=str(path)
        state['phaseSignAttempt']={**o._candidate_sign_phase_attempt(self.a,self.j,state,data,op.tail(),historical=True),'recordDigest':'e'*64}
        _,_,_,_,unchanged=o._candidate_preparation_sources();return state,data,op,unchanged
    def test_actual_journal_raw_guards_bracket_injected_not_tls_component_sample(self):
        state,data,op,unchanged=self.fixture();before=o.sha(o.encode(state));files={p.name:p.read_bytes()for p in op.path.iterdir()};events=[]
        capture=o._candidate_capture_current_phase;authorize=o._candidate_sign_authorization
        def observed(a,j,s):
            events.append('capture')
            return capture(a,j,s,_transport=lambda raw:(200,[],base64.b64decode(data['current']['responseBase64'])))
        def auth(*args,**kw):events.append('authorization');return authorize(*args,**kw)
        with mock.patch.object(o,'_candidate_capture_current_phase',side_effect=observed),mock.patch.object(o,'_candidate_sign_authorization',side_effect=auth):
            report=o._candidate_signer_current_boundary(self.a,self.j,op,state,unchanged)
        self.assertEqual(events,['authorization','capture','authorization']);self.assertFalse(report['publicCapture']['tlsReceiptValidated']);self.assertFalse(report['nativeAdmissionQualified'])
        request=self.c.strict_json(base64.b64decode(report['publicFrame']['requestBase64']));stored=self.c.strict_json(base64.b64decode(data['current']['responseBase64']))
        self.assertEqual(request[2]['params'][1]['minContextSlot'],stored[2]['result']['context']['slot']);self.assertGreater(request[2]['params'][1]['minContextSlot'],int(state['phasePreparation']['pending']['preContext']))
        self.assertEqual(o.sha(o.encode(state)),before);self.assertEqual(files,{p.name:p.read_bytes()for p in op.path.iterdir()});self.assertEqual(len(op.chain),1)
    def test_authorization_expiry_after_sample_preserves_raw_and_original_component(self):
        state,data,op,unchanged=self.fixture('105');clock=[100];before=o.sha(o.encode(state));capture=o._candidate_capture_current_phase
        def observed(a,j,s):
            def transport(raw):clock[0]=110;return 200,[],base64.b64decode(data['current']['responseBase64'])
            return capture(a,j,s,_transport=transport)
        with mock.patch.object(o.time,'time',side_effect=lambda:clock[0]),mock.patch.object(o,'_candidate_capture_current_phase',side_effect=observed):
            with self.assertRaisesRegex(ValueError,'authorization_expired')as caught:o._candidate_signer_current_boundary(self.a,self.j,op,state,unchanged)
        self.assertEqual(base64.b64decode(caught.exception.publicCapture['publicFrame']['responseBase64']),base64.b64decode(data['current']['responseBase64']))
        self.assertFalse(caught.exception.publicCapture['tlsReceiptValidated']);self.assertEqual(o.sha(o.encode(state)),before);self.assertEqual(len(op.chain),1)
    def test_stale_original_static_and_postcallback_raw_source_faults_refuse(self):
        capture=o._candidate_capture_current_phase
        for change,reason in (('hash','current_original_lifetime'),('sign_context','current_identity_or_context'),('sign_time','current_clock_regression'),('static','static_account_changed'),('raw','sign_current_raw_changed'),('source','injected_sign_current_source')):
            state,data,op,unchanged=self.fixture();body=base64.b64decode(data['current']['responseBase64']);replies=self.c.strict_json(body)
            if change=='hash':replies[3]['result']['value']=False
            if change=='sign_context':
                for index in (1,2,3):replies[index]['result']['context']['slot']=int(state['phasePreparation']['pending']['preContext'])
            if change=='sign_time':
                request=self.c.strict_json(base64.b64decode(data['current']['requestBase64']));row=replies[2]['result']['value'][request[2]['params'][0].index(self.a.candidate_oracle()['roles']['clock'])]
                clock=bytearray(base64.b64decode(row['data'][0]));clock[32:40]=int(state['phasePreparation']['pending']['preClock']['unixTimestamp']).to_bytes(8,'little',signed=True);row['data'][0]=base64.b64encode(clock).decode()
            if change=='static':
                request=self.c.strict_json(base64.b64decode(data['current']['requestBase64']));key=self.a.candidate_oracle()['roles']['g0'];replies[2]['result']['value'][request[2]['params'][0].index(key)]['lamports']+=1
            body=self.c.encode(replies)
            def observed(a,j,s):
                def transport(raw):
                    if change=='raw':(op.path/'00000000.json').write_bytes((op.path/'00000000.json').read_bytes()+b' ')
                    return 200,[],body
                return capture(a,j,s,_transport=transport)
            def sources(boundary):
                unchanged(boundary)
                if change=='source'and boundary=='sign_current_after_capture':raise ValueError('injected_sign_current_source')
            with mock.patch.object(o,'_candidate_capture_current_phase',side_effect=observed):
                with self.assertRaisesRegex(ValueError,reason)as caught:o._candidate_signer_current_boundary(self.a,self.j,op,state,sources)
            self.assertEqual(base64.b64decode(caught.exception.publicCapture['publicFrame']['responseBase64']),body);self.assertEqual(len(op.chain),1)
    def test_postcallback_retained_sidecar_mutation_refuses_without_adoption(self):
        state,data,op,unchanged=self.fixture();raw=o.encode({'hostComponentOnly':True});ref=o._candidate_prefix_store(self.c,self.j,op,{'cursor':0},'SIGN_PHASE_ATTEMPT',raw)
        # Synthetic raw/reference chain ONLY: no funded replay/accepted attempt.
        body={'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'sequence':'1','previousDigest':op.tail(),'lockIdentity':op.identity,'kind':'SIGN_PHASE_ATTEMPT','data':{'evidence':ref,'derived':{'index':0}}}
        record={**body,'digest':o.sha(o.encode(body))};self.j.write_file(op.fd,'00000001.json',o.encode(record));op.chain.append(record)
        payload=op.path.with_name(op.path.name+'.evidence')/ref['name']/'payload.json';capture=o._candidate_capture_current_phase
        def observed(a,j,s):
            def transport(request):payload.write_bytes(b'{}\n');return 200,[],base64.b64decode(data['current']['responseBase64'])
            return capture(a,j,s,_transport=transport)
        with mock.patch.object(o,'_candidate_capture_current_phase',side_effect=observed):
            with self.assertRaisesRegex(ValueError,'prefix_payload_hash')as caught:o._candidate_signer_current_boundary(self.a,self.j,op,state,unchanged)
        self.assertEqual(payload.read_bytes(),b'{}\n');self.assertEqual(len(op.chain),2);self.assertEqual(base64.b64decode(caught.exception.publicCapture['publicFrame']['responseBase64']),base64.b64decode(data['current']['responseBase64']))
    def test_actual_unfunded_and_invalid_signature_routes_remain_network_free(self):
        state,data,op,unchanged=self.fixture();request={'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'SIGN_PHASE_ATTEMPT','expectedTail':op.tail(),'data':data};op.close()
        host=self.host(methodName='test_original_backend_response_mapping_and_real_crypto_refuse_without_mutation');response=host.response(state)
        with mock.patch.object(o,'public_rpc_transport',side_effect=AssertionError('unexpected_rpc'))as transport:
            with self.assertRaisesRegex(ValueError,'prefix_funding_required'):o.operate(request,op.path)
            with self.assertRaisesRegex(ValueError,'signed_packet_or_receipt_rejected:zero_signature'):o._candidate_original_signer_response(self.a,self.j,state,response)
            with self.assertRaisesRegex(ValueError,'controller_offline_only'):o.operate(request,op.path,live_sign=True)
            transport.assert_not_called()


class CandidateFundingResultCaptureTests(unittest.TestCase):
    """Raw declaration components, real crypto refusals; no signed/TLS positive."""
    @classmethod
    def setUpClass(cls):
        from tools.test_startup_manifest import CandidateFundingReceiptTests
        cls.host=CandidateFundingReceiptTests;cls.host.setUpClass();cls.a=cls.host.a;cls.manifest=cls.host.manifest;cls.j,_,cls.c=cls.a._candidate_helpers()
    def fixture(self,signature=bytes(64)):
        receipt,_,_=self.host.fixture(signature);_,_,_,domain,unchanged=o._candidate_preparation_sources();data={k:receipt[k]for k in ('feeCapLamports','originalBlockhash','preflight')}
        prepared=o._candidate_prepare_funding(self.a,self.j,self.manifest,data,None,domain)
        original=o._candidate_controller_intent(self.a,self.j,self.manifest,{k:receipt[k]for k in ('wireBase64','feeCapLamports','originalBlockhash','preflight')});original['preparationRequestId']=prepared['request']['requestId']
        state={'manifest':self.manifest,'fundingPreparation':prepared,'intent':original,'funding':None}
        bodies=[self.c.encode([{'jsonrpc':'2.0','id':1,'result':self.c.GENESIS}]),base64.b64decode(receipt['finalized']['responseBase64']),base64.b64decode(receipt['poststate']['responseBase64'])]
        return state,receipt,bodies,unchanged
    def test_exact_three_raw_batches_keep_existing_receipt_bytes_and_false_flags(self):
        state,receipt,bodies,guard=self.fixture();calls=[]
        def transport(raw):calls.append(self.c.strict_json(raw));return 200,[],bodies[len(calls)-1]
        out=o._candidate_collect_funding_frames(self.a,self.j,state,guard,_transport=transport)
        self.assertEqual([[r['method']for r in batch]for batch in calls],[['getGenesisHash'],['getSignatureStatuses','getTransaction'],['getMultipleAccounts']])
        self.assertEqual(out['result']['receipt']['finalized']['responseBase64'],receipt['finalized']['responseBase64']);self.assertEqual(out['result']['receipt']['originalBlockhash'],receipt['originalBlockhash']);self.assertEqual(out['result']['receipt']['preflight'],receipt['preflight'])
        self.assertEqual(calls[2][0]['params'][1]['minContextSlot'],self.c.strict_json(bodies[1])[1]['result']['slot']);self.assertEqual(len(out['frames']['identity']),3)
        self.assertEqual(out['result']['expectedReceiptSha256'],self.a.sha(self.a.canonical(out['result']['receipt'])));self.assertFalse(out['suppliedReceiptVerifierAccepted']);self.assertFalse(out['fundingAuthenticated']);self.assertIn('NOT_TLS',out['transportMode'])
    def test_unknown_null_failed_or_foreign_original_holds_before_dependent_capture(self):
        for change in ('unknown','null','nonfinal','failed','wire'):
            state,receipt,bodies,guard=self.fixture();reply=self.c.strict_json(bodies[1]);calls=[]
            if change=='unknown':reply[0]['result']['value']=[None]
            elif change=='null':reply[1]['result']=None
            elif change=='nonfinal':reply[0]['result']['value'][0]['confirmationStatus']='confirmed'
            elif change=='failed':reply[1]['result']['meta']['err']={'InstructionError':[0,'Custom']}
            else:reply[1]['result']['transaction'][0]='AA=='
            bodies[1]=self.c.encode(reply)
            def transport(raw):calls.append(raw);return 200,[],bodies[len(calls)-1]
            with self.assertRaisesRegex(ValueError,'original_unknown_hold')as caught:o._candidate_collect_funding_frames(self.a,self.j,state,guard,_transport=transport)
            self.assertEqual(len(calls),2);self.assertEqual(base64.b64decode(caught.exception.fundingResultCapture['frames']['finalized']['responseBase64']),bodies[1])
    def test_complete_source_fault_and_genuine_partial_keep_original_raw(self):
        real=o.bounded_transport;active={}
        def observed(c,transport):active['c']=c;return real(c,transport)
        for change in ('source','http','partial'):
            state,receipt,bodies,unchanged=self.fixture();calls=[]
            def guard(boundary):
                unchanged(boundary)
                if change=='source'and boundary=='funding_identity_after_callback':raise ValueError('injected_funding_source_failure')
            def transport(raw):
                calls.append(raw)
                if change=='partial'and len(calls)==2:raise active['c'].TransportFailure('synthetic_funding_partial',b'partial-original-public-body')
                return (503 if change=='http'else 200),[],bodies[len(calls)-1]
            with mock.patch.object(o,'bounded_transport',side_effect=observed):
                with self.assertRaisesRegex(Exception,'injected_funding_source_failure|candidate_funding_http|synthetic_funding_partial')as caught:o._candidate_collect_funding_frames(self.a,self.j,state,guard,_transport=transport)
            frames=caught.exception.fundingResultCapture['frames'];self.assertEqual(base64.b64decode(frames['identity']['responseBase64']),bodies[0])
            if change=='partial':self.assertEqual(base64.b64decode(frames['finalized']['partialResponseBase64']),b'partial-original-public-body');self.assertNotIn('httpBase64',frames['finalized'])
    def test_actual_unprepared_hold_and_real_zero_invalid_refuse_before_network(self):
        with tempfile.TemporaryDirectory()as tmp:
            path=Path(tmp)/'funding.candidate-controller';created=o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'CREATE','expectedTail':None,'data':self.manifest},path)
            state,receipt,bodies,guard=self.fixture();request={'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'FUNDING_INTENT','expectedTail':created['tail'],'data':state['intent']['intent']};held=o.operate(request,path)
            before={p.name:p.read_bytes()for p in path.iterdir()}
            with mock.patch.object(o,'public_rpc_transport',side_effect=AssertionError('unexpected_rpc'))as transport:
                with self.assertRaisesRegex(ValueError,'capture_prepared_original_required'):o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'FUNDING_RESULT','expectedTail':held['tail'],'data':None},path)
                op=o.Journal(self.j,path,_candidate=True)
                try:
                    for signature,reason in ((bytes(64),'zero_signature'),(bytes([1])*64,'signature_invalid')):
                        component,_,_,unchanged=self.fixture(signature);component['controllerPath']=str(path)
                        with self.assertRaisesRegex(ValueError,'signed_packet_or_receipt_rejected:'+reason):o._candidate_capture_funding_result(self.a,self.j,op,component,unchanged)
                finally:op.close()
                transport.assert_not_called()
            self.assertEqual(before,{p.name:p.read_bytes()for p in path.iterdir()});self.assertEqual(held['status'],'UNKNOWN_HOLD_ORIGINAL_FUNDING_ONLY')


class CandidateNativeCallbackTests(unittest.TestCase):
    """Host refusal/runner components only; no funded callback or native positive."""
    @classmethod
    def setUpClass(cls):
        cls.host=CandidateSignerFreshnessTests;cls.host.setUpClass();cls.a=cls.host.a;cls.j=cls.host.j;cls.c=cls.host.c
    def fixture(self):
        host=self.host(methodName='test_actual_journal_raw_guards_bracket_injected_not_tls_component_sample')
        state,data,op,unchanged=host.fixture();self.addCleanup(host.doCleanups)
        derived={k:state['phaseSignAttempt'][k]for k in ('input','identity')};state['phaseSignAttempt']=None
        return state,derived,op,unchanged
    def runner(self,output,*,partial=False,sink=None):
        # Actual runner, fake process/pipes only; no real backend, private reader or keys.
        j=mock.Mock(NODE=Path('/pinned/node'));j.pins.return_value=REPO;j.parse.side_effect=json.loads
        stdin=mock.Mock(closed=False);stdout=mock.Mock(closed=False);stdin.fileno.return_value=101;stdout.fileno.return_value=102
        process=mock.Mock(stdin=stdin,stdout=stdout);process.poll.return_value=0;process.wait.return_value=0
        selector=mock.Mock();registered={};selector.register.side_effect=lambda stream,event:registered.update({stream:event})
        selector.unregister.side_effect=lambda stream:registered.pop(stream);selector.get_map.side_effect=lambda:registered
        selector.select.side_effect=lambda wait:[(mock.Mock(fileobj=stream),event)for stream,event in list(registered.items())]
        reads=iter((output,TimeoutError('synthetic pipe failure')if partial else b''))
        def read(fd,size):
            value=next(reads)
            if isinstance(value,Exception):raise value
            return value
        with mock.patch.object(o.subprocess,'Popen',return_value=process)as popen,mock.patch.object(o.selectors,'DefaultSelector',return_value=selector),mock.patch.object(o.os,'set_blocking'),mock.patch.object(o.os,'write',side_effect=lambda fd,raw:len(raw)),mock.patch.object(o.os,'read',side_effect=read):
            try:return o.invoke_signer(j,{'signerProfilePath':'/not-read/public-fixture'},_raw_capture=sink)
            finally:
                self.assertEqual(popen.call_args.args[0],['/pinned/node',str(REPO/'tools/startup_signer_backend.cjs'),'--allow-live-sign'])
                self.assertIs(popen.call_args.kwargs['stderr'],o.subprocess.DEVNULL)
    def test_optin_runner_captures_literal_stdout_before_parse_or_refusal(self):
        literal=b' { "schema" : "public-response" } \n';seen=[]
        self.assertEqual(self.runner(literal,sink=lambda raw,complete:seen.append((raw,complete))),{'schema':'public-response'})
        self.assertEqual(seen,[(literal,True)])
        malformed=b'{"public-response":';seen=[]
        with self.assertRaisesRegex(o.Stop,'^signer_attempt_unresolved$'):self.runner(malformed,sink=lambda raw,complete:seen.append((raw,complete)))
        self.assertEqual(seen,[(malformed,True)])
    def test_runner_genuine_partial_is_bounded_and_default_failure_discards(self):
        raw=b'public-partial-stdout';seen=[]
        with self.assertRaisesRegex(o.Stop,'^signer_attempt_unresolved$'):self.runner(raw,partial=True,sink=lambda raw,complete:seen.append((raw,complete)))
        self.assertEqual(seen,[(raw,False)])
        with self.assertRaisesRegex(o.Stop,'^signer_attempt_unresolved$')as caught:self.runner(raw,partial=True)
        self.assertFalse(hasattr(caught.exception,'nativeSignerResponse'));self.assertNotIn(raw.decode(),str(caught.exception))
        seen=[]
        with self.assertRaisesRegex(o.Stop,'^signer_attempt_unresolved$'):self.runner(b'x'*262145,sink=lambda raw,complete:seen.append((raw,complete)))
        self.assertEqual(seen,[(b'x'*262144,False)])
    def test_unarmed_commit_consumes_continuation_once_and_reopen_cannot_mint(self):
        state,derived,op,unchanged=self.fixture();before={p.name:p.read_bytes()for p in op.path.iterdir()}
        continuation=o._candidate_native_phase_continuation(self.a,self.j,op,state,derived,{},'/not-read/public-fixture',unchanged)
        with mock.patch.object(o,'invoke_signer')as invoke:
            with self.assertRaisesRegex(ValueError,'native_new_commit_required'):continuation(op.chain[-1])
            with self.assertRaisesRegex(ValueError,'native_continuation_consumed'):continuation(op.chain[-1])
            state['phaseSignAttempt']={**derived,'recordDigest':'e'*64}
            with self.assertRaisesRegex(ValueError,'native_fresh_reservation_only'):o._candidate_native_phase_continuation(self.a,self.j,op,state,derived,{},'/not-read/public-fixture',unchanged)
            invoke.assert_not_called()
        self.assertEqual(before,{p.name:p.read_bytes()for p in op.path.iterdir()})
    def test_host_new_record_without_funding_fails_real_replay_before_callback(self):
        state,derived,op,unchanged=self.fixture();raw=o.encode(derived['input'])
        ref=o._candidate_prefix_store(self.c,self.j,op,state,'SIGN_PHASE_ATTEMPT',raw)
        continuation=o._candidate_native_phase_continuation(self.a,self.j,op,state,derived,ref,'/not-read/public-fixture',unchanged)
        body={'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'sequence':str(len(op.chain)),'previousDigest':op.tail(),'lockIdentity':op.identity,
            'kind':'SIGN_PHASE_ATTEMPT','data':{'evidence':ref,'derived':derived['identity']}}
        # Memory-only invalid history: real replay must reject, never accepted as a durable funded append.
        record={**body,'digest':o.sha(o.encode(body))};op.chain.append(record)
        with mock.patch.object(o,'invoke_signer')as invoke:
            with self.assertRaisesRegex(ValueError,'prefix_funding_required'):continuation(record)
            with self.assertRaisesRegex(ValueError,'native_continuation_consumed'):continuation(record)
            invoke.assert_not_called()
        self.assertEqual({p.name for p in op.path.iterdir()},{'lock','00000000.json'})
        self.assertEqual(o.encode(o._candidate_prefix_load(self.j,op.path,ref,'1','SIGN_PHASE_ATTEMPT',0)),raw)
    def test_new_commit_source_fault_consumes_once_without_callback_or_adoption(self):
        state,derived,op,unchanged=self.fixture();events=[]
        def fault(boundary):unchanged(boundary);events.append(boundary);raise ValueError('injected_native_source_fault')
        continuation=o._candidate_native_phase_continuation(self.a,self.j,op,state,derived,{},'/not-read/public-fixture',fault)
        body={'kind':'SIGN_PHASE_ATTEMPT','sequence':str(len(op.chain)),'previousDigest':op.tail(),'data':{'evidence':{},'derived':derived['identity']}}
        op.chain.append(body)
        with mock.patch.object(o,'invoke_signer')as invoke:
            with self.assertRaisesRegex(ValueError,'injected_native_source_fault'):continuation(body)
            with self.assertRaisesRegex(ValueError,'native_continuation_consumed'):continuation(body)
            invoke.assert_not_called()
        self.assertEqual(events,['native_after_reservation']);self.assertEqual({p.name for p in op.path.iterdir()},{'lock','00000000.json'})
    def test_actual_public_native_overrides_and_private_nonfresh_actions_refuse(self):
        state,derived,op,unchanged=self.fixture();request={'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'SIGN_PHASE_ATTEMPT','expectedTail':op.tail(),'data':{}}
        before={p.name:p.read_bytes()for p in op.path.iterdir()}
        with mock.patch.object(o,'invoke_signer')as invoke,mock.patch.object(o,'public_rpc_transport')as rpc:
            for options in ({'live':True},{'live_sign':True},{'_signer':lambda value:None},{'authorization_raw':b'public'}):
                with self.assertRaisesRegex(ValueError,'controller_offline_only'):o.operate(request,op.path,**options)
            with self.assertRaisesRegex(ValueError,'native_fresh_action_only'):o._candidate_controller_operate({**request,'action':'INSPECT'},op.path,_native_profile_path='/not-read/public-fixture')
            invoke.assert_not_called();rpc.assert_not_called()
        self.assertEqual(before,{p.name:p.read_bytes()for p in op.path.iterdir()})


class CandidatePhaseTransportTests(unittest.TestCase):
    """Readonly raw constructors and real refusal boundaries, never funded send."""
    @classmethod
    def setUpClass(cls):
        cls.host=CandidateNativeCallbackTests;cls.host.setUpClass();cls.a=cls.host.a;cls.j=cls.host.j;cls.c=cls.host.c
    def fixture(self,index=0):
        host=self.host(methodName='test_unarmed_commit_consumes_continuation_once_and_reopen_cannot_mint')
        state,derived,op,guard=host.fixture();self.addCleanup(host.doCleanups)
        state['pending']=copy.deepcopy(state['phasePreparation']['pending']);state['cursor']=index
        pending=state['pending'];current=self.c.strict_json(base64.b64decode(derived['input']['current']['responseBase64']))
        pending['quoteContexts']=[str(current[i]['result']['context']['slot'])for i in (1,3)]
        if index==945:pending['input']['wireBase64']=CandidateCurrentPhaseTests.binding['unsignedInitializerPlan']['packets'][8]['wireBase64']
        slot=max(int(state['lastSlot']),int(pending['preContext']),*map(int,pending['quoteContexts']))+2
        status={'slot':slot,'err':None,'confirmationStatus':'finalized'}
        tx={'slot':slot,'version':0 if index==945 else 'legacy','transaction':[pending['input']['wireBase64'],'base64'],
            'meta':{'err':None,'fee':int(pending['feeQuoteLamports'])}}
        bodies=[self.c.encode([{'jsonrpc':'2.0','id':1,'result':self.c.GENESIS}]),
            self.c.encode([{'jsonrpc':'2.0','id':1,'result':{'context':{'slot':slot},'value':[status]}},{'jsonrpc':'2.0','id':2,'result':tx}]),
            self.c.encode([{'jsonrpc':'2.0','id':1,'result':{'context':{'slot':slot},'value':[]}}])]
        return state,derived,op,guard,bodies
    def test_exact_original_three_batches_and_v0_use_fixed_coverage_without_lifetime_refresh(self):
        for index,count in ((0,60),(1,10),(945,60)):
            state,_,_,guard,bodies=self.fixture(index);calls=[];before=o.sha(o.encode(state))
            def transport(raw):calls.append(self.c.strict_json(raw));return 200,[],bodies[len(calls)-1]
            out=o._candidate_collect_phase_frames(self.a,self.j,state,guard,_transport=transport)
            self.assertEqual([[r['method']for r in batch]for batch in calls],[['getGenesisHash'],['getSignatureStatuses','getTransaction'],['getMultipleAccounts']])
            self.assertEqual(len(calls[2][0]['params'][0]),count);self.assertEqual(calls[2][0]['params'][1]['minContextSlot'],self.c.strict_json(bodies[1])[1]['result']['slot'])
            self.assertEqual(base64.b64decode(out['result']['finalized']['responseBase64']),bodies[1]);self.assertIn('NOT_TLS',out['transportMode'])
            self.assertFalse(out['suppliedPhaseVerifierAccepted']);self.assertFalse(out['chainAuthenticated']);self.assertEqual(o.sha(o.encode(state)),before)
            if index==945:
                wire=base64.b64decode(state['pending']['input']['wireBase64']);self.assertEqual(wire[1+64*wire[0]],128)
    def test_unknown_failed_wrong_wire_version_and_fee_stop_before_poststate(self):
        for change in ('unknown','null','nonfinal','failed','wire','version','fee'):
            state,_,_,guard,bodies=self.fixture();reply=self.c.strict_json(bodies[1]);calls=[]
            if change=='unknown':reply[0]['result']['value']=[None]
            elif change=='null':reply[1]['result']=None
            elif change=='nonfinal':reply[0]['result']['value'][0]['confirmationStatus']='confirmed'
            elif change=='failed':reply[1]['result']['meta']['err']={'InstructionError':[0,'Custom']}
            elif change=='wire':reply[1]['result']['transaction'][0]='AA=='
            elif change=='version':reply[1]['result']['version']=0
            else:reply[1]['result']['meta']['fee']+=1
            bodies[1]=self.c.encode(reply)
            def transport(raw):calls.append(raw);return 200,[],bodies[len(calls)-1]
            with self.assertRaisesRegex(ValueError,'original_unknown_hold')as caught:o._candidate_collect_phase_frames(self.a,self.j,state,guard,_transport=transport)
            self.assertEqual(len(calls),2);self.assertEqual(base64.b64decode(caught.exception.phaseResultCapture['frames']['finalized']['responseBase64']),bodies[1])
    def test_http_source_fault_and_genuine_partial_preserve_same_bytes_without_retry(self):
        real=o.bounded_transport;active={}
        def observed(c,transport):active['c']=c;return real(c,transport)
        for change in ('http','source','partial'):
            state,_,_,unchanged,bodies=self.fixture();calls=[]
            def guard(boundary):
                unchanged(boundary)
                if change=='source'and boundary=='phase_identity_after_callback':raise ValueError('injected_phase_source_fault')
            def transport(raw):
                calls.append(raw)
                if change=='partial'and len(calls)==2:raise active['c'].TransportFailure('synthetic_phase_partial',b'original-public-partial')
                return (503 if change=='http'else 200),[],bodies[len(calls)-1]
            with mock.patch.object(o,'bounded_transport',side_effect=observed):
                with self.assertRaisesRegex(Exception,'prefix_http|injected_phase_source_fault|synthetic_phase_partial')as caught:o._candidate_collect_phase_frames(self.a,self.j,state,guard,_transport=transport)
            frames=caught.exception.phaseResultCapture['frames'];self.assertEqual(base64.b64decode(frames['identity']['responseBase64']),bodies[0])
            self.assertEqual(len(calls),2 if change=='partial'else 1)
            if change=='partial':self.assertEqual(base64.b64decode(frames['finalized']['partialResponseBase64']),b'original-public-partial');self.assertNotIn('httpBase64',frames['finalized'])
    def test_real_zero_invalid_crypto_and_private_selector_refuse_before_network(self):
        state,_,op,guard,_=self.fixture();before={p.name:p.read_bytes()for p in op.path.iterdir()}
        with mock.patch.object(o,'public_rpc_transport',side_effect=AssertionError('unexpected_rpc'))as transport:
            for signature,reason in ((bytes(64),'zero_signature'),(bytes([1])*64,'signature_invalid')):
                component=copy.deepcopy(state);pending=component['pending'];wire=base64.b64decode(pending['input']['wireBase64'])
                wire=wire[:1]+signature*wire[0]+wire[1+64*wire[0]:];pending['input']['wireBase64']=base64.b64encode(wire).decode()
                pending['identity'].update(wireSha256=o.sha(wire),signatureHex=signature.hex())
                with self.assertRaisesRegex(ValueError,'signed_packet_or_receipt_rejected:'+reason):o._candidate_capture_phase_result(self.a,self.j,op,component,guard)
            request={'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'PHASE_RESULT','expectedTail':op.tail(),'data':None}
            for selector in (True,1,'true'):
                with self.assertRaisesRegex(ValueError,'phase_submission_private_scope'):o._candidate_controller_operate(request,op.path,_submit_original=selector)
            with self.assertRaisesRegex(ValueError,'controller_offline_only'):o.operate(request,op.path,live=True)
            transport.assert_not_called()
        self.assertEqual(before,{p.name:p.read_bytes()for p in op.path.iterdir()})
        with tempfile.TemporaryDirectory()as tmp:
            path=Path(tmp)/'phase.candidate-controller';created=o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'CREATE','expectedTail':None,'data':state['manifest']},path)
            files={p.name:p.read_bytes()for p in path.iterdir()}
            with mock.patch.object(o,'public_rpc_transport',side_effect=AssertionError('unexpected_rpc'))as transport:
                with self.assertRaisesRegex(ValueError,'prefix_funding_required'):o.operate({**request,'expectedTail':created['tail']},path)
                transport.assert_not_called()
            self.assertEqual(files,{p.name:p.read_bytes()for p in path.iterdir()})
    def test_send_continuation_requires_same_new_commit_and_is_consumed_on_refusal(self):
        state,derived,op,guard,_=self.fixture();pending=state['pending'];state['pending']=None;state['phaseSignAttempt']={**derived,'recordDigest':'e'*64}
        entry={'requestId':'a'*64,'wireBase64':pending['input']['wireBase64'],'derived':pending['identity']}
        continuation=o._candidate_phase_send_continuation(self.a,self.j,op,state,pending,entry,guard)
        with mock.patch.object(o,'public_rpc_transport')as transport:
            with self.assertRaisesRegex(ValueError,'send_new_commit_required'):continuation(op.chain[-1])
            with self.assertRaisesRegex(ValueError,'send_continuation_consumed'):continuation(op.chain[-1])
            state['pending']=pending
            with self.assertRaisesRegex(ValueError,'send_fresh_prepared_original_only'):o._candidate_phase_send_continuation(self.a,self.j,op,state,pending,entry,guard)
            transport.assert_not_called()
        self.assertEqual({p.name for p in op.path.iterdir()},{'lock','00000000.json'})


class CandidateFundingInputCaptureTests(unittest.TestCase):
    """Durable unsigned host preparation with explicit NOT_TLS inputs, never funding."""
    @classmethod
    def setUpClass(cls):
        cls.host=CandidateFundingPreparationTests;cls.host.setUpClass();cls.a=cls.host.a;cls.j,_,cls.c=cls.a._candidate_helpers();cls.manifest=cls.host.manifest
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.path=Path(self.tmp.name)/'input.candidate-controller'
        out=o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'CREATE','expectedTail':None,'data':self.manifest},self.path);self.tail=out['tail']
        receipt,_,_=self.host.host.fixture();self.data={'feeCapLamports':receipt['feeCapLamports']}
        self.bodies=[base64.b64decode(receipt[k]['responseBase64'])for k in ('originalBlockhash','preflight')];self.calls=[]
    def call(self,data):return o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'PREPARE_FUNDING','expectedTail':self.tail,'data':data},self.path)
    def transport(self,raw):self.calls.append(self.c.strict_json(raw));return 200,[],self.bodies[len(self.calls)-1]
    def test_actual_shorthand_captures_exact_two_batches_and_durable_unsigned_readback(self):
        actual=o._candidate_capture_funding_input
        def observed(a,j,op,state,data,domain,unchanged):return actual(a,j,op,state,data,domain,unchanged,_transport=self.transport)
        with mock.patch.object(o,'_candidate_capture_funding_input',side_effect=observed),mock.patch.object(o,'public_rpc_transport',side_effect=AssertionError('unexpected_rpc'))as rpc:
            out=self.call(self.data);rpc.assert_not_called()
        prepared=out['fundingPreparation'];wire=base64.b64decode(prepared['packet']['wireBase64'])
        self.assertEqual(len(wire),215);self.assertEqual(wire[:65],b'\1'+bytes(64));self.assertEqual(prepared['input']['feeCapLamports'],self.data['feeCapLamports'])
        self.assertEqual([[r['method']for r in batch]for batch in self.calls],[['getGenesisHash','getLatestBlockhash'],['getFeeForMessage','getMultipleAccounts','getBlockHeight','isBlockhashValid']])
        origin=self.c.strict_json(self.bodies[0])[1]['result']['context']['slot'];self.assertEqual(self.calls[1][1]['params'][1]['minContextSlot'],origin)
        self.assertEqual(self.calls[1][0]['params'][0],prepared['packet']['reviewed']['messageBase64']);self.assertEqual(prepared['packet']['reviewed']['requiredSigners'],[self.a.candidate_oracle()['roles']['source']])
        self.assertIn('NOT_TLS',out['fundingInputCapture']['transportMode']);self.assertFalse(out['fundingInputCapture']['freshAfterDurability']);self.assertFalse(out['liveReady'])
        files={p.name:p.read_bytes()for p in self.path.iterdir()};self.tail=out['tail']
        read=o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'INSPECT','expectedTail':self.tail,'data':None},self.path)
        self.assertEqual(read['fundingPreparation'],prepared);self.assertEqual(files,{p.name:p.read_bytes()for p in self.path.iterdir()})
    def test_caps_are_explicit_positive_ascii_u64_and_invalid_inputs_never_capture(self):
        with mock.patch.object(o,'public_rpc_transport',side_effect=AssertionError('unexpected_rpc'))as rpc:
            for cap in (0,True,'0','01','١','18446744073709551616'):
                with self.assertRaisesRegex(ValueError,'u64_string|positive_cap'):o._candidate_collect_funding_input_frames(self.a,self.j,{}, {'feeCapLamports':cap},lambda boundary:None)
            for data in (None,{}, {'feeCapLamports':'0'}):
                with self.assertRaisesRegex(ValueError,'object_fields|positive_cap'):self.call(data)
            rpc.assert_not_called()
        self.assertEqual({p.name for p in self.path.iterdir()},{'lock','00000000.json'})
    def test_prepared_or_intent_or_funding_blocks_recollection_without_transport(self):
        actual=o._candidate_capture_funding_input
        with mock.patch.object(o,'_candidate_capture_funding_input',side_effect=lambda a,j,op,state,data,domain,unchanged:actual(a,j,op,state,data,domain,unchanged,_transport=self.transport)):
            out=self.call(self.data)
        self.tail=out['tail'];before={p.name:p.read_bytes()for p in self.path.iterdir()}
        with mock.patch.object(o,'public_rpc_transport',side_effect=AssertionError('unexpected_rpc'))as rpc:
            with self.assertRaisesRegex(ValueError,'once_before_intent'):self.call(self.data)
            for field in ('fundingPreparation','intent','funding'):
                state={'fundingPreparation':None,'intent':None,'funding':None};state[field]={}
                with self.assertRaisesRegex(ValueError,'once_before_intent'):o._candidate_collect_funding_input_frames(self.a,self.j,state,self.data,lambda boundary:None)
            rpc.assert_not_called()
        self.assertEqual(len(self.calls),2);self.assertEqual(before,{p.name:p.read_bytes()for p in self.path.iterdir()})
    def test_identity_and_existing_account_clock_fee_lifetime_rules_refuse_without_append(self):
        op=o.Journal(self.j,self.path,_candidate=True);self.addCleanup(op.close);state=o._candidate_controller_replay(self.j,op.chain,self.path)
        _,_,_,domain,guard=o._candidate_preparation_sources();before={p.name:p.read_bytes()for p in self.path.iterdir()}
        reasons={'genesis':'funding_input_identity','context':'funding_preparation_context','source':'funding_preparation_system','h2':'funding_preparation_sufficiency','clock':'startup_clock_identity','quote':'u64_range','height':'controller_declared_preflight','valid':'controller_declared_preflight'}
        for change,reason in reasons.items():
            original=self.c.strict_json(self.bodies[0]);pre=self.c.strict_json(self.bodies[1]);calls=[]
            if change=='genesis':original[0]['result']='11111111111111111111111111111111'
            elif change=='context':pre[1]['result']['context']['slot']=original[1]['result']['context']['slot']-1
            elif change=='source':pre[1]['result']['value'][0]['owner']=self.a.candidate_oracle()['roles']['g0']
            elif change=='h2':pre[1]['result']['value'][1]['lamports']+=1
            elif change=='clock':pre[1]['result']['value'][2]['owner']=self.a.SYSTEM
            elif change=='quote':pre[0]['result']['value']=None
            elif change=='height':pre[2]['result']=original[1]['result']['value']['lastValidBlockHeight']-39
            else:pre[3]['result']['value']=False
            bodies=[self.c.encode(original),self.c.encode(pre)]
            def transport(raw):calls.append(raw);return 200,[],bodies[len(calls)-1]
            with self.assertRaisesRegex(ValueError,reason):o._candidate_capture_funding_input(self.a,self.j,op,state,self.data,domain,guard,_transport=transport)
            self.assertEqual(len(calls),1 if change=='genesis'else 2);self.assertEqual(before,{p.name:p.read_bytes()for p in self.path.iterdir()})
    def test_http_partial_source_and_raw_faults_preserve_evidence_without_retry(self):
        op=o.Journal(self.j,self.path,_candidate=True);self.addCleanup(op.close);state=o._candidate_controller_replay(self.j,op.chain,self.path)
        _,_,_,domain,unchanged=o._candidate_preparation_sources();actual=o.bounded_transport;active={}
        def observed(c,transport):active['c']=c;return actual(c,transport)
        for change in ('http','partial','source','raw'):
            calls=[]
            def guard(boundary):
                unchanged(boundary)
                if change=='source'and boundary=='funding_input_originalBlockhash_after_callback':raise ValueError('injected_input_source_fault')
            def transport(raw):
                calls.append(raw)
                if change=='partial':raise active['c'].TransportFailure('synthetic_input_partial',b'partial-original-input')
                if change=='raw':
                    path=self.path/'00000000.json';path.write_bytes(path.read_bytes()+b'\n')
                return (503 if change=='http'else 200),[],self.bodies[0]
            with mock.patch.object(o,'bounded_transport',side_effect=observed):
                with self.assertRaisesRegex(Exception,'funding_http|synthetic_input_partial|injected_input_source_fault|input_raw_changed')as caught:o._candidate_capture_funding_input(self.a,self.j,op,state,self.data,domain,guard,_transport=transport)
            frame=caught.exception.fundingInputCapture['frames']['originalBlockhash'];self.assertEqual(len(calls),1)
            if change=='partial':self.assertEqual(base64.b64decode(frame['partialResponseBase64']),b'partial-original-input');self.assertNotIn('httpBase64',frame)
            else:self.assertEqual(base64.b64decode(frame['responseBase64']),self.bodies[0])
            self.assertFalse(caught.exception.fundingInputCapture['unsignedPreparationVerified']);self.assertEqual({p.name for p in self.path.iterdir()},{'lock','00000000.json'})


class CandidateFundingSubmissionTests(unittest.TestCase):
    """Current/bank/once-only host components; no accepted signed send or TLS mock."""
    @classmethod
    def setUpClass(cls):
        cls.host=CandidateFundingPreparationTests;cls.host.setUpClass();cls.a=cls.host.a;cls.j,_,cls.c=cls.a._candidate_helpers()
    def setUp(self):
        self.hostcase=self.host(methodName='test_actual_unsigned_prepare_exact_durable_readback_and_host_only_scope');self.hostcase.setUp();self.addCleanup(self.hostcase.doCleanups)
        self.prepared=self.hostcase.call('PREPARE_FUNDING',self.hostcase.data)['fundingPreparation']
    def opened(self):
        op=o.Journal(self.j,self.hostcase.path,_candidate=True);self.addCleanup(op.close)
        state=o._candidate_controller_replay(self.j,op.chain,op.path);_,_,_,_,unchanged=o._candidate_preparation_sources();return op,state,unchanged
    def body(self,delta=0,advance=10,height=101):
        replies=self.c.strict_json(base64.b64decode(self.prepared['input']['preflight']['responseBase64']));slot=int(self.prepared['clock']['contextSlot'])+advance
        quote,before,_,valid=[copy.deepcopy(x['result'])for x in replies]
        for value in (quote,before,valid):value['context']['slot']=slot
        before['value'][0]['lamports']+=delta;clock=bytearray(base64.b64decode(before['value'][2]['data'][0]))
        clock[:8]=slot.to_bytes(8,'little');clock[32:40]=(int(self.prepared['clock']['unixTimestamp'])+advance).to_bytes(8,'little',signed=True);before['value'][2]['data'][0]=base64.b64encode(clock).decode()
        return self.c.encode([{'jsonrpc':'2.0','id':i+1,'result':v}for i,v in enumerate((self.c.GENESIS,quote,before,height,valid))])
    def current(self,op,state,guard,body,previous=None):
        return o._candidate_capture_funding_current(self.a,self.j,op,state,guard,_previous=previous,_transport=lambda raw:(200,[],body))
    def test_current_source_can_change_while_quote_packet_h2_and_original_frames_stay_fixed(self):
        op,state,guard=self.opened();original=o.encode(state['fundingPreparation']);first=self.current(op,state,guard,self.body(1_000_000))
        second=self.current(op,state,guard,self.body(-1_000_000,20,102),first)
        self.assertNotEqual(first['sourceLamports'],second['sourceLamports']);self.assertEqual(second['feeQuoteLamports'],self.prepared['packet']['feeQuoteLamports'])
        self.assertEqual(o.encode(state['fundingPreparation']),original);self.assertIn('NOT_TLS',second['transportMode']);self.assertFalse(second['liveReady'])
        queries=self.c.strict_json(base64.b64decode(second['frame']['requestBase64']))
        self.assertEqual([q['method']for q in queries],['getGenesisHash','getFeeForMessage','getMultipleAccounts','getBlockHeight','isBlockhashValid'])
        self.assertEqual(queries[1]['params'][0],self.prepared['packet']['reviewed']['messageBase64']);self.assertEqual(queries[4]['params'][0],self.prepared['packet']['lifetime']['recentBlockhash'])
        self.assertEqual(queries[2]['params'][1]['minContextSlot'],int(first['contextSlot']))
    def test_immutable_fee_current_context_clock_system_h2_sufficiency_hash_and_margin_refuse(self):
        op,state,guard=self.opened();first=self.current(op,state,guard,self.body())
        reasons={'fee':'immutable_quote','context':'identity_context','time':'current_clock','owner':'current_system','h2':'sufficiency_h2','source':'sufficiency_h2','hash':'original_lifetime','height':'original_lifetime'}
        for change,reason in reasons.items():
            rows=self.c.strict_json(self.body(0,20,102));accounts=rows[2]['result']['value']
            if change=='fee':rows[1]['result']['value']+=1
            elif change=='context':rows[1]['result']['context']['slot']=int(first['contextSlot'])-1
            elif change=='time':
                raw=bytearray(base64.b64decode(accounts[2]['data'][0]));raw[32:40]=(int(first['clock']['unixTimestamp'])-1).to_bytes(8,'little',signed=True);accounts[2]['data'][0]=base64.b64encode(raw).decode()
            elif change=='owner':accounts[0]['owner']=self.a.candidate_oracle()['roles']['g0']
            elif change=='h2':accounts[1]['lamports']+=1
            elif change=='source':accounts[0]['lamports']=4431889280+int(self.prepared['packet']['feeQuoteLamports'])-1
            elif change=='hash':rows[4]['result']['value']=False
            else:rows[3]['result']=int(self.prepared['packet']['lifetime']['lastValidBlockHeight'])-39
            with self.assertRaisesRegex(ValueError,reason):self.current(op,state,guard,self.c.encode(rows),first)
    def test_http_source_raw_and_genuine_partial_failures_keep_literal_frame(self):
        op,state,unchanged=self.opened();actual=o.bounded_transport;active={};body=self.body()
        def observed(c,transport):active['c']=c;return actual(c,transport)
        for change in ('http','partial','source','raw'):
            def guard(boundary):
                unchanged(boundary)
                if change=='source'and boundary=='funding_current_after_callback':raise ValueError('injected_funding_current_source')
            def transport(raw):
                if change=='partial':raise active['c'].TransportFailure('synthetic_current_partial',b'original-current-partial')
                if change=='raw':
                    path=op.path/'00000000.json';path.write_bytes(path.read_bytes()+b'\n')
                return (503 if change=='http'else 200),[],body
            with mock.patch.object(o,'bounded_transport',side_effect=observed):
                with self.assertRaisesRegex(Exception,'funding_http|synthetic_current_partial|injected_funding_current_source|submission_raw_changed')as caught:o._candidate_capture_funding_current(self.a,self.j,op,state,guard,_transport=transport)
            frame=caught.exception.fundingCurrentCapture['frame'];self.assertFalse(caught.exception.fundingCurrentCapture['currentSnapshotVerified'])
            if change=='partial':self.assertEqual(base64.b64decode(frame['partialResponseBase64']),b'original-current-partial');self.assertNotIn('httpBase64',frame)
            else:self.assertEqual(base64.b64decode(frame['responseBase64']),body)
            self.assertEqual({p.name for p in op.path.iterdir()},{'lock','00000000.json','00000001.json'})
    def test_real_zero_invalid_and_unprepared_or_public_send_refuse_before_network(self):
        before=self.hostcase.hashes();packet=self.prepared['packet'];request={'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'FUNDING_INTENT','expectedTail':self.hostcase.tail,'data':None}
        with mock.patch.object(o,'public_rpc_transport',side_effect=AssertionError('unexpected_rpc'))as rpc:
            for signature,reason in ((bytes(64),'zero_signature'),(bytes([1])*64,'signature_invalid')):
                wire=base64.b64decode(packet['wireBase64']);data={'requestId':self.prepared['request']['requestId'],'wireBase64':base64.b64encode(wire[:1]+signature+wire[65:]).decode()}
                with self.assertRaisesRegex(ValueError,'signed_packet_or_receipt_rejected:'+reason):o._candidate_controller_operate({**request,'data':data},self.hostcase.path,_submit_original=True)
            with self.assertRaisesRegex(ValueError,'controller_offline_only'):o.operate(request,self.hostcase.path,live=True)
            rpc.assert_not_called()
        self.assertEqual(before,self.hostcase.hashes())
        with tempfile.TemporaryDirectory()as tmp:
            path=Path(tmp)/'unprepared.candidate-controller';out=o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'CREATE','expectedTail':None,'data':self.host.manifest},path)
            raw={k:self.hostcase.receipt[k]for k in ('wireBase64','feeCapLamports','originalBlockhash','preflight')}
            with mock.patch.object(o,'public_rpc_transport',side_effect=AssertionError('unexpected_rpc'))as rpc:
                with self.assertRaisesRegex(ValueError,'send_fresh_prepared_only'):o._candidate_controller_operate({**request,'expectedTail':out['tail'],'data':raw},path,_submit_original=True)
                rpc.assert_not_called()
    def test_same_new_intent_continuation_once_and_same_call_recovery_floor(self):
        op,state,guard=self.opened();first=self.current(op,state,guard,self.body());prepared=state['fundingPreparation']
        derived=o._candidate_controller_intent(self.a,self.j,state['manifest'],prepared['unsignedIntent']);derived['preparationRequestId']=prepared['request']['requestId']
        continuation=o._candidate_funding_send_continuation(self.a,self.j,op,state,derived,first,guard)
        with mock.patch.object(o,'public_rpc_transport')as rpc:
            with self.assertRaisesRegex(ValueError,'send_new_commit_required'):continuation(op.chain[-1])
            with self.assertRaisesRegex(ValueError,'send_continuation_consumed'):continuation(op.chain[-1])
            state['intent']=derived
            with self.assertRaisesRegex(ValueError,'send_fresh_prepared_only'):o._candidate_funding_send_continuation(self.a,self.j,op,state,derived,first,guard)
            rpc.assert_not_called()
        current=first;future=self.current(op,{**state,'intent':None},guard,self.body(0,20,102),first)
        evidence={'inclusionSlot':future['contextSlot'],'postClock':future['clock']};o._candidate_funding_recovery_floor(current,evidence)
        bad=copy.deepcopy(evidence);bad['inclusionSlot']=str(int(current['contextSlot'])-1)
        with self.assertRaisesRegex(ValueError,'recovery_current_floor'):o._candidate_funding_recovery_floor(current,bad)
        bad=copy.deepcopy(evidence);raw=bytearray(base64.b64decode(bad['postClock']['dataBase64']));stamp=int(current['clock']['unixTimestamp'])-1
        raw[32:40]=stamp.to_bytes(8,'little',signed=True);bad['postClock'].update(unixTimestamp=str(stamp),dataBase64=base64.b64encode(raw).decode(),dataSha256=o.sha(raw))
        with self.assertRaisesRegex(ValueError,'recovery_current_floor'):o._candidate_funding_recovery_floor(current,bad)


class CandidatePhaseInputCaptureTests(unittest.TestCase):
    """Unsigned constructor/guard components only; no valid funded preparation or TLS."""
    @classmethod
    def setUpClass(cls):
        cls.host=CandidateCurrentPhaseTests;cls.host.setUpClass();cls.a=cls.host.a;cls.j=cls.host.j;cls.c=cls.host.c;cls.ref=cls.host.ref;cls.manifest=cls.host.manifest
    def fixture(self,index,seconds=1):
        # Placeholder funding/runtime facts select raw construction, never admission.
        floor=int(self.host.pre['contextSlot']);accounts=copy.deepcopy(self.host.host.original);slot=floor+5
        clock=bytearray(base64.b64decode(accounts[self.ref['roles']['clock']]['dataBase64']));clock[:8]=slot.to_bytes(8,'little')
        clock[32:40]=(int(self.host.pre['unixTimestamp'])+seconds).to_bytes(8,'little',signed=True);accounts[self.ref['roles']['clock']]['dataBase64']=base64.b64encode(clock).decode()
        state={'manifest':self.manifest,'cursor':index,'funding':{},'pending':None,'phasePreparation':None,'phaseSignAttempt':None,
            'lastClock':copy.deepcopy(self.host.pre),'initializerBinding':copy.deepcopy(self.host.binding)if index>937 else None,'runtimeFacts':{'componentOnlyNotAuthenticated':True}}
        calls=[];blockhash=self.ref['preManifest']['initializer']['transport']['recentBlockhash']
        def transport(raw):
            batch=self.c.strict_json(raw);calls.append(batch);results=[]
            for query in batch:
                method=query['method']
                if method=='getGenesisHash':value=self.c.GENESIS
                elif method=='getLatestBlockhash':value={'context':{'slot':slot+1},'value':{'blockhash':blockhash,'lastValidBlockHeight':300}}
                elif method=='getMultipleAccounts':
                    rows=[]
                    for key in query['params'][0]:
                        acc=accounts[key];rows.append(None if not acc['exists']else {'owner':acc['owner'],'executable':acc['executable'],'lamports':int(acc['lamports']),'data':[acc['dataBase64'],'base64']})
                    value={'context':{'slot':slot+2 if index<935 else slot},'value':rows}
                elif method=='getFeeForMessage':value={'context':{'slot':slot+2},'value':int(self.manifest['phases'][index]['historicalFeeProjectionLamports'])}
                elif method=='getBlockHeight':value=100
                else:self.assertEqual(method,'isBlockhashValid');value={'context':{'slot':slot+2},'value':True}
                results.append({'jsonrpc':'2.0','id':query['id'],'result':value})
            return 200,[],self.c.encode(results)
        return state,calls,transport
    def capture(self,index,seconds=1):
        state,calls,transport=self.fixture(index,seconds);before=o.encode(state)
        out=o._candidate_collect_phase_input_frames(self.a,self.j,state,{'feeCapLamports':'100000'},lambda boundary:None,_transport=transport)
        self.assertEqual(o.encode(state),before);self.assertIn('NOT_TLS',out['transportMode']);self.assertFalse(out['unsignedPreparationVerified']);self.assertFalse(out['freshAfterDurability'])
        return state,calls,out
    def test_prefix_exact_source_policy60_and10_original_message_zero_wires(self):
        for index,count in ((0,60),(1,10)):
            state,calls,out=self.capture(index);keys=self.a.candidate_expected(self.manifest,index,'PRE',funding_mode='CONDITIONAL_H2_CREDIT')['requiredSnapshotKeys']
            self.assertEqual(len(keys),count);self.assertEqual(calls[1][1]['params'][0],keys)
            self.assertEqual([[q['method']for q in batch]for batch in calls],[['getGenesisHash','getLatestBlockhash'],['getFeeForMessage','getMultipleAccounts','getBlockHeight','isBlockhashValid']])
            wire=base64.b64decode(out['input']['wireBase64']);self.assertLessEqual(len(wire),1232);self.assertFalse(any(wire[1:1+64*wire[0]]))
            review,=o.describe(self.j,[{'wireBase64':out['input']['wireBase64'],'lookupTables':[]}]);self.assertEqual(calls[1][0]['params'][0],review['messageBase64'])
            original=self.c.strict_json(base64.b64decode(out['frames']['originalBlockhash']['responseBase64']))[1]['result'];self.assertEqual(calls[1][1]['params'][1]['minContextSlot'],original['context']['slot'])
    def test_937_raw_preclock_changes_quote_binding_and_precedes_hash_without_adopting_facts(self):
        _,calls,first=self.capture(937,1);_,_,later=self.capture(937,2)
        self.assertEqual([[q['method']for q in batch]for batch in calls],[['getMultipleAccounts'],['getGenesisHash','getLatestBlockhash'],['getFeeForMessage','getBlockHeight','isBlockhashValid']])
        pre=self.c.strict_json(base64.b64decode(first['frames']['prestate']['responseBase64']))[0]['result'];self.assertEqual(calls[1][1]['params'][0]['minContextSlot'],pre['context']['slot'])
        self.assertNotEqual(first['input']['wireBase64'],later['input']['wireBase64']);wire=base64.b64decode(first['input']['wireBase64']);self.assertEqual(calls[2][0]['params'][0],base64.b64encode(wire[1+64*wire[0]:]).decode())
        state,calls,transport=self.fixture(937,-1)
        with self.assertRaisesRegex(ValueError,'candidate_runtime_prestate_order'):o._candidate_collect_phase_input_frames(self.a,self.j,state,{'feeCapLamports':'100000'},lambda boundary:None,_transport=transport)
        self.assertEqual(len(calls),1)
    def test_945_real_pinned_v0_template_preserves_frozen_alt_binding(self):
        state,calls,out=self.capture(945);plan=state['initializerBinding']['unsignedInitializerPlan'];packet=plan['packets'][8]
        self.assertEqual(out['input']['wireBase64'],packet['wireBase64']);wire=base64.b64decode(out['input']['wireBase64']);self.assertEqual(wire[1+64*wire[0]],128)
        review,=o.describe(self.j,[{'wireBase64':out['input']['wireBase64'],'lookupTables':[{'address':plan['alt']['address'],'addresses':plan['alt']['addresses']}]}])
        self.assertEqual(calls[2][0]['params'][0],review['messageBase64']);self.assertEqual(review['lookupTables'],[{'address':plan['alt']['address'],'addresses':plan['alt']['addresses']}])
    def test_actual_unfunded_shorthand_and_cap_hold_cursor_refusals_are_before_rpc(self):
        with tempfile.TemporaryDirectory()as tmp:
            path=Path(tmp)/'phase.candidate-controller';out=o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'CREATE','expectedTail':None,'data':self.manifest},path)
            before={p.name:p.read_bytes()for p in path.iterdir()}
            with mock.patch.object(o,'public_rpc_transport',side_effect=AssertionError('unexpected_rpc'))as rpc:
                with self.assertRaisesRegex(ValueError,'candidate_prefix_funding_required'):o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'PREPARE_PHASE','expectedTail':out['tail'],'data':{'feeCapLamports':'100000'}},path)
                state,_,_=self.fixture(0)
                for cap in (True,0,'0','01','١','18446744073709551616'):
                    with self.assertRaisesRegex(ValueError,'u64_string|positive_cap'):o._candidate_collect_phase_input_frames(self.a,self.j,state,{'feeCapLamports':cap},lambda boundary:None)
                for key in ('pending','phasePreparation','phaseSignAttempt'):
                    changed=copy.deepcopy(state);changed[key]={}
                    with self.assertRaisesRegex(ValueError,'funding_or_hold'):o._candidate_collect_phase_input_frames(self.a,self.j,changed,{'feeCapLamports':'1'},lambda boundary:None)
                for cursor in (True,947):
                    changed={**state,'cursor':cursor}
                    with self.assertRaisesRegex(ValueError,'candidate_startup_index'):o._candidate_collect_phase_input_frames(self.a,self.j,changed,{'feeCapLamports':'1'},lambda boundary:None)
                rpc.assert_not_called()
            self.assertEqual(before,{p.name:p.read_bytes()for p in path.iterdir()})
    def test_raw_http_partial_source_and_orphan_faults_retain_exact_bytes_without_retry(self):
        actual=o.bounded_transport;active={}
        def observed(c,transport):active['c']=c;return actual(c,transport)
        for fault in ('http','partial','source','orphan'):
            with tempfile.TemporaryDirectory()as tmp:
                path=Path(tmp)/'guard.candidate-controller';o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'CREATE','expectedTail':None,'data':self.manifest},path)
                op=o.Journal(self.j,path,_candidate=True)
                try:
                    state,calls,transport=self.fixture(0);state['controllerPath']=str(path);_,_,_,domain,unchanged=o._candidate_preparation_sources();bodies=[]
                    def guard(boundary):
                        unchanged(boundary)
                        if fault=='source'and boundary=='phase_input_originalBlockhash_after_callback':raise ValueError('injected_phase_input_source_fault')
                    def callback(raw):
                        status,headers,body=transport(raw);bodies.append(body)
                        if fault=='partial':raise active['c'].TransportFailure('synthetic_phase_input_partial',b'exact-partial')
                        if fault=='orphan':
                            root=path.with_name(path.name+'.evidence');root.mkdir(mode=0o700);(root/'orphan').mkdir(mode=0o700)
                        return (503 if fault=='http'else status),headers,body
                    with mock.patch.object(o,'bounded_transport',side_effect=observed):
                        with self.assertRaisesRegex(Exception,'candidate_prefix_http|synthetic_phase_input_partial|injected_phase_input_source_fault|orphan_uncertain_stop')as caught:o._candidate_capture_phase_input(self.a,self.j,op,state,{'feeCapLamports':'100000'},domain,guard,_transport=callback)
                    frame=caught.exception.phaseInputCapture['frames']['originalBlockhash'];self.assertEqual(len(calls),1);self.assertFalse(caught.exception.phaseInputCapture['unsignedPreparationVerified'])
                    if fault=='partial':self.assertEqual(base64.b64decode(frame['partialResponseBase64']),b'exact-partial');self.assertNotIn('httpBase64',frame)
                    else:self.assertEqual(base64.b64decode(frame['responseBase64']),bodies[0])
                    self.assertEqual(len(op.chain),1)
                finally:op.close()


class CandidateSignAttemptInputTests(unittest.TestCase):
    """Public v2 declaration/current components only; no funded reservation or grant."""
    @classmethod
    def setUpClass(cls):
        cls.host=CandidateSignerBoundaryTests;cls.host.setUpClass();cls.a=cls.host.a;cls.j=cls.host.j;cls.c=cls.host.c
    def fixture(self,expiry='18446744073709551615'):
        host=self.host(methodName='test_existing_public_contract_projection_binds_prepared_source_nullparent_and_lifetime');state,data=host.fixture(expiry)
        tmp=tempfile.TemporaryDirectory();self.addCleanup(tmp.cleanup);path=Path(tmp.name)/'sign-input.candidate-controller'
        o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'CREATE','expectedTail':None,'data':state['manifest']},path)
        op=o.Journal(self.j,path,_candidate=True);self.addCleanup(op.close);state.update(controllerPath=str(path),funding={})
        # Synthetic prepared state over observed PRE0; the real journal has CREATE only.
        _,_,_,_,guard=o._candidate_preparation_sources();return state,data,op,guard
    def shorthand(self,data):return {k:data[k]for k in ('authorizationRawBase64','authorizationSha256')}
    def capture(self,state,data,op,guard,transport):return o._candidate_capture_sign_attempt_input(self.a,self.j,op,state,self.shorthand(data),guard,_transport=transport)
    def test_real_declaration_first_single_capture_composes_unchanged_attempt_without_commit(self):
        state,data,op,guard=self.fixture();before=o.encode(state);files={p.name:p.read_bytes()for p in op.path.iterdir()};events=[];authorize=o._candidate_sign_authorization;calls=[]
        def auth(*args,**kw):events.append('authorization');return authorize(*args,**kw)
        def transport(raw):events.append('capture');calls.append(raw);return 200,[],base64.b64decode(data['current']['responseBase64'])
        with mock.patch.object(o,'_candidate_sign_authorization',side_effect=auth):out=self.capture(state,data,op,guard,transport)
        self.assertEqual(len(calls),1);self.assertEqual(calls[0],base64.b64decode(data['current']['requestBase64']));self.assertEqual(events,['authorization','authorization','capture','authorization'])
        self.assertEqual(set(out['input']),{'authorizationRawBase64','authorizationSha256','current'});self.assertEqual(out['input']['current'],out['currentCapture']['publicFrame'])
        self.assertFalse(out['currentCapture']['publicCapture']['tlsReceiptValidated']);self.assertIn('NOT_TLS',out['currentCapture']['publicCapture']['transportMode']);self.assertTrue(out['currentInputVerified']);self.assertFalse(out['freshAfterDurability'])
        derived=o._candidate_sign_phase_attempt(self.a,self.j,state,out['input'],op.tail());self.assertEqual(derived['identity']['request']['authorizationSha256'],data['authorizationSha256'])
        self.assertEqual(derived['identity']['preparationRequestId'],state['phasePreparation']['request']['requestId']);self.assertFalse(derived['identity']['nativeAdmissionQualified'])
        self.assertEqual(o.encode(state),before);self.assertEqual(len(op.chain),1);self.assertEqual(files,{p.name:p.read_bytes()for p in op.path.iterdir()})
    def test_expired_invalid_hold_and_actual_unfunded_shorthand_stop_before_rpc(self):
        state,data,op,guard=self.fixture();files={p.name:p.read_bytes()for p in op.path.iterdir()}
        with mock.patch.object(o,'public_rpc_transport',side_effect=AssertionError('unexpected_rpc'))as rpc:
            for change,reason in (('expired','authorization_expired'),('sha','authorization_required'),('source','authorization_binding'),('profile','digest')):
                changed=copy.deepcopy(data);receipt=json.loads(base64.b64decode(data['authorizationRawBase64']))
                if change=='expired':receipt['expiresUnixTimestamp']='0'
                if change=='source':receipt['operatorSourceSha256']='0'*64
                if change=='profile':receipt['signerProfileSha256']='invalid'
                raw=o.encode(receipt);changed.update(authorizationRawBase64=base64.b64encode(raw).decode(),authorizationSha256='0'*64 if change=='sha'else o.sha(raw))
                with self.assertRaisesRegex(ValueError,reason):o._candidate_capture_sign_attempt_input(self.a,self.j,op,state,self.shorthand(changed),guard)
            for key in ('pending','phaseSignAttempt','phasePreparation','funding'):
                changed=copy.deepcopy(state);changed[key]=None if key in ('phasePreparation','funding')else {}
                with self.assertRaisesRegex(ValueError,'prepared_funding_or_hold'):o._candidate_capture_sign_attempt_input(self.a,self.j,op,changed,self.shorthand(data),guard)
            path=op.path;tail=op.tail();op.close();request={'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'SIGN_PHASE_ATTEMPT','expectedTail':tail,'data':self.shorthand(data)}
            with self.assertRaisesRegex(ValueError,'prefix_funding_required'):o.operate(request,path)
            with self.assertRaisesRegex(ValueError,'controller_offline_only'):o.operate(request,path,live_sign=True)
            rpc.assert_not_called()
        self.assertEqual(files,{p.name:p.read_bytes()for p in path.iterdir()})
    def test_expiry_source_or_inline_raw_after_capture_preserves_false_evidence(self):
        for fault,reason in (('expiry','authorization_expired'),('source','injected_sign_input_source'),('raw','sign_input_raw_changed')):
            state,data,op,unchanged=self.fixture('105');clock=[100];body=base64.b64decode(data['current']['responseBase64']);calls=[]
            def guard(boundary):
                unchanged(boundary)
                if fault=='source'and boundary=='sign_input_after_capture':raise ValueError('injected_sign_input_source')
            def transport(raw):
                calls.append(raw)
                if fault=='expiry':clock[0]=110
                if fault=='raw':(op.path/'00000000.json').write_bytes((op.path/'00000000.json').read_bytes()+b' ')
                return 200,[],body
            with mock.patch.object(o.time,'time',side_effect=lambda:clock[0]):
                with self.assertRaisesRegex(ValueError,reason)as caught:self.capture(state,data,op,guard,transport)
            evidence=caught.exception.signAttemptInputCapture;self.assertEqual(len(calls),1);self.assertFalse(evidence['currentInputVerified']);self.assertFalse(evidence['currentCapture']['publicCapture']['tlsReceiptValidated'])
            self.assertEqual(base64.b64decode(evidence['currentCapture']['publicFrame']['responseBase64']),body);self.assertEqual(len(op.chain),1)
    def test_retained_sidecar_or_orphan_after_callback_refuses_without_adoption(self):
        for fault,reason in (('payload','prefix_payload_hash'),('orphan','orphan_uncertain_stop')):
            state,data,op,guard=self.fixture();raw=o.encode({'syntheticPreparationReferenceOnly':True});ref=o._candidate_prefix_store(self.c,self.j,op,{'cursor':0},'PREPARE_PHASE',raw)
            # Host raw/reference chain only, never accepted funded replay or an old parent.
            body={'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'sequence':'1','previousDigest':op.tail(),'lockIdentity':op.identity,'kind':'PREPARE_PHASE','data':{'evidence':ref,'derived':{'index':0}}}
            record={**body,'digest':o.sha(o.encode(body))};self.j.write_file(op.fd,'00000001.json',o.encode(record));op.chain.append(record)
            root=op.path.with_name(op.path.name+'.evidence');payload=root/ref['name']/'payload.json';response=base64.b64decode(data['current']['responseBase64'])
            def transport(request):
                if fault=='payload':payload.write_bytes(b'{}\n')
                else:(root/'orphan').mkdir(mode=0o700)
                return 200,[],response
            with self.assertRaisesRegex(ValueError,reason)as caught:self.capture(state,data,op,guard,transport)
            self.assertFalse(caught.exception.signAttemptInputCapture['currentInputVerified']);self.assertEqual(base64.b64decode(caught.exception.signAttemptInputCapture['currentCapture']['publicFrame']['responseBase64']),response)
            self.assertEqual(len(op.chain),2);self.assertTrue(payload.exists())
    def test_http_stale_and_genuine_partial_preserve_literal_body_without_retry(self):
        actual=o.bounded_transport;active={}
        def observed(c,transport):active['c']=c;return actual(c,transport)
        for fault,reason in (('http','prefix_http'),('stale','current_original_lifetime'),('partial','synthetic_sign_input_partial')):
            state,data,op,guard=self.fixture();replies=self.c.strict_json(base64.b64decode(data['current']['responseBase64']));calls=[]
            if fault=='stale':replies[3]['result']['value']=False
            body=self.c.encode(replies)
            def transport(raw):
                calls.append(raw)
                if fault=='partial':raise active['c'].TransportFailure('synthetic_sign_input_partial',b'sign-input-partial')
                return (503 if fault=='http'else 200),[],body
            with mock.patch.object(o,'bounded_transport',side_effect=observed):
                with self.assertRaisesRegex(Exception,reason)as caught:self.capture(state,data,op,guard,transport)
            evidence=caught.exception.signAttemptInputCapture;self.assertEqual(len(calls),1);self.assertFalse(evidence['currentInputVerified']);captured=evidence['currentCapture'];self.assertFalse(captured['tlsReceiptValidated'])
            if fault=='partial':self.assertEqual(base64.b64decode(captured['partialResponseBase64']),b'sign-input-partial');self.assertIsNone(captured['publicFrame'])
            else:self.assertEqual(base64.b64decode(captured['publicFrame']['responseBase64']),body)
            self.assertEqual(len(op.chain),1)


class CandidateUnfundedReplayReuseTests(unittest.TestCase):
    """Real public host reconstruction; no accepted signed funding or RPC."""
    @classmethod
    def setUpClass(cls):
        from tools.test_startup_manifest import CandidateFundingReceiptTests
        cls.host=CandidateFundingReceiptTests;cls.host.setUpClass()
        cls.a=o.startup_manifest();cls.manifest=cls.host.manifest;cls.j=cls.a._candidate_helpers()[0]
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
        self.path=Path(self.tmp.name)/'reuse.candidate-controller'
        self.receipt,_,_=self.host.fixture()
        self.data={k:self.receipt[k]for k in ('feeCapLamports','originalBlockhash','preflight')}
        self.created=o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'CREATE','expectedTail':None,'data':self.manifest},self.path)
    def chain(self):
        op=o.Journal(self.j,self.path,_candidate=True)
        try:return copy.deepcopy(op.chain)
        finally:op.close()
    def prepare(self):
        return o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'PREPARE_FUNDING','expectedTail':self.created['tail'],'data':self.data},self.path)
    def observe_sources(self,modules,fault=None):
        real=o._candidate_preparation_sources
        def sources():
            a,j,s,domain,unchanged=real();original=a._candidate_helpers
            def verified():
                value=original()
                if fault=='helper':raise ValueError('injected_after_real_helper_verification')
                return value
            spy=mock.Mock(side_effect=verified);a._candidate_helpers=spy;modules.append((a,spy))
            def checked(boundary):
                unchanged(boundary)
                if fault=='final'and boundary=='controller_replay':raise ValueError('injected_final_source_refusal')
            return a,j,s,domain,checked
        return sources
    def test_prepared_reconstruction_matches_fresh_parsers_and_rederives_changed_input(self):
        self.prepare();records=self.chain();before=o.encode(records)
        a,j,s,domain,unchanged=o._candidate_preparation_sources();fresh=a._candidate_helpers
        with mock.patch.object(a,'_candidate_helpers',wraps=fresh)as calls:
            expected_manifest=a.candidate_validate(records[0]['data']['manifest'])
            expected_preparation=o._candidate_prepare_funding(a,j,expected_manifest,records[1]['data']['input'],records[1]['previousDigest'],domain)
        self.assertEqual(calls.call_count,3)  # Actual independent validation+two pure parsers.
        state=o._candidate_controller_replay(self.j,records,self.path)
        self.assertEqual(o.encode(state['manifest']),o.encode(expected_manifest))
        self.assertEqual(o.encode(state['fundingPreparation']),o.encode(expected_preparation));self.assertEqual(o.encode(records),before)
        changed=copy.deepcopy(records);data=changed[1]['data']['input'];reply=json.loads(base64.b64decode(data['preflight']['responseBase64']))
        reply[1]['result']['value'][1]['lamports']+=1;data['preflight']['responseBase64']=base64.b64encode(self.a.canonical(reply)).decode()
        with self.assertRaisesRegex(ValueError,'preparation_sufficiency'):
            o._candidate_controller_replay(self.j,changed,self.path)
        self.assertEqual(o.encode(self.chain()),before)
    def test_modules_are_verified_once_inside_each_replay_and_never_shared_across_calls(self):
        self.prepare();records=self.chain();modules=[]
        with mock.patch.object(o,'_candidate_preparation_sources',side_effect=self.observe_sources(modules)):
            first=o._candidate_controller_replay(self.j,records,self.path)
            second=o._candidate_controller_replay(self.j,records,self.path)
            create=o._candidate_controller_replay(self.j,records[:1],self.path)
        self.assertEqual(o.encode(first),o.encode(second));self.assertIsNone(create['fundingPreparation'])
        self.assertEqual([spy.call_count for a,spy in modules],[1,1,1]);self.assertEqual(len({id(a)for a,spy in modules}),3)
        for a,spy in modules:self.assertIs(a._candidate_helpers,spy)
    def test_real_source_checks_and_finally_restore_refuse_faults_without_raw_mutation(self):
        self.prepare();records=self.chain();before={p.name:p.read_bytes()for p in self.path.iterdir()}
        for fault,reason in (('helper','injected_after_real_helper_verification'),('final','injected_final_source_refusal')):
            modules=[]
            with mock.patch.object(o,'_candidate_preparation_sources',side_effect=self.observe_sources(modules,fault)):
                with self.assertRaisesRegex(ValueError,reason):o._candidate_controller_replay(self.j,records,self.path)
            self.assertEqual(len(modules),1);a,spy=modules[0];self.assertIs(a._candidate_helpers,spy);self.assertEqual(spy.call_count,1)
        self.assertEqual(before,{p.name:p.read_bytes()for p in self.path.iterdir()})
    def test_unverified_intent_and_funding_result_keep_default_helpers_and_real_crypto_refusal(self):
        raw={k:self.receipt[k]for k in ('wireBase64','feeCapLamports','originalBlockhash','preflight')}
        held=o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'FUNDING_INTENT','expectedTail':self.created['tail'],'data':raw},self.path)
        self.assertEqual(held['status'],'UNKNOWN_HOLD_ORIGINAL_FUNDING_ONLY');records=self.chain();modules=[]
        with mock.patch.object(o,'_candidate_preparation_sources',side_effect=self.observe_sources(modules)):
            state=o._candidate_controller_replay(self.j,records,self.path)
        self.assertIsNone(state['funding']);self.assertEqual(modules[0][1].call_count,2)
        candidate=copy.deepcopy(records);candidate.append({'kind':'FUNDING_RESULT','data':{'receipt':self.receipt,'expectedReceiptSha256':o.sha(o.encode(self.receipt))}})
        modules=[]
        with mock.patch.object(o,'_candidate_preparation_sources',side_effect=self.observe_sources(modules)):
            with self.assertRaisesRegex(ValueError,'signature_or_receipt:zero_signature'):
                o._candidate_controller_replay(self.j,candidate,self.path)
        self.assertGreater(modules[0][1].call_count,1);self.assertIs(modules[0][0]._candidate_helpers,modules[0][1])
        self.assertEqual(o.encode(records),o.encode(self.chain()))


class CandidateSavedFundingRouteTests(unittest.TestCase):
    """Real host records and injected NOT_TLS declarations; no funded/signature positive."""
    @classmethod
    def setUpClass(cls):
        from tools.test_startup_manifest import CandidateSavedFundingReceiptTests,CandidateFundingReceiptTests
        CandidateFundingReceiptTests.setUpClass();cls.host=CandidateFundingReceiptTests
        cls.a=cls.host.a;cls.manifest=cls.host.manifest;cls.j,_,cls.c=cls.a._candidate_helpers()
        cls.route_fixture=staticmethod(CandidateSavedFundingReceiptTests.route)

    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
        self.path=Path(self.tmp.name)/'saved.candidate-controller';self.tail=None;self.route=self.route_fixture();self.calls=[]
        receipt,_,_=self.host.fixture();self.bodies=[base64.b64decode(receipt[k]['responseBase64'])for k in ('originalBlockhash','preflight')]
        self.data={k:receipt[k]for k in ('feeCapLamports','originalBlockhash','preflight')}

    def transport(self,c,raw):
        self.calls.append(c.strict_json(raw));return 200,[],self.bodies[len(self.calls)-1]

    def call(self,action,data=None,**kwargs):
        out=o._candidate_controller_operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':action,'expectedTail':self.tail,'data':data},
            self.path,_funding_route=self.route,_funding_transport=self.transport,**kwargs)
        self.tail=out['tail'];return out

    def test_route_bound_before_create_and_request_replay_without_default_transport(self):
        created=self.call('CREATE',self.manifest)
        with mock.patch.object(o,'public_rpc_transport',side_effect=AssertionError('unexpected_default_rpc'))as rpc:
            prepared=self.call('PREPARE_FUNDING',{'feeCapLamports':'10000'})['fundingPreparation'];rpc.assert_not_called()
        self.assertEqual(len(self.calls),2);self.assertEqual(created['fundingRoute'],self.route)
        journal=o.Journal(self.j,self.path,_candidate=True)
        try:
            state=o._candidate_controller_replay(self.j,journal.chain,self.path)
            self.assertEqual(journal.chain[0]['data']['sourceDomain']['fundingRoute'],self.route)
            self.assertEqual(state['fundingRoute'],self.route);self.assertEqual(state['fundingPreparation'],prepared)
            domain=copy.deepcopy(journal.chain[0]['data']['sourceDomain']);domain.pop('fundingRoute')
            other=o._candidate_prepare_funding(self.a,self.j,self.manifest,self.data,journal.chain[1]['previousDigest'],domain)
            self.assertNotEqual(other['request']['requestId'],prepared['request']['requestId'])
        finally:journal.close()
        self.assertEqual(self.call('INSPECT')['fundingPreparation'],prepared)

    def test_changed_missing_route_or_public_override_refuses_before_network_and_append(self):
        self.call('CREATE',self.manifest);before={p.name:p.read_bytes()for p in self.path.iterdir()}
        bad=copy.deepcopy(self.route);bad['originCommitmentSha256']='3'*64
        request={'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'PREPARE_FUNDING','expectedTail':self.tail,'data':{'feeCapLamports':'10000'}}
        for kwargs in ({'_funding_route':bad,'_funding_transport':self.transport},{},):
            with self.assertRaisesRegex(ValueError,'candidate_funding_route_'):
                o._candidate_controller_operate(request,self.path,**kwargs)
        with self.assertRaisesRegex(ValueError,'offline_only'):o.operate(request,self.path,_transport=lambda raw:None)
        self.assertEqual(self.calls,[]);self.assertEqual(before,{p.name:p.read_bytes()for p in self.path.iterdir()})

    def test_zero_invalid_original_never_reach_route_callback_or_replace_preparation(self):
        self.call('CREATE',self.manifest);prepared=self.call('PREPARE_FUNDING',self.data)['fundingPreparation'];self.calls=[]
        before={p.name:p.read_bytes()for p in self.path.iterdir()};wire=base64.b64decode(prepared['packet']['wireBase64'])
        for signature,reason in ((bytes(64),'zero_signature'),(bytes([1])*64,'signature_invalid')):
            signed=wire[:1]+signature+wire[65:]
            with self.assertRaisesRegex(ValueError,reason):self.call('FUNDING_INTENT',{'requestId':prepared['request']['requestId'],'wireBase64':base64.b64encode(signed).decode()},_submit_original=True)
        self.assertEqual(self.calls,[]);self.assertEqual(before,{p.name:p.read_bytes()for p in self.path.iterdir()})

    def test_original_only_recovery_component_emits_v2_without_official_relabel_and_keeps_partials(self):
        state,receipt,bodies,guard=CandidateFundingResultCaptureTests.fixture(self);state['fundingRoute']=copy.deepcopy(self.route);seen=[]
        def transport(c,raw):seen.append(c.strict_json(raw));return 200,[],bodies[len(seen)-1]
        out=o._candidate_collect_funding_frames(self.a,self.j,state,guard,_transport=transport,_route=self.route)
        supplied=out['result']['receipt'];self.assertEqual(supplied['schema'],'piv1-candidate-supplied-funding-receipt-v2')
        self.assertNotIn('endpoint',supplied);self.assertEqual(supplied['route'],self.route);self.assertFalse(out['suppliedReceiptVerifierAccepted'])
        self.assertEqual([len(q)for q in seen],[1,2,1]);self.assertIn('NOT_TLS',out['transportMode'])
        with self.assertRaisesRegex(ValueError,'signature_or_receipt:zero_signature'):
            self.a.candidate_verify_funding(self.manifest,supplied,out['result']['expectedReceiptSha256'],expected_route=self.route)
        def partial(c,raw):raise c.TransportFailure('synthetic_public_partial',b'[{')
        with self.assertRaises(Exception)as caught:o._candidate_collect_funding_frames(self.a,self.j,state,guard,_transport=partial,_route=self.route)
        self.assertEqual(base64.b64decode(caught.exception.fundingResultCapture['frames']['identity']['partialResponseBase64']),b'[{')

    def test_direct_callback_route_mutation_refuses_after_retaining_literal_response(self):
        self.call('CREATE',self.manifest);journal=o.Journal(self.j,self.path,_candidate=True)
        try:state=o._candidate_controller_replay(self.j,journal.chain,self.path)
        finally:journal.close()
        mutable=copy.deepcopy(self.route);calls=[]
        def transport(c,raw):
            calls.append(raw);mutable['originCommitmentSha256']='3'*64
            return 200,[],self.bodies[0]
        with self.assertRaisesRegex(ValueError,'candidate_funding_route_changed')as caught:
            o._candidate_collect_funding_input_frames(self.a,self.j,state,{'feeCapLamports':'10000'},lambda boundary:None,_transport=transport,_route=mutable)
        self.assertEqual(len(calls),1)
        self.assertEqual(base64.b64decode(caught.exception.fundingInputCapture['frames']['originalBlockhash']['responseBase64']),self.bodies[0])
        self.assertFalse(caught.exception.fundingInputCapture['unsignedPreparationVerified'])

    def test_saved_route_replay_refuses_phase_kinds_before_evidence_and_never_persists(self):
        self.call('CREATE',self.manifest);before={p.name:p.read_bytes()for p in self.path.iterdir()}
        journal=o.Journal(self.j,self.path,_candidate=True)
        try:
            for kind in ('PREPARE_PHASE','SIGN_PHASE_ATTEMPT','PHASE_INTENT','PHASE_RESULT'):
                with self.subTest(kind=kind),self.assertRaisesRegex(ValueError,'candidate_saved_route_funding_only'):
                    journal.append(kind,{})
                self.assertEqual(len(journal.chain),1);self.assertEqual(journal.tail(),self.tail)
        finally:journal.close()
        self.assertEqual(before,{p.name:p.read_bytes()for p in self.path.iterdir()})
        self.assertFalse(self.path.with_name(self.path.name+'.evidence').exists())
        legacy=self.path.with_name('default.candidate-controller')
        created=o.operate({'schema':o.CANDIDATE_CONTROLLER_SCHEMA,'action':'CREATE','expectedTail':None,'data':self.manifest},legacy)
        old={p.name:p.read_bytes()for p in legacy.iterdir()};journal=o.Journal(self.j,legacy,_candidate=True)
        try:
            with self.assertRaisesRegex(ValueError,'candidate_prefix_funding_required'):journal.append('PHASE_INTENT',{})
            self.assertEqual(journal.tail(),created['tail']);self.assertEqual(len(journal.chain),1)
        finally:journal.close()
        self.assertEqual(old,{p.name:p.read_bytes()for p in legacy.iterdir()})


class CandidateFactoryRuntimeGateTests(unittest.TestCase):
    """One private full runtime gate per factory; every later guard rereads it."""
    def observe_helpers(self,observed,*,invalid_runtime=None):
        real_exec=exec
        def checked_exec(code,namespace,**kwargs):
            real_exec(code,namespace,**kwargs)
            if namespace.get('__file__')==str(HERE/'startup_manifest.py'):
                load=namespace['_candidate_python_helpers']
                def loaded():
                    repo,modules=load();j,r,c=modules
                    if invalid_runtime is not None:j.NODE=invalid_runtime
                    j.candidate_pins=mock.Mock(side_effect=AssertionError('default_runtime_gate_called'))
                    r.verify_sources=mock.Mock(wraps=r.verify_sources)
                    observed.append((j,r,c));return repo,modules
                namespace['_candidate_python_helpers']=loaded
                namespace['_candidate_helpers']=mock.Mock(wraps=namespace['_candidate_helpers'])
        return checked_exec

    def observe_runtime(self,checks,*,wrong_repo=False):
        factory=o._candidate_runtime_pins
        def create(j):
            real=factory(j)
            def check():
                repo=real();return repo/'foreign'if wrong_repo else repo
            observed=mock.Mock(side_effect=check);checks.append(observed);return observed
        return create

    def test_factory_full_runtime_once_and_each_later_guard_rechecks(self):
        observed=[];checks=[]
        with mock.patch.object(o,'exec',side_effect=self.observe_helpers(observed),create=True),mock.patch.object(o,'_candidate_runtime_pins',side_effect=self.observe_runtime(checks)):
            a,j,s,domain,guard=o._candidate_preparation_sources()
            self.assertEqual(len(observed),1);self.assertIs(j,observed[0][0])
            self.assertEqual(checks[0].call_count,1);self.assertEqual(observed[0][1].verify_sources.call_count,1)
            j.candidate_pins.assert_not_called()
            guard('separate_test_boundary')
            self.assertEqual(checks[0].call_count,2);self.assertEqual(observed[0][1].verify_sources.call_count,2)

    def test_factory_invalid_runtime_refuses_at_final_gate_before_return(self):
        observed=[];checks=[]
        with tempfile.TemporaryDirectory()as temp:
            invalid=Path(temp)/'invalid-public-runtime';invalid.write_bytes(b'PUBLIC_INVALID_RUNTIME')
            with mock.patch.object(o,'exec',side_effect=self.observe_helpers(observed,invalid_runtime=invalid),create=True),mock.patch.object(o,'_candidate_runtime_pins',side_effect=self.observe_runtime(checks)):
                with self.assertRaisesRegex(ValueError,'source_pin'):o._candidate_preparation_sources()
        self.assertEqual(len(observed),1);self.assertEqual(checks[0].call_count,1)
        observed[0][0].candidate_pins.assert_not_called();observed[0][1].verify_sources.assert_not_called()

    def test_factory_wrong_runtime_repo_refuses_before_rules_and_return(self):
        observed=[];checks=[]
        with mock.patch.object(o,'exec',side_effect=self.observe_helpers(observed),create=True),mock.patch.object(o,'_candidate_runtime_pins',side_effect=self.observe_runtime(checks,wrong_repo=True)):
            with self.assertRaisesRegex(ValueError,'candidate_helper_repo'):o._candidate_preparation_sources()
        self.assertEqual(len(observed),1);self.assertEqual(checks[0].call_count,1)
        observed[0][1].verify_sources.assert_not_called()

    def test_factory_captured_source_change_refuses_before_runtime_and_return(self):
        observed=[];checks=[];read=Path.read_bytes;own=(HERE/'startup_release_operator.py').resolve();reads=0
        def changed(path):
            nonlocal reads
            raw=read(path)
            if path==own:
                reads+=1
                if reads==2:return raw+b'\n# synthetic public source change\n'
            return raw
        with mock.patch.object(o,'exec',side_effect=self.observe_helpers(observed),create=True),mock.patch.object(Path,'read_bytes',new=changed),mock.patch.object(o,'_candidate_runtime_pins',side_effect=self.observe_runtime(checks)):
            with self.assertRaisesRegex(ValueError,'candidate_preparation_source_changed:load'):o._candidate_preparation_sources()
        self.assertEqual(reads,2);self.assertEqual(len(observed),1)
        checks[0].assert_not_called();observed[0][1].verify_sources.assert_not_called()

    def test_private_helpers_load_fresh_code_and_keep_full_runtime_at_point_of_use(self):
        observed=[];checks=[]
        with mock.patch.object(o,'exec',side_effect=self.observe_helpers(observed),create=True),mock.patch.object(o,'_candidate_runtime_pins',side_effect=self.observe_runtime(checks)):
            a,j,_,_,_=o._candidate_preparation_sources()
            first=a._candidate_helpers();first[0].synthetic_marker='private first load only'
            second=a._candidate_helpers()
            self.assertEqual(len(observed),3);self.assertEqual(len(checks),1);self.assertEqual(checks[0].call_count,1)
            self.assertIs(j,observed[0][0]);self.assertEqual(first,observed[1]);self.assertEqual(second,observed[2])
            self.assertEqual(len({id(module)for modules in observed for module in modules}),9)
            self.assertIs(first[0].parse.__code__,second[0].parse.__code__)
            self.assertIs(first[1].clock.__code__,second[1].clock.__code__)
            self.assertIsNot(first[0].parse,second[0].parse)
            self.assertIsNot(first[0].parse.__globals__,second[0].parse.__globals__)
            self.assertFalse(hasattr(second[0],'synthetic_marker'))
            for modules in (first,second):
                self.assertEqual(modules[0].candidate_pins.__name__,'candidate_pins');self.assertEqual(modules[1].verify_sources.call_count,1)
            self.assertIsNot(first[0].candidate_pins,second[0].candidate_pins)
            j.candidate_pins.assert_not_called()
            self.assertEqual(first[0].candidate_pins(),REPO);self.assertEqual(checks[0].call_count,2)
            self.assertEqual(second[0].candidate_pins(),REPO);self.assertEqual(checks[0].call_count,3)

    def test_private_loader_does_not_change_default_modules_or_share_between_factories(self):
        default=o.startup_manifest();default_loader=default._candidate_python_helpers;default_helpers=default._candidate_helpers
        _,default_modules=default_loader();default_pins=default_modules[0].candidate_pins
        observed=[];checks=[]
        with mock.patch.object(o,'exec',side_effect=self.observe_helpers(observed),create=True),mock.patch.object(o,'_candidate_runtime_pins',side_effect=self.observe_runtime(checks)):
            first,_,_,_,_=o._candidate_preparation_sources();second,_,_,_,_=o._candidate_preparation_sources()
            left=first._candidate_helpers();right=second._candidate_helpers()
            self.assertIsNot(first,second);self.assertIsNot(left[0],right[0]);self.assertIsNot(checks[0],checks[1])
            self.assertIsNot(first.compile,second.compile);self.assertIsNot(left[0].parse.__code__,right[0].parse.__code__)
            self.assertIsNot(left[0].candidate_pins,right[0].candidate_pins)
            self.assertEqual([check.call_count for check in checks],[1,1])
            left[0].candidate_pins();right[0].candidate_pins()
            self.assertEqual([check.call_count for check in checks],[2,2])
        self.assertIs(default._candidate_python_helpers,default_loader);self.assertIs(default._candidate_helpers,default_helpers)
        self.assertIs(default_modules[0].candidate_pins,default_pins)
        _,fresh_default=default_loader();self.assertEqual(fresh_default[0].candidate_pins.__name__,'candidate_pins')
        self.assertIsNot(fresh_default[0],left[0]);self.assertIsNot(fresh_default[0],right[0])

    def test_private_compiler_reuses_only_exact_fixed_source_and_policy_after_fresh_loads(self):
        with mock.patch.object(o,'compile',wraps=compile,create=True)as compiled:
            a,j,_,_,unchanged=o._candidate_preparation_sources();before=compiled.call_count
            first=a._candidate_helpers();second=a._candidate_helpers()
            self.assertEqual(compiled.call_count,before);self.assertIs(first[0].parse.__code__,second[0].parse.__code__)
            self.assertIsNot(first[0].parse.__globals__,second[0].parse.__globals__)
            path=REPO/next(iter(a.CANDIDATE_HELPER_PINS));raw=path.read_bytes()
            with self.assertRaisesRegex(ValueError,'compile_scope'):a.compile(raw,str(path),'exec',flags=1,dont_inherit=True)
            with self.assertRaisesRegex(ValueError,'source_pin'):a.compile(raw+b'\n# public changed source\n',str(path),'exec',dont_inherit=True)
            self.assertEqual(compiled.call_count,before)
            self.assertEqual(set(a.compile.__code__.co_freevars),{'compiled_helpers','helper_pins'})
        self.assertNotIn('compile',o.startup_manifest().__dict__)

    def test_pure_load_rechecks_rust_jito_repository_and_policy_before_return(self):
        observed=[];checks=[];read=Path.read_bytes
        with mock.patch.object(o,'exec',side_effect=self.observe_helpers(observed),create=True),mock.patch.object(o,'_candidate_runtime_pins',side_effect=self.observe_runtime(checks)):
            a,j,_,_,unchanged=o._candidate_preparation_sources();load=a._candidate_python_helpers
            sources=[(REPO/next(iter(observed[0][1].SOURCE_PINS)),'startup_rule_source_pin'),
                (REPO/next(iter(a.JITO_SOURCE_PINS)),'jito_source_changed')]
            for target,reason in sources:
                def changed(path):
                    raw=read(path);return raw+b'\n# synthetic public source drift\n'if path==target else raw
                with self.subTest(source=target.name),mock.patch.object(Path,'read_bytes',new=changed),self.assertRaisesRegex(ValueError,reason):
                    a._candidate_helpers()
            def drift():
                repo,modules=load();verify=modules[1].verify_sources
                def rules(path):
                    verify(path);modules[0].SDK_SHA='0'*64
                modules[1].verify_sources=rules;return repo,modules
            with mock.patch.object(a,'_candidate_python_helpers',side_effect=drift),self.assertRaisesRegex(ValueError,'runtime_policy_changed'):
                a._candidate_helpers()
            # A repository change cannot be admitted through the pure span.
            with mock.patch.dict(os.environ,{'PIV1_PUBLIC_CHECK_REPO':str(REPO/'foreign')}),self.assertRaisesRegex(ValueError,'candidate_helper_repo'):
                a._candidate_helpers()
            self.assertEqual(checks[0].call_count,1);self.assertNotEqual(j.SDK_SHA,'0'*64)
            a._candidate_helpers();self.assertEqual(checks[0].call_count,1)

    def test_node_request_keeps_distinct_base_planner_candidate_gates_before_packet_or_process(self):
        a,_,_,_,unchanged=o._candidate_preparation_sources();j,_,_=a._candidate_helpers()
        # Refusal at each actual-use gate must precede encoding and Popen.
        # The full real candidate gate is exercised separately above.
        with mock.patch.object(j,'pins',side_effect=o.Stop('base_runtime_gate'))as base,\
                mock.patch.object(j,'candidate_pins',side_effect=o.Stop('candidate_runtime_gate'))as candidate,\
                mock.patch.object(j,'encode',side_effect=AssertionError('packet_before_gate'))as packet,\
                mock.patch.object(j.subprocess,'Popen',side_effect=AssertionError('process_before_gate'))as process:
            with self.assertRaisesRegex(ValueError,'base_runtime_gate'):j._node_request([])
            base.assert_called_once_with(False);candidate.assert_not_called();base.reset_mock()
            with self.assertRaisesRegex(ValueError,'base_runtime_gate'):j._node_request([],planner=True)
            base.assert_called_once_with(True);candidate.assert_not_called();base.reset_mock()
            with self.assertRaisesRegex(ValueError,'candidate_runtime_gate'):j._node_request([],candidate_planner=True)
            candidate.assert_called_once_with();base.assert_not_called();packet.assert_not_called();process.assert_not_called()

    def test_each_private_load_refuses_helper_source_and_runtime_faults_before_rules(self):
        observed=[];checks=[];read=Path.read_bytes;target=HERE/'startup_public_journal.py'
        with mock.patch.object(o,'exec',side_effect=self.observe_helpers(observed),create=True),mock.patch.object(o,'_candidate_runtime_pins',side_effect=self.observe_runtime(checks)):
            a,j,_,_,_=o._candidate_preparation_sources()
            def changed(path):
                raw=read(path);return raw+b'\n# synthetic public helper source fault\n'if path==target else raw
            with mock.patch.object(Path,'read_bytes',new=changed),self.assertRaisesRegex(ValueError,'candidate_helper_source_pin'):
                a._candidate_helpers()
            self.assertEqual(len(observed),1);self.assertEqual(checks[0].call_count,1)
            with tempfile.TemporaryDirectory()as temp:
                invalid=Path(temp)/'invalid-public-runtime';invalid.write_bytes(b'PUBLIC_INVALID_RUNTIME')
                with mock.patch.object(j,'NODE',invalid),self.assertRaisesRegex(ValueError,'runtime_policy_changed'):a._candidate_helpers()
            self.assertEqual(len(observed),2);self.assertEqual(checks[0].call_count,1)
            observed[-1][1].verify_sources.assert_not_called()
            # Neither refusal reuses a prior helper or a validity result.
            valid=a._candidate_helpers();self.assertEqual(len(observed),3);self.assertEqual(checks[0].call_count,1)
            self.assertEqual(valid[1].verify_sources.call_count,1)

    def test_fresh_and_matching_initial_module_drift_refuse_immutable_policy_before_rules(self):
        observed=[];checks=[]
        with mock.patch.object(o,'exec',side_effect=self.observe_helpers(observed),create=True),mock.patch.object(o,'_candidate_runtime_pins',side_effect=self.observe_runtime(checks)):
            a,j,_,_,_=o._candidate_preparation_sources();load=a._candidate_python_helpers
            with tempfile.TemporaryDirectory()as temp:
                invalid=Path(temp)/'invalid-public-runtime';invalid.write_bytes(b'PUBLIC_INVALID_RUNTIME')
                def changed():
                    repo,modules=load();modules[0].NODE=invalid;return repo,modules
                # Fresh-only drift and identical drift of both modules must fail
                # against the independent snapshot, before any runtime/rules call.
                with mock.patch.object(a,'_candidate_python_helpers',side_effect=changed):
                    with self.assertRaisesRegex(ValueError,'runtime_policy_changed'):a._candidate_helpers()
                    observed[-1][1].verify_sources.assert_not_called()
                    with mock.patch.object(j,'NODE',invalid),self.assertRaisesRegex(ValueError,'runtime_policy_changed'):a._candidate_helpers()
                    observed[-1][1].verify_sources.assert_not_called()
            self.assertEqual(checks[0].call_count,1)

    def test_planner_mapping_in_place_drift_cannot_rebind_both_module_policies(self):
        observed=[];checks=[]
        with mock.patch.object(o,'exec',side_effect=self.observe_helpers(observed),create=True),mock.patch.object(o,'_candidate_runtime_pins',side_effect=self.observe_runtime(checks)):
            a,j,_,_,_=o._candidate_preparation_sources();load=a._candidate_python_helpers
            key=next(iter(j.CANDIDATE_PLANNER_PINS));original=j.CANDIDATE_PLANNER_PINS[key]
            def changed():
                repo,modules=load();modules[0].CANDIDATE_PLANNER_PINS[key]='0'*64;return repo,modules
            with mock.patch.dict(j.CANDIDATE_PLANNER_PINS,{key:'0'*64}),mock.patch.object(a,'_candidate_python_helpers',side_effect=changed):
                with self.assertRaisesRegex(ValueError,'runtime_policy_changed'):a._candidate_helpers()
            observed[-1][1].verify_sources.assert_not_called();self.assertEqual(checks[0].call_count,1)
            self.assertEqual(j.CANDIDATE_PLANNER_PINS[key],original)
            self.assertEqual(a._candidate_helpers()[0].CANDIDATE_PLANNER_PINS[key],original)
            self.assertEqual(checks[0].call_count,1)

    def test_policy_drift_during_point_use_runtime_read_refuses(self):
        observed=[];checks=[]
        with mock.patch.object(o,'exec',side_effect=self.observe_helpers(observed),create=True),mock.patch.object(o,'_candidate_runtime_pins',side_effect=self.observe_runtime(checks)):
            a,j,_,_,_=o._candidate_preparation_sources();check=checks[0].side_effect
            def drift():
                repo=check();observed[-1][0].SDK_SHA='0'*64;return repo
            fresh=a._candidate_helpers();checks[0].side_effect=drift
            with self.assertRaisesRegex(ValueError,'runtime_policy_changed'):fresh[0].candidate_pins()
            self.assertEqual(fresh[1].verify_sources.call_count,1);self.assertEqual(checks[0].call_count,2)
            self.assertNotEqual(j.SDK_SHA,'0'*64)

    def test_runtime_reference_releases_without_gc_and_expired_scope_validates_fresh(self):
        factory=o._candidate_runtime_pins;references=[]
        def observed(j):
            guard=factory(j);references.append(o.weakref.ref(guard));return guard
        with mock.patch.object(o,'_candidate_runtime_pins',side_effect=observed)as created:
            a,j,_,_,unchanged=o._candidate_preparation_sources()
            _,modules=a._candidate_python_helpers();fresh_j=modules[0]
            self.assertEqual(fresh_j.candidate_pins(),REPO);self.assertEqual(created.call_count,1)
            self.assertIsNotNone(references[0]())
            del unchanged
            # No explicit collection: module/function cycles cannot own Node bytes.
            self.assertIsNone(references[0]())
            self.assertEqual(a._candidate_helpers()[0].candidate_pins(),REPO)
            self.assertEqual(created.call_count,2);self.assertTrue(all(ref()is None for ref in references))
            del a
            self.assertEqual(fresh_j.candidate_pins(),REPO);self.assertEqual(created.call_count,3)
            self.assertTrue(all(ref()is None for ref in references))
            # The fallback remains a complete real runtime gate, never a pass.
            with tempfile.TemporaryDirectory()as temp:
                invalid=Path(temp)/'invalid-public-runtime';invalid.write_bytes(b'PUBLIC_INVALID_RUNTIME')
                with mock.patch.object(j,'NODE',invalid),self.assertRaisesRegex(ValueError,'runtime_policy_changed'):
                    fresh_j.candidate_pins()
            self.assertEqual(created.call_count,3)
            with mock.patch.object(j,'sha',return_value='0'*64)as fresh_sha:
                with self.assertRaisesRegex(ValueError,'source_pin'):fresh_j.candidate_pins()
                fresh_sha.assert_called_once()
            self.assertEqual(created.call_count,4)


class CandidateRuntimeSourceBytesTests(unittest.TestCase):
    """Synthetic public files exercise real full reads, not trusted stat results."""
    def setUp(self):
        temp=tempfile.TemporaryDirectory();self.addCleanup(temp.cleanup);self.repo=Path(temp.name)
        self.node=self.repo/'node';self.original=b'PUBLIC_RUNTIME_BYTES_0123456789';self.node.write_bytes(self.original)
        self.names=['clients/cli/startup-public-check.cjs','spikes/task-0.4-jito/package-lock.json',
            'spikes/task-0.4-jito/node_modules/@solana/web3.js/lib/index.cjs.js',
            'clients/cli/initializer-existing-alt-plan.cjs','clients/cli/initializer-plan.cjs']
        self.raw={}
        for index,name in enumerate(self.names):
            path=self.repo/name;path.parent.mkdir(parents=True,exist_ok=True)
            self.raw[path]=('PUBLIC_PIN_'+str(index)).encode();path.write_bytes(self.raw[path])
        self.j=types.SimpleNamespace(__file__=str(self.repo/'tools/startup_public_journal.py'),NODE=self.node,
            NODE_SHA=o.sha(self.original),CHECKER_SHA=o.sha(self.raw[self.repo/self.names[0]]),
            LOCK_SHA=o.sha(self.raw[self.repo/self.names[1]]),SDK_SHA=o.sha(self.raw[self.repo/self.names[2]]),
            CANDIDATE_PLANNER_PINS={name:o.sha(self.raw[self.repo/name])for name in self.names[3:]},sha=o.sha,require=o.need)
        self.environment=mock.patch.dict(os.environ,{'PIV1_PUBLIC_CHECK_REPO':str(self.repo)})
        self.environment.start();self.addCleanup(self.environment.stop)

    def test_runtime_policy_path_order_constants_and_fresh_environment_match_default(self):
        _,(default,_,_)=o.startup_manifest()._candidate_python_helpers()
        attrs={name:getattr(self.j,name)for name in ('NODE','NODE_SHA','CHECKER_SHA','LOCK_SHA','SDK_SHA','CANDIDATE_PLANNER_PINS')}
        real_open=Path.open;real_os_open=os.open;events=[]
        def opened(path,*args,**kwargs):events.append(path);return real_open(path,*args,**kwargs)
        def fd_opened(path,*args,**kwargs):events.append(Path(path));return real_os_open(path,*args,**kwargs)
        with mock.patch.multiple(default,**attrs),mock.patch.object(Path,'open',new=opened),mock.patch.object(o.os,'open',side_effect=fd_opened):
            self.assertEqual(default.candidate_pins(),self.repo);expected=list(events);events.clear()
            pins=o._candidate_runtime_pins(default);self.assertEqual(pins(),self.repo)
            self.assertEqual(events,expected);self.assertEqual(events,[self.node]+[self.repo/name for name in self.names])
        # Identical public bytes in a different repo still select its current path;
        # the factory's independent resolved-repo equality must reject that result.
        other=self.repo/'other'
        for name in self.names:
            path=other/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(self.raw[self.repo/name])
        with mock.patch.multiple(default,**attrs),mock.patch.dict(os.environ,{'PIV1_PUBLIC_CHECK_REPO':str(other)}):self.assertEqual(pins(),other)

    def test_each_closure_authenticates_sha_then_every_boundary_rereads_exact_bytes(self):
        hashes=[];original_sha=self.j.sha
        def digest(raw):hashes.append(raw);return original_sha(raw)
        self.j.sha=digest;pins=o._candidate_runtime_pins(self.j)
        real_read=os.read;reads=[]
        def read(fd,size):raw=real_read(fd,size);reads.append(raw);return raw
        with mock.patch.object(o.os,'read',side_effect=read):
            pins();pins();o._candidate_runtime_pins(self.j)()
        self.assertEqual(hashes.count(self.original),2);self.assertEqual(reads.count(self.original),3)
        self.j.NODE_SHA='0'*64
        with self.assertRaisesRegex(ValueError,'source_pin'):o._candidate_runtime_pins(self.j)()

    def test_same_size_restored_mtime_truncation_and_growth_never_reuse_validity(self):
        pins=o._candidate_runtime_pins(self.j);pins();meta=self.node.stat()
        for raw in (b'X'+self.original[1:],self.original[:-1],self.original+b'X'):
            with self.subTest(size=len(raw)):
                self.node.write_bytes(raw);os.utime(self.node,ns=(meta.st_atime_ns,meta.st_mtime_ns))
                with self.assertRaisesRegex(ValueError,'source_pin'):pins()
                self.node.write_bytes(self.original);pins()

    def test_symlink_nonregular_and_oversized_runtime_refuse_without_blocking(self):
        target=self.repo/'public-target';target.write_bytes(self.original);self.node.unlink();self.node.symlink_to(target)
        with self.assertRaises(OSError):o._candidate_runtime_pins(self.j)()
        self.node.unlink();self.node.mkdir()
        with self.assertRaisesRegex(ValueError,'source_file'):o._candidate_runtime_pins(self.j)()
        self.node.rmdir();os.mkfifo(self.node)
        with self.assertRaisesRegex(ValueError,'source_file'):o._candidate_runtime_pins(self.j)()
        self.node.unlink()
        with self.node.open('wb')as stream:stream.truncate(128*1024*1024+1)
        with self.assertRaisesRegex(ValueError,'source_file'):o._candidate_runtime_pins(self.j)()

    def test_midread_path_replacement_and_metadata_changes_refuse_and_close_fd(self):
        real_read=os.read
        for fault in ('replace','mtime','grow'):
            self.node.write_bytes(self.original);seen=[]
            def read(fd,size):
                raw=real_read(fd,size)
                if not seen:
                    seen.append(fd)
                    if fault=='replace':
                        self.node.rename(self.repo/'displaced');self.node.write_bytes(self.original)
                    elif fault=='mtime':
                        meta=self.node.stat();os.utime(self.node,ns=(meta.st_atime_ns,meta.st_mtime_ns+1_000_000))
                    else:
                        with self.node.open('ab')as stream:stream.write(b'X')
                return raw
            with self.subTest(fault=fault),mock.patch.object(o.os,'read',side_effect=read):
                with self.assertRaisesRegex(ValueError,'source_file_changed'):o._candidate_runtime_pins(self.j)()
            with self.assertRaises(OSError):os.fstat(seen[0])

    def test_all_non_node_pins_are_fresh_on_every_guard(self):
        pins=o._candidate_runtime_pins(self.j);pins()
        for index,name in enumerate(self.names):
            path=self.repo/name
            with self.subTest(path=name):
                path.write_bytes(self.raw[path]+b'CHANGED')
                with self.assertRaisesRegex(ValueError,'candidate_source_pin'if index>=3 else 'source_pin'):pins()
                path.write_bytes(self.raw[path]);pins()

    def test_streamed_multichunk_short_reads_authenticate_once_and_reread_to_eof(self):
        self.original=bytes(range(256))*8192+b'PUBLIC_RUNTIME_TAIL';self.node.write_bytes(self.original);self.j.NODE_SHA=o.sha(self.original)
        hashes=[];digest=self.j.sha;read=os.read;reads=[];sizes=(7,1048573,13,65537,524299)
        def hashed(raw):hashes.append(raw);return digest(raw)
        def partial(fd,size):
            raw=read(fd,min(size,sizes[len(reads)%len(sizes)]));reads.append(raw);return raw
        self.j.sha=hashed;pins=o._candidate_runtime_pins(self.j)
        with mock.patch.object(o.os,'read',side_effect=partial):
            self.assertEqual(pins(),self.repo);self.assertEqual(b''.join(reads),self.original);self.assertEqual(reads[-1],b'')
            reads.clear()
            self.assertEqual(pins(),self.repo);self.assertEqual(b''.join(reads),self.original);self.assertEqual(reads[-1],b'')
        self.assertGreater(len(reads),3);self.assertEqual(hashes.count(self.original),1)

    def test_streamed_late_byte_mismatch_reads_through_eof_and_closes(self):
        self.original=bytes(range(256))*8192+b'PUBLIC_RUNTIME_TAIL';self.node.write_bytes(self.original);self.j.NODE_SHA=o.sha(self.original)
        pins=o._candidate_runtime_pins(self.j);pins();changed=self.original[:-1]+b'X';self.node.write_bytes(changed)
        read=os.read;reads=[];fds=[]
        def observed(fd,size):
            raw=read(fd,min(size,65537));reads.append(raw);fds.append(fd);return raw
        with mock.patch.object(o.os,'read',side_effect=observed),self.assertRaisesRegex(ValueError,'source_pin'):
            pins()
        self.assertEqual(b''.join(reads),changed);self.assertEqual(reads[-1],b'');self.assertGreater(len(reads),3)
        with self.assertRaises(OSError):os.fstat(fds[0])

    def test_streamed_early_mismatch_and_metadata_fault_keep_metadata_priority(self):
        self.original=bytes(range(256))*8192+b'PUBLIC_RUNTIME_TAIL';self.node.write_bytes(self.original);self.j.NODE_SHA=o.sha(self.original)
        pins=o._candidate_runtime_pins(self.j);pins();read=os.read;reads=[];fds=[]
        def mismatched(fd,size):
            raw=read(fd,min(size,65537));fds.append(fd)
            if not reads:
                raw=b'X'+raw[1:];meta=self.node.stat()
                os.utime(self.node,ns=(meta.st_atime_ns,meta.st_mtime_ns+1_000_000))
            reads.append(raw);return raw
        with mock.patch.object(o.os,'read',side_effect=mismatched),self.assertRaisesRegex(ValueError,'^source_file_changed$'):
            pins()
        self.assertEqual(sum(map(len,reads)),len(self.original));self.assertEqual(reads[-1],b'');self.assertGreater(len(reads),3)
        self.assertNotEqual(b''.join(reads),self.original)
        with self.assertRaises(OSError):os.fstat(fds[0])


class CandidateFundingCurrentRuntimeGateTests(unittest.TestCase):
    """Actual HOST current capture keeps the real full source gate before transport."""
    @classmethod
    def setUpClass(cls):
        cls.host=CandidateFundingSubmissionTests;cls.host.setUpClass()

    def setUp(self):
        self.component=self.host(methodName='test_current_source_can_change_while_quote_packet_h2_and_original_frames_stay_fixed')
        self.component.setUp();self.addCleanup(self.component.doCleanups)
        self.op,self.state,_=self.component.opened()
        self.a,self.gate_j,_,_,self.unchanged=o._candidate_preparation_sources()
        self.body=self.component.body()

    def capture(self,unchanged,transport):
        return o._candidate_capture_funding_current(self.a,self.component.j,self.op,self.state,unchanged,_transport=transport)

    def assert_request_only_refusal(self,error):
        capture=error.fundingCurrentCapture;self.assertFalse(capture['currentSnapshotVerified'])
        self.assertEqual(set(capture['frame']),{'requestBase64'})
        self.assertFalse(capture['nativeAdmissionQualified']);self.assertFalse(capture['liveReady'])

    def test_private_python_setup_keeps_all_three_full_guards_in_order(self):
        events=[];original=self.a._candidate_python_helpers
        def unchanged(boundary):events.append(boundary);self.unchanged(boundary)
        def transport(raw):events.append('callback');return 200,[],self.body
        with mock.patch.object(self.a,'_candidate_helpers',side_effect=AssertionError('unexpected_early_runtime_gate'))as full,mock.patch.object(self.a,'_candidate_python_helpers',wraps=original)as python,mock.patch.object(o.os,'open',wraps=o.os.open)as pins:
            out=self.capture(unchanged,transport)
        full.assert_not_called();python.assert_called_once_with();self.assertEqual(sum(call.args[0]==self.gate_j.NODE for call in pins.call_args_list),3)
        self.assertEqual(events,['funding_current_before_callback','callback','funding_current_after_callback','funding_current_after_verification'])
        self.assertEqual(base64.b64decode(out['frame']['responseBase64']),self.body)
        self.assertTrue(out['currentSnapshotVerified']);self.assertFalse(out['nativeAdmissionQualified']);self.assertFalse(out['liveReady'])

    def test_real_runtime_and_repo_gate_refuse_before_callback_with_request_only(self):
        with tempfile.TemporaryDirectory()as temp:
            invalid=Path(temp)/'invalid-public-runtime';invalid.write_bytes(b'PUBLIC_INVALID_RUNTIME')
            transport=mock.Mock(return_value=(200,[],self.body))
            with mock.patch.object(self.gate_j,'NODE',invalid):
                with self.assertRaisesRegex(ValueError,'source_pin')as caught:self.capture(self.unchanged,transport)
            transport.assert_not_called();self.assert_request_only_refusal(caught.exception)
        transport=mock.Mock(return_value=(200,[],self.body))
        resolve=Path.resolve
        def foreign(path,*args,**kwargs):return REPO/'foreign'if path==REPO else resolve(path,*args,**kwargs)
        with mock.patch.object(Path,'resolve',new=foreign):
            with self.assertRaisesRegex(ValueError,'candidate_helper_repo')as caught:self.capture(self.unchanged,transport)
        transport.assert_not_called();self.assert_request_only_refusal(caught.exception)

    def test_real_captured_source_fault_refuses_before_runtime_or_callback(self):
        read=Path.read_bytes;own=(HERE/'startup_release_operator.py').resolve();transport=mock.Mock(return_value=(200,[],self.body))
        def changed(path):
            raw=read(path);return raw+b'\n# synthetic public source fault\n'if path==own else raw
        with mock.patch.object(Path,'read_bytes',new=changed),mock.patch.object(o.os,'open',wraps=o.os.open)as pins:
            with self.assertRaisesRegex(ValueError,'candidate_preparation_source_changed:funding_current_before_callback')as caught:self.capture(self.unchanged,transport)
        pins.assert_not_called();transport.assert_not_called();self.assert_request_only_refusal(caught.exception)

    def test_postcallback_and_postverification_gate_failures_retain_literal_response(self):
        for stop in ('funding_current_after_callback','funding_current_after_verification'):
            events=[]
            def unchanged(boundary):
                events.append(boundary);self.unchanged(boundary)
                if boundary==stop:raise ValueError('injected_after_real_full_gate')
            def transport(raw):events.append('callback');return 200,[],self.body
            with self.subTest(boundary=stop),mock.patch.object(o.os,'open',wraps=o.os.open)as pins:
                with self.assertRaisesRegex(ValueError,'injected_after_real_full_gate')as caught:self.capture(unchanged,transport)
                self.assertEqual(sum(call.args[0]==self.gate_j.NODE for call in pins.call_args_list),2 if stop=='funding_current_after_callback'else 3)
                capture=caught.exception.fundingCurrentCapture;self.assertFalse(capture['currentSnapshotVerified'])
                self.assertEqual(base64.b64decode(capture['frame']['responseBase64']),self.body)
                self.assertIn('httpBase64',capture['frame']);self.assertEqual(events[0],'funding_current_before_callback');self.assertEqual(events[1],'callback');self.assertEqual(events[-1],stop)
