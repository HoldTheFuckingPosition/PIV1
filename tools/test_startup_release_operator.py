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
