"""Focused offline operator tests. Existing historical signatures only.

Injected transport is synthetic retrospective response replay, never RPC.
No private key, key creation, signing, simulation or old-suite execution.
"""
import base64
import copy
import importlib.util
import json
import os
from pathlib import Path
import tempfile
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
