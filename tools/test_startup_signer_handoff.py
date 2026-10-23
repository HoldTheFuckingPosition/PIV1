"""Public signer-boundary tests using retained historical signatures only.

No signer backend, private input, key creation, signing or real transport.
The generic historical fixtures are offline integration seams, not startup or
current key-control evidence. Full canonical coverage is exercised separately.
"""
import base64
import copy
import importlib.util
import json
from pathlib import Path
import unittest
from unittest import mock

HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location("signer_historical_helpers",HERE/"test_startup_release_operator.py")
h=importlib.util.module_from_spec(spec);spec.loader.exec_module(h);o=h.o


class SignerHandoffTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls): h.OperatorTests.setUpClass()

    def setUp(self):
        self.host=h.OperatorTests(methodName="test_nullable_imports_no_helpers")
        self.host.setUp();self.addCleanup(self.host.doCleanups)
        patch=mock.patch.object(o,"helpers",return_value=(self.host.r,self.host.c,self.host.s,self.host.j))
        patch.start();self.addCleanup(patch.stop)
        self.pid=self.host.phase["id"]

    def records(self):
        return [json.loads(p.read_bytes()) for p in sorted(o.journal_path(self.host.base).glob("*.json"))]

    def begin(self):
        self.host.create();self.host.call("PREPARE",[self.host.prepared])
        return self.host.call("HANDOFF",[self.pid],**self.host.auth())

    def response(self, handoff):
        return {"schema":"piv1-startup-signing-response-v1","requestId":handoff["request"]["requestId"],
            "packets":[{"id":self.pid,"wireBase64":self.host.tx["signedWireBase64"]}]}

    def changed_auth(self):
        raw=json.loads(self.host.auth()["authorization_raw"])
        raw["expiresUnixTimestamp"]="18446744073709551614";raw=o.encode(raw)
        return {"authorization_raw":raw,"authorization_sha256":o.sha(raw)}

    def test_durable_exact_request_lost_output_recovery_and_consumption(self):
        before=self.host.create();self.host.call("PREPARE",[self.host.prepared]);prepared_tail=self.host.tail
        result=self.host.call("HANDOFF",[self.pid],**self.host.auth());request=result["request"]
        self.assertNotEqual(result["tail"],prepared_tail)
        self.assertEqual(request["operatorTail"],prepared_tail)
        self.assertEqual(request["operatorSourceSha256"],o.sha((HERE/"startup_release_operator.py").read_bytes()))
        self.assertEqual(request["baseParentPath"],str(self.host.base))
        self.assertEqual(request["baseParentTail"],self.host.manifest["parentTail"])
        self.assertEqual(request["manifestSha256"],o.sha(o.encode(self.host.manifest)))
        self.assertEqual(request["genesisHash"],self.host.c.GENESIS)
        self.assertEqual(request["cluster"],"testnet")
        self.assertEqual(request["authorizationSha256"],self.host.auth()["authorization_sha256"])
        self.assertIsNone(request["sealSha256"])
        packet=request["packets"][0]
        self.assertEqual(packet["preparedRecordDigest"],prepared_tail)
        self.assertEqual(packet["preparedSha256"],o.sha(o.encode(result["packets"][0])))
        self.assertEqual(packet["requiredSigners"],result["packets"][0]["reviewed"]["requiredSigners"])
        self.assertEqual(len(packet["requiredSigners"]),2)
        self.assertFalse(result["signerInvoked"]);self.assertFalse(result["signerControlProven"])
        self.assertFalse(result["liveReady"]);self.assertFalse(result["founderAuthorizationCryptographicallyVerified"])
        recovered=self.host.call("INSPECT")
        self.assertEqual(recovered["pendingSigningRequest"],request)
        self.assertEqual(recovered["signingPackets"],result["packets"])
        imported=self.host.call("IMPORT",self.response(result),**self.host.auth())
        self.assertIsNone(imported["pendingSigningRequest"])
        self.assertEqual(imported["historicalLedger"],before["historicalLedger"])
        self.assertEqual(imported["proposedFeeLedger"]["actualFeeLamports"],"0")
        self.assertEqual(self.host.calls,[])
        with self.assertRaisesRegex(o.Stop,"signing_request_binding"):
            self.host.call("IMPORT",self.response(result),**self.host.auth())

    def test_identical_prepare_and_new_handoff_invalidate_old_response(self):
        first=self.begin();response=self.response(first)
        self.host.call("PREPARE",[self.host.prepared])
        self.assertIsNone(self.host.call("INSPECT")["pendingSigningRequest"])
        with self.assertRaisesRegex(o.Stop,"signing_request_binding"):
            self.host.call("IMPORT",response,**self.host.auth())
        second=self.host.call("HANDOFF",[self.pid],**self.host.auth())
        self.assertNotEqual(first["request"]["requestId"],second["request"]["requestId"])
        with self.assertRaisesRegex(o.Stop,"signing_request_binding"):
            self.host.call("IMPORT",response,**self.host.auth())
        third=self.host.call("HANDOFF",[self.pid],**self.host.auth())
        with self.assertRaisesRegex(o.Stop,"signing_request_binding"):
            self.host.call("IMPORT",self.response(second),**self.host.auth())
        self.host.call("IMPORT",self.response(third),**self.host.auth())

    def test_strict_response_and_all_or_nothing_ordered_batch(self):
        # A second retained two-signature transaction exercises whole-batch
        # rejection; none of these wires is a current or signable startup packet.
        review,life,unsigned=h.phase_from_fixture(self.host.c,2)
        second=copy.deepcopy(self.host.phase);second.update(id="second_public_packet",reviewed=review,feeCapLamports=h.FIXTURE["transactions"][2]["feeLamports"])
        self.host.manifest["phases"].append(second)
        prepared={"id":second["id"],"wireBase64":unsigned,"lifetime":life,"feeQuoteLamports":second["feeCapLamports"]}
        self.host.create();self.host.call("PREPARE",[self.host.prepared,prepared])
        with self.assertRaisesRegex(o.Stop,"handoff_order"):
            self.host.call("HANDOFF",[second["id"],self.pid],**self.host.auth())
        handoff=self.host.call("HANDOFF",[self.pid,second["id"]],**self.host.auth())
        response=self.response(handoff);response["packets"].append({"id":second["id"],"wireBase64":h.FIXTURE["transactions"][2]["signedWireBase64"]})
        bads=[]
        for packets in (None,[],response["packets"][:1],response["packets"]*2,list(reversed(response["packets"])),[response["packets"][0]]*2):
            bad={**response,"packets":packets};bads.append(bad)
        bad={**response,"extra":True};bads.append(bad)
        bad=copy.deepcopy(response);bad["packets"][1]["extra"]=True;bads.append(bad)
        bad=copy.deepcopy(response);del bad["requestId"];bads.append(bad)
        bad=copy.deepcopy(response);bad["packets"][1]["wireBase64"]="invalid";bads.append(bad)
        before=self.records()
        for bad in bads:
            with self.assertRaises(ValueError): self.host.call("IMPORT",bad,**self.host.auth())
            self.assertEqual(self.records(),before)
        self.host.call("IMPORT",response,**self.host.auth())
        state=o.replay(self.host.j,self.records())
        self.assertEqual(list(state["signed"]),[self.pid,second["id"]])

    def test_second_required_signature_and_exact_message_verified(self):
        result=self.begin();good=self.response(result);before=self.records()
        raw=bytearray(base64.b64decode(good["packets"][0]["wireBase64"]))
        self.assertEqual(raw[0],2)
        raw[65]^=1;bad=copy.deepcopy(good);bad["packets"][0]["wireBase64"]=base64.b64encode(raw).decode()
        with self.assertRaisesRegex(ValueError,"signature_invalid"):
            self.host.call("IMPORT",bad,**self.host.auth())
        self.assertEqual(self.records(),before)
        raw=bytearray(base64.b64decode(good["packets"][0]["wireBase64"]));raw[-1]^=1
        bad["packets"][0]["wireBase64"]=base64.b64encode(raw).decode()
        with self.assertRaisesRegex(o.Stop,"signed_prepared_message_binding"):
            self.host.call("IMPORT",bad,**self.host.auth())
        raw=bytearray(base64.b64decode(good["packets"][0]["wireBase64"]));raw[65:129]=bytes(64)
        bad["packets"][0]["wireBase64"]=base64.b64encode(raw).decode()
        with self.assertRaisesRegex(o.Stop,"signed_required_signatures"):
            self.host.call("IMPORT",bad,**self.host.auth())
        self.assertEqual(self.records(),before)
        self.host.call("IMPORT",good,**self.host.auth())

    def test_changed_authorization_and_expiry_stop_import_and_send(self):
        result=self.begin();response=self.response(result);auth=self.host.auth();before=self.records()
        with self.assertRaisesRegex(o.Stop,"signing_authorization_changed"):
            self.host.call("IMPORT",response,**self.changed_auth())
        with mock.patch.object(o.time,"time",return_value=18446744073709551616):
            with self.assertRaisesRegex(o.Stop,"launcher_authorization_expired"):
                self.host.call("IMPORT",response,**auth)
            self.assertEqual(self.host.call("INSPECT")["pendingSigningRequest"],result["request"])
        self.assertEqual(self.records(),before)
        self.host.call("IMPORT",response,**auth)
        with self.assertRaisesRegex(o.Stop,"signing_authorization_changed"):
            self.host.call("SEND",{"id":self.pid},_transport=self.host.transport,**self.changed_auth())
        with mock.patch.object(o.time,"time",return_value=18446744073709551616):
            with self.assertRaisesRegex(o.Stop,"launcher_authorization_expired"):
                self.host.call("SEND",{"id":self.pid},_transport=self.host.transport,**auth)
            self.host.call("INSPECT")
        self.assertEqual(self.host.calls,[])

    def test_unknown_blocks_new_response_but_original_expired_recovery_remains(self):
        result=self.begin();response=self.response(result);auth=self.host.auth()
        self.host.call("IMPORT",response,**auth)
        transport=lambda raw:self.host.transport(raw,timeout=True,unknown=True)
        self.host.call("SEND",{"id":self.pid},_transport=transport,**auth)
        for action,data in (("HANDOFF",[self.pid]),("IMPORT",response),("PREPARE",[self.host.prepared])):
            with self.assertRaisesRegex(o.Stop,"unresolved_intent_blocks_"):
                self.host.call(action,data,**auth)
        with mock.patch.object(o.time,"time",return_value=18446744073709551616):
            self.host.call("INSPECT")
            self.host.call("RECOVER",{"id":self.pid},_transport=transport,**auth)
        self.assertEqual(self.host.calls.count("sendTransaction"),1)
        self.assertIn("getSignatureStatuses",self.host.calls)

    def test_replay_rederives_request_response_and_intent_bindings(self):
        result=self.begin();response=self.response(result)
        chain=self.records()
        for key,value in (("baseParentPath","/different/public/parent"),("operatorSourceSha256","0"*64),("manifestSha256","0"*64),("operatorTail","0"*64),("sealSha256","0"*64)):
            bad=copy.deepcopy(chain);bad[-1]["data"][key]=value
            with self.assertRaisesRegex(o.Stop,"handoff_replay_binding"):o.replay(self.host.j,bad)
        bad=copy.deepcopy(chain);bad[-1]["data"]["packets"][0]["requiredSigners"].reverse()
        with self.assertRaisesRegex(o.Stop,"handoff_replay_binding"):o.replay(self.host.j,bad)
        self.host.call("IMPORT",response,**self.host.auth());chain=self.records()
        for key in ("wireSha256","signatureHex","preparedSha256"):
            bad=copy.deepcopy(chain);bad[-1]["data"]["entries"][0][key]="0"*64
            with self.assertRaisesRegex(o.Stop,"signed_response_replay_binding"):o.replay(self.host.j,bad)
        bad=copy.deepcopy(chain);bad[-1]["data"]["responseSha256"]="0"*64
        with self.assertRaisesRegex(o.Stop,"signing_response_digest"):o.replay(self.host.j,bad)
        self.host.call("SEND",{"id":self.pid},_transport=self.host.transport,**self.host.auth())
        chain=self.records();intent=next(r for r in chain if r["kind"]=="INTENT")
        bad=copy.deepcopy(chain);bad[chain.index(intent)]["data"]["authorizationSha256"]="0"*64
        with self.assertRaisesRegex(o.Stop,"signing_authorization_changed"):o.replay(self.host.j,bad)

    def test_pending_request_cannot_use_legacy_import_bypass(self):
        self.begin();before=self.records()
        with self.assertRaisesRegex(o.Stop,"signing_response_required"):
            self.host.call("IMPORT",[{"id":self.pid,"wireBase64":self.host.tx["signedWireBase64"]}])
        self.assertEqual(self.records(),before)


if __name__=="__main__": unittest.main()
