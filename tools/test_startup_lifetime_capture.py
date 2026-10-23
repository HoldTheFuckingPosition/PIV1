"""Source-quote lifetime tests; historical signatures and synthetic transport.

The full-host ELF/account/message seams are explicit and match retained tests.
No actual network, signer, key read, signing or current-chain observation.
"""
import base64
import copy
import importlib.util
import json
from pathlib import Path
import struct
import unittest
from unittest import mock

HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location("lifetime_full_helpers",HERE/"test_startup_manifest.py")
t=importlib.util.module_from_spec(spec);spec.loader.exec_module(t);o=t.o


class Environment:
    def __init__(self,case):
        self.case=case;self.full=t.StartupManifestTests(methodName="test_actual_full_949_create_reopen_and_record_bound")
        self.full.setUp();case.addCleanup(self.full.doCleanups);self.host=self.full.host;self.a=self.full.a
        self.ref=copy.deepcopy(self.a.oracle());r=self.ref["roles"];self.accounts={}
        for key,digest in self.ref["baseline"].items():
            if not digest["exists"]:
                self.accounts[key]={"exists":False,"owner":None,"executable":None,"lamports":None,"dataBase64":None};continue
            raw=base64.b64decode(self.ref["multisigBase64"]) if key==r["multisig"] else bytes(digest["dataBytes"])
            self.accounts[key]={"exists":True,"owner":digest["owner"],"executable":digest["executable"],"lamports":digest["lamports"],"dataBase64":base64.b64encode(raw).decode()}
            self.ref["baseline"][key]={**digest,"dataSha256":o.sha(raw)}
        self.full.stack.enter_context(mock.patch.object(self.a,"oracle",return_value=self.ref))
        self.full.stack.enter_context(mock.patch.object(o,"helpers",return_value=(self.host.r,self.host.c,self.host.s,self.host.j)))
        review,life,unsigned=t.h.phase_from_fixture(self.host.c,1);self.review=review
        self.full.stack.enter_context(mock.patch.object(o,"phase_template",return_value=review))
        self.pid=self.host.manifest["phases"][0]["id"];self.host.prepared={"id":self.pid,"wireBase64":unsigned,"lifetime":life,"feeQuoteLamports":"10000"}
        self.original=life["recentBlockhash"];self.rotated=self.host.c.b58encode(bytes([9])*32)
        self.slot=int(self.host.tx["slot"]);self.capturing=False;self.capture_context=self.slot-10;self.sent=False;self.calls=[];self.queries=[];self.fault=None
        self.fee=10000;self.height=150;self.send_height=200;self.last=1000;self.host.create()

    def records(self):return [json.loads(p.read_bytes()) for p in sorted(o.journal_path(self.host.base).glob("*.json"))]

    def capture(self,packets=None,auth=None):
        self.capturing=True
        self.capture_context=max(self.slot-10,o.replay(self.host.j,self.records())["floor"])
        try:return self.host.call("PREPARE",{"captureLifetime":True,"packets":packets or [{"id":self.pid,"wireBase64":self.host.prepared["wireBase64"]}]},_transport=self.transport,**(auth or self.host.auth()))
        finally:self.capturing=False

    def signed(self,auth=None):
        auth=auth or self.host.auth();handoff=self.host.call("HANDOFF",[self.pid],**auth)
        response={"schema":"piv1-startup-signing-response-v1","requestId":handoff["request"]["requestId"],"packets":[{"id":self.pid,"wireBase64":self.host.tx["signedWireBase64"]}]}
        self.host.call("IMPORT",response,**auth)

    def transport(self,raw):
        queries=json.loads(raw);self.queries.append(queries);replies=[];c=self.host.c;r=self.ref["roles"]
        ctx=self.capture_context if self.capturing else self.slot-1
        for q in queries:
            method=q["method"];self.calls.append(method)
            if method=="getGenesisHash":value="wrong-genesis" if self.fault=="genesis" else c.GENESIS
            elif method=="getVersion":value=self.ref["version"]
            elif method=="getLatestBlockhash":value={"context":{"slot":1 if self.fault=="latest_context" else ctx},"value":{"blockhash":self.original if self.capturing else self.rotated,"lastValidBlockHeight":self.last if self.capturing else 1200}}
            elif method=="getBlockHeight":value=self.height if self.capturing else self.send_height
            elif method=="isBlockhashValid":
                self.case.assertEqual(q["params"][0],self.original)
                value={"context":{"slot":1 if self.fault=="valid_context" else ctx+1},"value":self.fault!="invalid"}
            elif method=="getFeeForMessage":value={"context":{"slot":1 if self.fault=="fee_context" else ctx+1},"value":9999 if self.fault=="fee" else self.fee}
            elif method=="getMinimumBalanceForRentExemption":value=int(self.ref["rents"][str(q["params"][0])])
            elif method=="getMultipleAccounts":
                accounts=copy.deepcopy(self.accounts)
                accounts[r["clock"]]["dataBase64"]=base64.b64encode(struct.pack('<QqQQq',ctx,0,0,0,100)).decode()
                value={"context":{"slot":ctx},"value":[self.host.wire_account(accounts[key]) for key in q["params"][0]]}
            elif method=="sendTransaction":
                self.case.assertEqual(self.records()[-1]["kind"],"INTENT");self.sent=True;value=self.host.tx["signature"]
            else:raise AssertionError(method)
            replies.append({"jsonrpc":"2.0","id":q["id"],"result":value})
        if self.fault=="transport":raise c.TransportFailure("injected public read loss",b'partial-public-response')
        return 200,[],o.encode(replies)


class LifetimeCaptureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):t.StartupManifestTests.setUpClass()

    def setUp(self):self.env=Environment(self)

    def test_captured_original_survives_latest_rotation_and_legacy_stays_exact(self):
        # A separate unquoted parent still requires exact latest equality.
        legacy=Environment(self)
        try:
            legacy.host.call("PREPARE",[legacy.host.prepared]);legacy.signed()
            with self.assertRaisesRegex(o.Stop,"fresh_blockhash_lifetime_binding"):
                legacy.host.call("SEND",{"id":legacy.pid},_transport=legacy.transport,**legacy.host.auth())
            self.assertFalse(legacy.sent)
        finally:legacy.full.doCleanups()
        e=self.env;e.capture();e.signed()
        result=e.host.call("SEND",{"id":e.pid},_transport=e.transport,**e.host.auth())
        self.assertTrue(e.sent);self.assertEqual(e.calls.count("sendTransaction"),1)
        self.assertEqual(result["status"],"UNKNOWN_OR_SUBMITTED_RECOVERY_ONLY")
        self.assertEqual(result["proposedFeeLedger"]["reservedFeeLamports"],"10000")
        self.assertFalse(result["liveReady"]);self.assertFalse(result["signerInvoked"])
        with mock.patch.object(o.time,"time",return_value=18446744073709551616):
            self.assertEqual(e.host.call("INSPECT")["tail"],result["tail"])

    def test_only_blockhash_rewritten_exact_fee_quote_and_23_packet_budget(self):
        e=self.env;scaffold=copy.deepcopy(e.host.prepared);wire=bytearray(base64.b64decode(scaffold["wireBase64"]))
        message=base64.b64decode(e.review["messageBase64"]);at=(4+32*message[3]);offset=1+64*wire[0]
        wire[offset+at:offset+at+32]=bytes([7])*32;scaffold["wireBase64"]=base64.b64encode(wire).decode()
        e.capture([{"id":e.pid,"wireBase64":scaffold["wireBase64"]}]);state=o.replay(e.host.j,e.records());prepared=state["prepared"][e.pid]
        self.assertEqual(prepared["wireBase64"],e.host.prepared["wireBase64"])
        self.assertEqual(set(prepared),{"id","wireBase64","reviewed","lifetime","feeQuoteLamports"})
        self.assertEqual(prepared["lifetime"]["currentBlockHeight"],"150");self.assertEqual(prepared["lifetime"]["lastValidBlockHeight"],"1000")
        self.assertEqual(state["floor"],e.slot-9)
        before=len(e.queries)
        packets=[{"id":e.host.manifest["phases"][i]["id"],"wireBase64":e.host.prepared["wireBase64"]} for i in range(1,24)]
        e.fee=5000;e.capture(packets)
        self.assertEqual(len(e.queries)-before,6)
        self.assertEqual(len([q for batch in e.queries[before:] for q in batch if q["method"]=="getFeeForMessage"]),23)
        for batch in e.queries[before:]:
            for q in batch:
                if q["method"]=="getBlockHeight":self.assertGreaterEqual(q["params"][0]["minContextSlot"],e.slot-9)
        before=len(e.queries)
        with self.assertRaisesRegex(o.Stop,"quote_batch_bound"):
            e.capture(packets+[packets[-1]])
        self.assertEqual(len(e.queries),before);self.assertEqual(o.MAX_RECORDS,8192)

    def test_invalid_stale_fee_expiry_and_authorization_refuse_before_prepare(self):
        e=self.env;before=e.records()
        for fault in ("genesis","latest_context","valid_context","fee_context","invalid","fee"):
            e.fault=fault
            with self.assertRaises(ValueError):e.capture()
            self.assertEqual(e.records(),before)
        e.fault=None;e.last=160
        with self.assertRaisesRegex(o.Stop,"quote_lifetime_margin"):e.capture()
        e.last=1000;reads=len(e.queries)
        with mock.patch.object(o.time,"time",return_value=18446744073709551616):
            with self.assertRaisesRegex(o.Stop,"launcher_authorization_expired"):e.capture()
        self.assertEqual(len(e.queries),reads)
        clock=[1];original=e.transport
        def expire(raw):
            result=original(raw)
            if any(q["method"]=="getBlockHeight" for q in json.loads(raw)):clock[0]=18446744073709551616
            return result
        with mock.patch.object(o.time,"time",side_effect=lambda:clock[0]):
            e.capturing=True
            try:
                with self.assertRaisesRegex(o.Stop,"launcher_authorization_expired"):
                    e.host.call("PREPARE",{"captureLifetime":True,"packets":[{"id":e.pid,"wireBase64":e.host.prepared["wireBase64"]}]},_transport=expire,**e.host.auth())
            finally:e.capturing=False
        self.assertEqual(e.records(),before);self.assertFalse(e.sent)
        e.capture();e.signed();e.send_height=961
        with self.assertRaisesRegex(o.Stop,"fresh_lifetime_margin"):
            e.host.call("SEND",{"id":e.pid},_transport=e.transport,**e.host.auth())
        self.assertFalse(e.sent);self.assertFalse(any(r["kind"]=="INTENT" for r in e.records()))

    def test_replay_rejects_source_parent_auth_message_order_and_raw_tampering(self):
        e=self.env;e.capture();chain=e.records()
        for key,value in (("operatorSourceSha256","0"*64),("baseParentTail","0"*64),("manifestSha256","0"*64),("authorizationSha256","0"*64),("orderedPreparedSha256",[])):
            bad=copy.deepcopy(chain);bad[-1]["data"]["quote"][key]=value
            with self.assertRaises(ValueError):o.replay(e.host.j,bad)
        bad=copy.deepcopy(chain);bad[-1]["data"]["entries"][0]["lifetime"]["lastValidBlockHeight"]="999999"
        with self.assertRaises(ValueError):o.replay(e.host.j,bad)
        quote=chain[-1]["data"]["quote"];path=Path(quote["path"])/"rpc-02.response.bin";saved=path.read_bytes()
        try:
            path.write_bytes(saved+b' ')
            with self.assertRaisesRegex(o.Stop,"quote_artifact_hash"):o.replay(e.host.j,chain)
        finally:path.write_bytes(saved)
        self.assertEqual(e.host.call("INSPECT")["tail"],e.host.tail)

    def test_injected_quote_cannot_promote_or_change_authorization(self):
        e=self.env;a=json.loads(e.host.auth()["authorization_raw"])
        a.update(schema="piv1-launcher-release-authorization-v2",signerBackendSha256=o.SIGNER_BACKEND_SHA,signerProfileSha256="e"*64)
        raw=o.encode(a);auth={"authorization_raw":raw,"authorization_sha256":o.sha(raw)}
        e.capture(auth=auth);state=o.replay(e.host.j,e.records());self.assertEqual(state["transportMode"],"OFFLINE_INJECTED_TEST")
        changed=copy.deepcopy(a);changed["expiresUnixTimestamp"]="18446744073709551614";raw=o.encode(changed)
        with self.assertRaisesRegex(o.Stop,"quote_authorization_changed"):
            e.host.call("HANDOFF",[e.pid],authorization_raw=raw,authorization_sha256=o.sha(raw))
        before=e.records()
        with mock.patch.object(o,"invoke_signer",side_effect=AssertionError("must not invoke")):
            with self.assertRaisesRegex(o.Stop,"injected_history_cannot_sign_live"):
                e.host.call("SIGN",{"ids":[e.pid],"signerProfilePath":"/not-opened/profile.json"},live_sign=True,**auth)
        self.assertEqual(e.records(),before)
        e.signed(auth=auth)
        with self.assertRaisesRegex(o.Stop,"transport_mode_changed"):
            e.host.call("SEND",{"id":e.pid},live=True,**auth)
        self.assertFalse(e.sent)

    def test_sign_import_freshly_rechecks_quote_after_callback_and_holds_on_tamper(self):
        e=self.env;a=json.loads(e.host.auth()["authorization_raw"])
        a.update(schema="piv1-launcher-release-authorization-v2",signerBackendSha256=o.SIGNER_BACKEND_SHA,signerProfileSha256="e"*64)
        raw=o.encode(a);auth={"authorization_raw":raw,"authorization_sha256":o.sha(raw)}
        e.capture(auth=auth);quote=e.records()[-1]["data"]["quote"]
        path=Path(quote["path"])/"rpc-02.response.bin";saved=path.read_bytes();callbacks=[]
        data={"ids":[e.pid],"signerProfilePath":"/not-opened/profile.json"}
        def backend(payload):
            callbacks.append(payload["request"]["requestId"])
            self.assertEqual(e.records()[-1]["kind"],"SIGN_ATTEMPT")
            path.write_bytes(saved+b' ')
            return {"schema":"piv1-startup-signing-response-v1","requestId":payload["request"]["requestId"],"packets":[{"id":e.pid,"wireBase64":e.host.tx["signedWireBase64"]}]}
        try:
            with mock.patch.object(o,"check_signed",wraps=o.check_signed) as crypto:
                with self.assertRaisesRegex(o.Stop,"quote_artifact_hash"):
                    e.host.call("SIGN",data,live_sign=True,_signer=backend,**auth)
                # Crypto verification completed, then the mandatory precommit
                # replay freshly rejected changed historical raw evidence.
                crypto.assert_called_once()
        finally:path.write_bytes(saved)
        self.assertEqual(len(callbacks),1);self.assertEqual(e.records()[-1]["kind"],"SIGN_ATTEMPT")
        e.host.refresh_tail();result=e.host.call("INSPECT")
        self.assertEqual(result["status"],"SIGNER_ATTEMPT_UNRESOLVED_ORIGINAL_RESPONSE_ONLY")
        for action,request in (("SIGN",data),("PREPARE",[e.host.prepared]),("SEND",{"id":e.pid})):
            with self.assertRaisesRegex(o.Stop,"signer_attempt_unresolved"):
                e.host.call(action,request,live_sign=True,_signer=backend,**auth)
        self.assertEqual(len(callbacks),1);self.assertFalse(e.sent)

    def test_failed_capture_preserved_exclusive_retry_and_unknown_blocks_reads(self):
        e=self.env;e.fault="transport";before=e.records()
        with self.assertRaises(ValueError):e.capture()
        self.assertEqual(e.records(),before)
        root=o.journal_path(e.host.base).with_name(o.journal_path(e.host.base).name+".evidence")
        first=next(root.iterdir());saved={p.name:p.read_bytes() for p in first.iterdir()}
        self.assertIn("rpc-01.partial-response.bin",saved)
        e.fault=None;e.capture();self.assertEqual({p.name:p.read_bytes() for p in first.iterdir()},saved)
        self.assertEqual(len(list(root.iterdir())),2)
        self.assertEqual([r["kind"] for r in e.records()],["CREATE","PREPARE"])
        self.assertNotEqual(e.records()[-1]["data"]["quote"]["path"],str(first))
        e.signed();e.host.call("SEND",{"id":e.pid},_transport=e.transport,**e.host.auth());reads=len(e.queries)
        with self.assertRaisesRegex(o.Stop,"unresolved_intent_blocks_prepare"):e.capture()
        self.assertEqual(len(e.queries),reads)


if __name__=="__main__":unittest.main()
