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
