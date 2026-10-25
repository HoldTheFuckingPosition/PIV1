"""Portable preparation/controller host tests; synthetic history/signatures.

Synthetic pin overrides are explicit and local to each test. Actual AU55/tail,
artifact and wire are checked separately by the root's once-only public CLI run.
Controller tests import pinned original primitives without old operate/replay;
crypto checking is an explicit test seam. No subprocess, network, keys or real
signing occurs here.
"""
import base64
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

HERE = Path(__file__).parent
spec = importlib.util.spec_from_file_location("phase10_exception", HERE/"startup_phase10_exception.py")
e = importlib.util.module_from_spec(spec); spec.loader.exec_module(e)


class Phase10ExceptionTests(unittest.TestCase):
    def setUp(self):
        self.oracle = json.loads((HERE/"startup_manifest.oracle.json").read_bytes())
        self.image = (bytes(range(256)) * ((e.ELF_BYTES+255)//256))[:e.ELF_BYTES]
        ref = self.oracle["writeTemplate"]
        prefix = base64.b64decode(ref["prefixBase64"])
        data = e.struct.pack("<IIQ", 1, e.OFFSET, e.LENGTH) + self.image[e.OFFSET:e.OFFSET+e.LENGTH]
        self.message = prefix + bytes([0xcc, 0x07]) + data
        old_message = self.message[:132] + bytes([7])*32 + self.message[164:]
        # Marker bytes do not constitute a signature/control proof.
        wire = bytes([1]) + bytes([8])*64 + old_message
        signature = e.b58(wire[1:65])
        self.manifest = {"parentTail": {"sequence": "11", "digest": "a"*64},
                         "startupProfile": {"elfBase64": base64.b64encode(self.image).decode()}}
        self.auth = {"manifestSha256": e.sha(e.encode(self.manifest)), "baseParentTail": self.manifest["parentTail"],
                     "operatorSourceSha256": e.PINS["startup_release_operator.py"], "expiresUnixTimestamp": "1792917973"}
        self.auth_raw = e.encode(self.auth)
        self.stop = {"phaseIndex": 10, "reason": "post_intent_lifetime_margin", "originalAuthorizationSha256": e.sha(self.auth_raw)}
        self.stop_raw = e.encode(self.stop)
        self.records = []
        def append(kind, data):
            body = {"schema": "piv1-startup-release-operator-v1", "sequence": str(len(self.records)),
                    "previousDigest": self.records[-1]["digest"] if self.records else None,
                    "lockIdentity": {"device": "1", "inode": "2", "ctimeNs": "3"}, "kind": kind, "data": data}
            self.records.append({**body, "digest": e.sha(e.encode(body))})
        append("CREATE", {"manifest": self.manifest})
        for i in range(11):
            pid = f"startup_{i:04}"
            prepared = {"id": pid, "feeQuoteLamports": "5000", "lifetime": {"recentBlockhash": e.b58(bytes([7])*32), "lastValidBlockHeight": "407927460"}}
            signed = {"id": pid, "wireBase64": base64.b64encode(wire).decode(), "wireSha256": e.sha(wire), "signatureHex": wire[1:65].hex(), "signature": signature}
            intent = {"id": pid, "feeLamports": "5000", "authorizationSha256": e.sha(self.auth_raw), **{k:signed[k] for k in ("wireSha256", "signature", "signatureHex")}}
            append("PREPARE_SIGN_ATTEMPT", {"entries": [prepared]})
            append("IMPORT_SEND_ATTEMPT_INTENT", {"import": {"entries": [signed]}, "intent": intent})
            if i < 10:
                append("SUBMITTED", {"id": pid}); append("ATTEMPT", {"id": pid})
                append("FINALIZED", {"id": pid, "feeLamports": "10000" if i == 0 else "5000"})
            else:
                append("ATTEMPT", {"id": pid}); append("UNKNOWN", {"id": pid})
        pins = {"TAIL": self.records[-1]["digest"], "MANIFEST_SHA": e.sha(e.encode(self.manifest)),
                "AUTH_SHA": e.sha(self.auth_raw), "STOP_SHA": e.sha(self.stop_raw), "WIRE_SHA": e.sha(wire),
                "SIGNATURE": signature, "ELF_SHA": e.sha(self.image)}
        patch = mock.patch.multiple(e, **pins); patch.start(); self.addCleanup(patch.stop)

    def result(self): return e.proposal(self.records, self.auth_raw, self.stop_raw, self.oracle)

    def repin_chain(self):
        previous = None
        for r in self.records:
            r["previousDigest"] = previous
            r["digest"] = e.sha(e.encode({k:v for k,v in r.items() if k != "digest"})); previous = r["digest"]
        e.TAIL = previous

    def test_concrete_inert_write_and_complete_buffer_hypotheses_preserve_unknown(self):
        before = copy.deepcopy(self.records)
        result = self.result(); write = result["singleWrite"]
        self.assertEqual(base64.b64decode(write["unsignedWireBase64"]), bytes([1])+bytes(64)+self.message)
        self.assertEqual(self.message[132:164], bytes(32))
        self.assertEqual(write["offsetBytes"], 8604); self.assertEqual(write["lengthBytes"], 956)
        self.assertEqual(write["reviewed"]["requiredSigners"], [self.oracle["roles"]["g0"]])
        self.assertIsNone(write["lifetime"]); self.assertIsNone(write["feeQuoteLamports"])
        header = e.struct.pack("<IB", 1, 1) + self.message[4:36]
        for branch in result["fullBufferPredicates"]["admissiblePrestateBranches"]:
            n = branch["writtenPrefixBytes"]; raw = header + self.image[:n] + bytes(e.ELF_BYTES-n)
            self.assertEqual(branch["accountDataBytes"], 894589); self.assertEqual(branch["accountDataSha256"], e.sha(raw))
        self.assertEqual(result["feePolicy"]["oldPossibleAdditionalLamports"], "5000")
        self.assertEqual(result["feePolicy"]["oldReservedLamports"], "60000")
        self.assertEqual(result["feePolicy"]["combinedReservationAfterOneNewIntentLamports"], "65000")
        self.assertEqual(result["feePolicy"]["conditionalWholeSequenceCapIfOtherQuotesUnchangedLamports"], "4820000")
        self.assertFalse(result["feePolicy"]["oldReservationReleased"])
        self.assertEqual(result["lineage"]["originalStatus"], "UNKNOWN_HOLD_PRESERVED")
        self.assertFalse(result["lineage"]["historyOrNoninclusionProved"]); self.assertFalse(result["liveReady"])
        self.assertIn("NEW_LIVE_GRANT_REQUIRED", result["remainingIntegrationBlocker"])
        self.assertEqual(self.records, before); self.assertEqual(self.result(), result)

    def test_exact_tail_and_original_auth_stop_pins_reject_substitution(self):
        with mock.patch.object(e, "TAIL", "0"*64):
            with self.assertRaisesRegex(e.Stop, "exact_original_unknown_tail"): self.result()
        for auth,stop in ((self.auth_raw+b" ",self.stop_raw),(self.auth_raw,self.stop_raw+b" ")):
            with self.assertRaisesRegex(e.Stop, "exact_authorization_and_stop"):
                e.proposal(self.records, auth, stop, self.oracle)
        self.records[52]["data"]["intent"]["feeLamports"] = "0"
        with self.assertRaisesRegex(e.Stop, "original_record_digest"): self.result()

    def test_terminal_or_released_fee_cannot_replace_unknown_even_under_synthetic_pin(self):
        self.records[54]["kind"] = "FINALIZED"; self.repin_chain()
        with self.assertRaisesRegex(e.Stop, "original_unknown_required"): self.result()

    def test_different_write_cannot_be_selected_even_under_synthetic_wire_pin(self):
        signed = self.records[52]["data"]["import"]["entries"][0]
        wire = bytearray(base64.b64decode(signed["wireBase64"]))
        wire[264:268] = e.struct.pack("<I", e.OFFSET + e.LENGTH)
        signed["wireBase64"] = base64.b64encode(wire).decode()
        e.WIRE_SHA = signed["wireSha256"] = self.records[52]["data"]["intent"]["wireSha256"] = e.sha(wire)
        self.repin_chain()
        with self.assertRaisesRegex(e.Stop, "original_write_semantics"): self.result()

    def test_different_artifact_cannot_be_selected_even_under_synthetic_manifest_pin(self):
        self.manifest["startupProfile"]["elfBase64"] = base64.b64encode(bytes(e.ELF_BYTES)).decode()
        e.MANIFEST_SHA = e.sha(e.encode(self.manifest)); self.auth["manifestSha256"] = e.MANIFEST_SHA
        self.auth_raw = e.encode(self.auth); e.AUTH_SHA = e.sha(self.auth_raw)
        self.stop["originalAuthorizationSha256"] = e.AUTH_SHA; self.stop_raw = e.encode(self.stop); e.STOP_SHA = e.sha(self.stop_raw)
        self.records[52]["data"]["intent"]["authorizationSha256"] = e.AUTH_SHA; self.repin_chain()
        with self.assertRaisesRegex(e.Stop, "exact_selected_artifact"): self.result()

    def test_public_read_stability_ignores_atime_updates(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)/"public.json"; path.write_bytes(b"{}\n")
            original = path.stat()
            names = ("st_dev", "st_ino", "st_uid", "st_gid", "st_mode", "st_nlink", "st_size", "st_mtime_ns", "st_ctime_ns")
            after = SimpleNamespace(**{name:getattr(original, name) for name in names}, st_atime=original.st_atime+1)
            with mock.patch.object(e.os, "fstat", side_effect=[original, after]), mock.patch.object(e.os, "stat", return_value=after):
                self.assertEqual(e.read_public(path, 100), b"{}\n")

    def test_original_source_drift_stops_before_any_evidence_read(self):
        with mock.patch.dict(e.PINS, {"startup_release_operator.py": "0"*64}):
            with self.assertRaisesRegex(e.Stop, "unchanged_original_source_pins"):
                e.prepare("/unreached-records", "/unreached-auth", "/unreached-stop")

    def test_public_input_bounds_symlinks_canonical_chain_and_namespace(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); raw = root/"raw.json"; raw.write_bytes(b"{}\n")
            with self.assertRaisesRegex(e.Stop, "metadata_or_bound"): e.read_public(raw, 2)
            link = root/"link.json"; link.symlink_to(raw)
            with self.assertRaises(OSError): e.read_public(link, 100)
            with self.assertRaisesRegex(e.Stop, "not_canonical"): e.canonical(b'{ "x":1}\n')
            with self.assertRaisesRegex(e.Stop, "exact_55_record_namespace"): e.records_at(root)
        with self.assertRaisesRegex(e.Stop, "exact_55_records"): e.proposal(self.records[:-1], self.auth_raw, self.stop_raw, self.oracle)


class Phase10ControllerTests(unittest.TestCase):
    """Real new journal/controller with synthetic RPC and signature markers.

    The immutable original replay/operate and native backend are never called.
    The existing cryptographic checker is explicitly replaced: these tests make
    no Ed25519, signer-control, public-state or native-runtime claim.
    """
    result = Phase10ExceptionTests.result

    def setUp(self):
        Phase10ExceptionTests.setUp(self)
        self.tmp = tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.journal = str(Path(self.tmp.name)/("au-phase10-"+e.TAIL+".exception"))
        self.p = self.result(); self.p["preparerSourceSha256"] = e.sha((HERE/"startup_phase10_exception.py").read_bytes())
        self.value = {"schema": "piv1-au-phase10-package-v1", "evidence": ["synthetic"]*3,
                      "proposal": self.p, "bridgeSourceSha256": e.sha((HERE/e.BRIDGE).read_bytes())}
        checked = mock.patch.object(e, "checked_package", side_effect=lambda value: value["proposal"]); checked.start(); self.addCleanup(checked.stop)
        self.keys = sorted(self.oracle["roles"][name] for name in ("g0", "h2", "pivProgram", "programData", "loaderBuffer", "loaderProgram", "computeBudget", "rent", "clock", "instructions"))
        self.raw_baseline = {}
        for key in self.keys:
            before = self.oracle["baseline"][key]
            if not before["exists"]: self.raw_baseline[key] = None; continue
            data = b"" if key in (self.oracle["roles"]["g0"], self.oracle["roles"]["h2"] ) else b"synthetic:"+key.encode()
            self.raw_baseline[key] = {"owner": before["owner"], "executable": before["executable"], "lamports": int(before["lamports"]),
                                      "data": [base64.b64encode(data).decode(), "base64"]}
            self.oracle["baseline"][key] = {**before, "dataBytes": len(data), "dataSha256": e.sha(data)}
        original_core = e.successor_core
        self.checks = 0
        def core():
            result = original_core()
            result.startup_manifest = lambda: SimpleNamespace(oracle=lambda: self.oracle, required_keys=lambda phase: list(self.keys))
            def checker(*args): self.checks += 1; return {"reason": "SYNTHETIC_CRYPTO_SEAM_ONLY"}
            result.check_signed = checker
            result.operate = mock.Mock(side_effect=AssertionError("historical operate forbidden"))
            return result
        patched = mock.patch.object(e, "successor_core", side_effect=core); patched.start(); self.addCleanup(patched.stop)
        self.core = core(); self.collector = self.core.helpers()[1]
        self.debit = 55000; self.ctx = 500000000; self.sends = 0; self.signs = 0
        self.recovered = False; self.send_unknown = False; self.fail_final_lifetime = False; self.callback_drift = False; self.recovery_interrupt = False
        self.make_authorization()

    def make_authorization(self):
        package_hash = e.sha(e.encode(self.value)); expiry = str(int(e.time.time())+3600)
        namespace = "piv1-testnet-startup-release-"+package_hash
        inner = {"schema": "piv1-launcher-release-authorization-v2", "manifestSha256": package_hash, "baseParentTail": {"sequence": "54", "digest": e.TAIL},
                 "operatorSourceSha256": e.PINS["startup_release_operator.py"], "expiresUnixTimestamp": expiry, "scope": "EXACT_BOUNDED_TESTNET_RELEASE",
                 "feeBudgetNamespace": namespace, "signerBackendSha256": e.PINS["startup_signer_backend.cjs"], "signerProfileSha256": "b"*64}
        inner_raw = e.encode(inner)
        self.authorization = {"schema": e.PRIVATE_AUTH_SCHEMA if "rpcTransport" in self.value else e.AUTH_SCHEMA, "scope": "EXACT_SINGLE_AU_PHASE10_EXCEPTION", "journalPath": self.journal,
            "packageSha256": package_hash, "originalTail": e.TAIL,
            "sourcePins": e.source_pins(self.value),
            "expiresUnixTimestamp": expiry, "signerProfileSha256": "b"*64, "feeBudgetNamespace": namespace,
            "backendAuthorizationRawBase64": base64.b64encode(inner_raw).decode(), "backendAuthorizationSha256": e.sha(inner_raw),
            "writeSliceSha256": self.p["singleWrite"]["sliceSha256"]}
        if "rpcTransport" in self.value: self.authorization["rpcTransport"] = self.value["rpcTransport"]
        self.authorization_raw = e.encode(self.authorization); self.authorization_pin = e.sha(self.authorization_raw)

    def snapshot(self):
        accounts = copy.deepcopy(self.raw_baseline); r = self.oracle["roles"]
        g, h = e.physical_balances(self.oracle, self.debit, 5000 if self.recovered else 0)
        accounts[r["g0"]]["lamports"] = g; accounts[r["h2"]]["lamports"] = h
        n = e.OFFSET+e.LENGTH if self.recovered else e.OFFSET
        raw = e.struct.pack("<IB", 1, 1)+base64.b64decode(self.oracle["writeTemplate"]["prefixBase64"])[4:36]+self.image[:n]+bytes(e.ELF_BYTES-n)
        accounts[r["loaderBuffer"]] = {"owner": e.LOADER, "executable": False, "lamports": int(self.oracle["rents"]["894589"]), "data": [base64.b64encode(raw).decode(), "base64"]}
        clock = e.struct.pack("<QqQQq", self.ctx, 0, 1, 1, 1792920000+self.ctx-500000000)
        accounts[r["clock"]] = {"owner": self.core.state_rules().SYSVAR, "executable": False, "lamports": 1, "data": [base64.b64encode(clock).decode(), "base64"]}
        return accounts

    def transport(self, raw):
        queries = json.loads(raw); self.ctx += 5
        if self.recovery_interrupt and any(q["method"] == "getSignatureStatuses" and q["params"][0] != [e.SIGNATURE] for q in queries):
            self.recovery_interrupt = False; raise ValueError("synthetic interrupted recovery")
        result = []
        for q in queries:
            method, args = q["method"], q["params"]; context = {"slot": self.ctx}
            if "rpcTransport" in self.value: context["apiVersion"] = self.oracle["version"]["solana-core"]
            if method == "getGenesisHash": value = e.GENESIS
            elif method == "getVersion": value = self.oracle["version"]
            elif method == "getLatestBlockhash": value = {"context": context, "value": {"blockhash": e.b58(bytes([11])*32), "lastValidBlockHeight": 407927800}}
            elif method == "getBlockHeight": value = 407927761 if self.fail_final_lifetime and len(queries) == 2 else 407927500
            elif method == "isBlockhashValid": value = {"context": context, "value": args[0] != self.p["lineage"]["originalLifetime"]["recentBlockhash"]}
            elif method == "getFeeForMessage": value = {"context": context, "value": 5000}
            elif method == "getMinimumBalanceForRentExemption": value = int(self.oracle["rents"][str(args[0])])
            elif method == "getMultipleAccounts":
                snapshot = self.snapshot(); value = {"context": context, "value": [snapshot[key] for key in args[0]]}
            elif method == "getSignatureStatuses":
                status = {"slot": self.ctx-1, "confirmationStatus": "finalized", "err": None} if self.recovered and args[0] != [e.SIGNATURE] else None
                value = {"context": context, "value": [status]}
            elif method == "getTransaction":
                value = {"slot": self.ctx-1, "version": "legacy", "transaction": [self.signed_wire, "base64"], "meta": {"fee": 5000, "err": None}} if self.recovered and args[0] != e.SIGNATURE else None
            elif method == "sendTransaction":
                self.sends += 1
                self.assertEqual(args[1]["maxRetries"], 0); self.assertFalse(args[1]["skipPreflight"])
                self.assertEqual(args[1]["preflightCommitment"], "finalized")
                if self.send_unknown: raise ValueError("synthetic uncertain send")
                value = self.signature
            else: raise AssertionError(method)
            result.append({"jsonrpc": "2.0", "id": q["id"], "result": value})
        return 200, {}, e.encode(result)

    def signer(self, payload):
        self.signs += 1; request = payload["request"]
        wire = bytearray(base64.b64decode(payload["packets"][0]["wireBase64"])); wire[1:65] = bytes([9])*64
        self.signed_wire = base64.b64encode(wire).decode(); self.signature = e.b58(wire[1:65])
        nested = {**request, "schema": "piv1-startup-signing-request-v1", "operatorSourceSha256": e.PINS["startup_release_operator.py"], "authorizationSha256": self.authorization["backendAuthorizationSha256"]}
        del nested["requestId"]; nested["requestId"] = e.sha(e.encode(nested))
        if self.callback_drift: self.debit = 60000
        return {"schema": "piv1-au-phase10-signing-response-v1", "requestId": request["requestId"], "backendRequest": nested,
                "backendResponse": {"schema": "piv1-startup-signing-response-v1", "requestId": nested["requestId"], "packets": [{"id": e.PHASE, "wireBase64": self.signed_wire}]}}

    def run_action(self, action, tail=None):
        return e.successor(action, self.value, self.journal, authorization_raw=self.authorization_raw,
                           authorization_sha256=self.authorization_pin, expected_tail=tail, live=True, live_sign=True,
                           signer_profile_path="/synthetic-unread-profile.json", _transport=self.transport, _signer=self.signer)

    def tail(self):
        paths = sorted(Path(self.journal).glob("*.json")); return json.loads(paths[-1].read_bytes())["digest"]

    def test_history_singleton_null_preserves_strict_rpc_and_retained_receipt_binding(self):
        original_transport = self.transport; original_core_factory = e.successor_core
        history_batches = []; transport_batches = []; completed_writes = []
        def observed_core():
            core = original_core_factory(); original_helpers = core.helpers
            def helpers():
                modules = original_helpers(); original_write = modules[1].Store.write
                def write(store, name, raw):
                    result = original_write(store, name, raw)
                    completed_writes.append((store.path.name, name))  # Real file+directory fsyncs completed.
                    return result
                modules[1].Store.write = write
                return modules
            core.helpers = helpers
            return core
        def batch_sensitive(raw):
            queries = json.loads(raw)
            methods = [query["method"] for query in queries]
            if "getLatestBlockhash" in methods:
                self.assertIn(("00000001-history", "rpc-01.response.bin"), completed_writes)
                self.assertIn(("00000001-history", "rpc-01.http.json"), completed_writes)
                self.assertIn(("00000001-quote", "prequote-history.json"), completed_writes)
            if len(transport_batches) == 2:
                self.assertIn(("00000001-quote", "rpc-01.response.bin"), completed_writes)
                self.assertIn(("00000001-quote", "rpc-01.http.json"), completed_writes)
            transport_batches.append(methods)
            status, headers, body = original_transport(raw)
            history = [q for q in queries if q["method"] == "getTransaction" and q["params"][0] == e.SIGNATURE]
            if history:
                history_batches.append(queries)
                if len(queries) > 1:
                    # Exact demonstrated failure shape: the history reply has
                    # neither result nor error; every companion remains valid.
                    replies = json.loads(body)
                    del next(reply for reply in replies if reply["id"] == history[0]["id"])["result"]
                    body = e.encode(replies)
            return status, headers, body
        with mock.patch.object(self, "transport", side_effect=batch_sensitive), \
             mock.patch.object(e, "successor_core", side_effect=observed_core):
            created = self.run_action("CREATE")
            sent = self.run_action("CAPTURE_SIGN_SEND", created["tail"])
        self.assertEqual(sent["status"], "SUBMITTED")
        self.assertEqual(len(history_batches), 4)
        self.assertTrue(all(len(batch) == 1 and batch[0]["id"] == 1 for batch in history_batches))
        self.assertTrue(all(batch[0]["params"] == [e.SIGNATURE, {"encoding": "base64", "commitment": "finalized",
            "maxSupportedTransactionVersion": 0}] for batch in history_batches))
        self.assertEqual(self.signs, 1); self.assertEqual(self.sends, 1)
        self.assertEqual(transport_batches[:2], [["getTransaction"], ["getGenesisHash", "getLatestBlockhash"]])
        self.assertEqual([len(batch) for batch in transport_batches[2:4]], [5, 4])
        self.assertNotIn("getTransaction", transport_batches[2]+transport_batches[3])
        self.assertEqual([len(batch) for batch in transport_batches[4:13]], [5, 1, 4]*3)
        records = [json.loads(path.read_bytes()) for path in sorted(Path(self.journal).glob("*.json"))]
        attempt = records[1]["data"]; receipt = attempt["guard"]; directory = Path(receipt["path"])
        self.assertEqual(set(receipt["files"]), {f"rpc-{index:02}{suffix}" for index in (1, 2)
            for suffix in (".request.json", ".started.json", ".http.json", ".response.bin")})
        history_directory = Path(attempt["history"]["path"])
        self.assertEqual(e.retained_rpc(self.core, self.collector, history_directory, [[e.history_call()]]), [None])
        quote_directory = Path(attempt["quote"]["path"])
        link = quote_directory/"prequote-history.json"; link_raw = link.read_bytes()
        self.assertEqual(link_raw, e.encode(attempt["history"]))
        self.assertEqual(attempt["quote"]["files"][link.name], e.sha(link_raw))
        quote_replies = json.loads((Path(attempt["quote"]["path"])/"rpc-01.response.bin").read_bytes())
        quoted_floor = next(reply for reply in quote_replies if reply["id"] == 2)["result"]["context"]["slot"]
        calls = e.guard_calls(self.core, self.p, attempt["prepared"], quoted_floor)
        batches = e.guard_batches(calls, history_before_quote=True)
        self.assertEqual([len(batch) for batch in batches], [5, 4])
        self.assertEqual([call for batch in batches for call in batch], calls[:5]+calls[6:])
        for index, batch in enumerate(batches, 1):
            self.assertEqual(json.loads((directory/f"rpc-{index:02}.request.json").read_bytes()),
                [{"jsonrpc": "2.0", "id": i+1, "method": method, "params": args}
                 for i, (method, args) in enumerate(batch)])
        # Replay binds the separate history receipt and post-quote nine reads.
        self.assertEqual(len(e.retained_rpc(self.core, self.collector, directory, batches)), 9)
        self.assertEqual(self.run_action("INSPECT", sent["tail"])["status"], "SUBMITTED")
        # Rehashing an altered quote link cannot adopt a different history receipt.
        bad_history = copy.deepcopy(attempt["history"]); bad_history["path"] += "-substitute"
        link.write_bytes(e.encode(bad_history)); altered = copy.deepcopy(records)
        altered[1]["data"]["quote"] = e.quote_rpc_receipt(quote_directory)
        with self.assertRaisesRegex(e.Stop, "successor_history_before_quote_binding"):
            e.successor_replay(self.core, self.collector, self.core.helpers()[3], altered, self.journal)
        link.write_bytes(link_raw)
        # The unchanged Rpc and raw replay must still reject the old malformed
        # five-call layout; isolating history never converts an omission to null.
        malformed = self.collector.Store(Path(self.tmp.name)/"legacy-malformed-history-batch")
        try:
            rpc = self.core.Rpc(self.collector, self.core.helpers()[3], malformed, batch_sensitive)
            with self.assertRaisesRegex(ValueError, "rpc_reply"):
                rpc.query(calls[5:])
            with self.assertRaisesRegex(e.Stop, "successor_raw_result_error"):
                e.retained_rpc(self.core, self.collector, malformed.path, [calls[5:]])
        finally: malformed.close()
        # Even a still-null semantically valid reply is bound to exact bytes.
        response = history_directory/"rpc-01.response.bin"
        response.write_bytes(response.read_bytes()+b" ")
        with self.assertRaisesRegex(e.Stop, "successor_raw_receipt_changed"):
            self.run_action("INSPECT", sent["tail"])

    def test_history_singleton_missing_error_or_nonnull_stops_before_sign_attempt(self):
        original_transport = self.transport
        for kind, reason in (("missing", "rpc_reply"), ("error", "rpc_reply"),
                             ("nonnull", "successor_original_history_requires_review")):
            with self.subTest(kind=kind):
                parent = Path(self.tmp.name)/kind; parent.mkdir()
                self.journal = str(parent/("au-phase10-"+e.TAIL+".exception")); self.make_authorization()
                created = self.run_action("CREATE")
                def refused_history(raw):
                    queries = json.loads(raw)
                    status, headers, body = original_transport(raw)
                    if any(q["method"] == "getTransaction" and q["params"][0] == e.SIGNATURE for q in queries):
                        self.assertEqual(len(queries), 1)
                        replies = json.loads(body); reply = replies[0]
                        if kind == "missing": del reply["result"]
                        elif kind == "error":
                            del reply["result"]; reply["error"] = {"code": -32000, "message": "synthetic history refusal"}
                        else: reply["result"] = {"slot": self.ctx}
                        body = e.encode(replies)
                    return status, headers, body
                with mock.patch.object(self, "transport", side_effect=refused_history):
                    with self.assertRaisesRegex(ValueError, reason):
                        self.run_action("CAPTURE_SIGN_SEND", created["tail"])
                self.assertEqual(self.signs, 0); self.assertEqual(self.sends, 0)
                records = [json.loads(path.read_bytes()) for path in sorted(Path(self.journal).glob("*.json"))]
                self.assertEqual([record["kind"] for record in records], ["CREATE"])
                history_response = Path(self.journal+".evidence")/"00000001-history"/"rpc-01.response.bin"
                self.assertTrue(history_response.is_file())
                self.assertFalse((history_response.parent.parent/"00000001-quote").exists())
                self.assertFalse((history_response.parent.parent/"00000001-presign").exists())

    def test_authorization_expiring_during_first_history_refuses_before_quote(self):
        original_transport = self.transport; now = [int(e.time.time())]; calls = []
        def expired_after_history(raw):
            queries = json.loads(raw); calls.extend(queries)
            response = original_transport(raw)
            now[0] = int(self.authorization["expiresUnixTimestamp"])
            return response
        created = self.run_action("CREATE")
        with mock.patch.object(e.time, "time", side_effect=lambda: now[0]), \
             mock.patch.object(self, "transport", side_effect=expired_after_history):
            with self.assertRaisesRegex(e.Stop, "successor_authorization_expired"):
                self.run_action("CAPTURE_SIGN_SEND", created["tail"])
        self.assertEqual([query["method"] for query in calls], ["getTransaction"])
        evidence = Path(self.journal+".evidence")
        self.assertTrue((evidence/"00000001-history"/"rpc-01.response.bin").is_file())
        self.assertFalse((evidence/"00000001-quote").exists())
        self.assertFalse((evidence/"00000001-presign").exists())
        self.assertEqual(self.signs, 0); self.assertEqual(self.sends, 0)
        records = [json.loads(path.read_bytes()) for path in sorted(Path(self.journal).glob("*.json"))]
        self.assertEqual([record["kind"] for record in records], ["CREATE"])

    def test_controller_single_send_exact_branches_recovery_and_retained_unknown(self):
        for debit in (55000, 60000):
            with self.subTest(debit=debit):
                if debit == 60000:
                    self.tmp.cleanup(); self.tmp = tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
                    self.journal = str(Path(self.tmp.name)/("au-phase10-"+e.TAIL+".exception")); self.make_authorization()
                self.debit = debit; self.recovered = False; self.send_unknown = debit == 60000
                created = self.run_action("CREATE"); sent = self.run_action("CAPTURE_SIGN_SEND", created["tail"])
                self.assertEqual(sent["status"], "UNKNOWN" if self.send_unknown else "SUBMITTED")
                self.assertEqual(sent["combinedReservedLamports"], "65000"); self.assertEqual(sent["oldPossibleFeeLamports"], "5000")
                self.recovered = True; recovered = self.run_action("RECOVER", sent["tail"])
                self.assertEqual(recovered["status"], "FINALIZED"); self.assertEqual(recovered["combinedKnownActualLamports"], "60000")
                self.assertEqual(recovered["downstreamCanonicalG0AdjustmentLamports"], str(debit-55000))
                self.assertEqual(recovered["originalStatus"], "UNKNOWN_HOLD_PRESERVED"); self.assertFalse(recovered["dependentProgressionAllowed"])
                with self.assertRaisesRegex(e.Stop, "attempt_consumed"): self.run_action("CAPTURE_SIGN_SEND", recovered["tail"])
        self.assertEqual(self.sends, 2); self.assertEqual(self.signs, 2); self.assertGreater(self.checks, 0)

    def test_final_lifetime_refusal_keeps_intent_and_original_only_unknown_recovery(self):
        created = self.run_action("CREATE"); self.fail_final_lifetime = True
        with self.assertRaisesRegex(e.Stop, "post_intent_lifetime_margin"): self.run_action("CAPTURE_SIGN_SEND", created["tail"])
        inspected = self.run_action("INSPECT", self.tail())
        self.assertEqual(inspected["status"], "INTENT"); self.assertEqual(inspected["combinedReservedLamports"], "65000")
        self.assertEqual(self.sends, 0); self.assertEqual(self.signs, 1)
        recovered = self.run_action("RECOVER", inspected["tail"])
        self.assertEqual(recovered["status"], "UNKNOWN"); self.assertEqual(self.sends, 0)

    def test_postcallback_exact_balance_drift_retains_response_and_recovery_without_send(self):
        created = self.run_action("CREATE"); self.callback_drift = True
        with self.assertRaisesRegex(e.Stop, "exact_g0_branch"): self.run_action("CAPTURE_SIGN_SEND", created["tail"])
        inspected = self.run_action("INSPECT", self.tail())
        self.assertEqual(inspected["status"], "SIGN_ATTEMPT"); self.assertEqual(inspected["combinedReservedLamports"], "65000")
        self.assertEqual(self.sends, 0); self.assertEqual(self.signs, 1)
        self.debit = 55000
        recovered = self.run_action("RECOVER", inspected["tail"])
        self.assertEqual(recovered["status"], "UNKNOWN"); self.assertEqual(self.sends, 0); self.assertEqual(self.signs, 1)
        self.assertTrue(any(json.loads(path.read_bytes())["kind"] == "ORIGINAL_RESPONSE_HOLD" for path in Path(self.journal).glob("*.json")))

    def test_late_callback_or_response_fsync_retains_complete_response_for_original_only_recovery(self):
        original_signer = self.signer; original_core_factory = e.successor_core
        for label, callback_seconds, fsync_seconds in (("late-callback", 6.001, 0), ("late-fsync", 5.75, 0.251)):
            with self.subTest(boundary=label):
                parent = Path(self.tmp.name)/label; parent.mkdir()
                self.journal = str(parent/("au-phase10-"+e.TAIL+".exception")); self.make_authorization()
                self.signs = self.sends = 0
                created = self.run_action("CREATE"); now = [100.0]; retained = {}
                def timed_signer(payload):
                    response = original_signer(payload); retained["raw"] = e.encode(response)
                    now[0] += callback_seconds
                    return response
                def timed_core():
                    core = original_core_factory(); original_helpers = core.helpers
                    def helpers():
                        modules = original_helpers(); original_write = modules[1].Store.write
                        def write(store, name, raw):
                            result = original_write(store, name, raw)
                            if name == "signer-response.json":
                                # Advance only AFTER the real file+directory
                                # fsyncs, so the cap must cover durable R too.
                                retained["fsynced"] = True; now[0] += fsync_seconds
                            return result
                        modules[1].Store.write = write
                        return modules
                    core.helpers = helpers
                    return core
                with mock.patch.object(e.time, "monotonic", side_effect=lambda: now[0]), \
                     mock.patch.object(e, "successor_core", side_effect=timed_core), \
                     mock.patch.object(self, "signer", side_effect=timed_signer):
                    with self.assertRaisesRegex(e.Stop, "successor_signer_response_runtime"):
                        self.run_action("CAPTURE_SIGN_SEND", created["tail"])
                response = Path(self.journal+".evidence")/"00000001-response"/"signer-response.json"
                self.assertTrue(retained["fsynced"]); self.assertEqual(response.read_bytes(), retained["raw"])
                self.assertFalse((response.parent.parent/"00000002-postsign").exists())
                records = [json.loads(path.read_bytes()) for path in sorted(Path(self.journal).glob("*.json"))]
                self.assertEqual([record["kind"] for record in records], ["CREATE", "SIGN_ATTEMPT"])
                inspected = self.run_action("INSPECT", self.tail())
                self.assertEqual(inspected["status"], "SIGN_ATTEMPT")
                with self.assertRaisesRegex(e.Stop, "successor_attempt_consumed_no_retry"):
                    self.run_action("CAPTURE_SIGN_SEND", inspected["tail"])
                original_transport = self.transport; recovery_calls = []
                def recovery_transport(raw):
                    recovery_calls.extend(json.loads(raw)); return original_transport(raw)
                with mock.patch.object(self, "transport", side_effect=recovery_transport):
                    recovered = self.run_action("RECOVER", inspected["tail"])
                self.assertEqual(recovered["status"], "UNKNOWN")
                self.assertEqual(self.signs, 1); self.assertEqual(self.sends, 0)
                self.assertEqual(response.read_bytes(), retained["raw"])
                records = [json.loads(path.read_bytes()) for path in sorted(Path(self.journal).glob("*.json"))]
                self.assertEqual([record["kind"] for record in records],
                    ["CREATE", "SIGN_ATTEMPT", "ORIGINAL_RESPONSE_HOLD", "RECOVER_ATTEMPT", "UNKNOWN"])
                self.assertEqual(records[2]["data"]["responseSha256"], e.sha(retained["raw"]))
                self.assertEqual(records[2]["data"]["signed"]["wireBase64"], self.signed_wire)
                self.assertTrue(recovery_calls)
                for query in recovery_calls:
                    self.assertNotEqual(query["method"], "sendTransaction")
                    if query["method"] == "getSignatureStatuses": self.assertEqual(query["params"][0], [self.signature])
                    if query["method"] == "getTransaction": self.assertEqual(query["params"][0], self.signature)

    def test_postcapture_history_budget_retains_raw_before_private_refusal_and_parser_error(self):
        original_core_factory = e.successor_core; original_transport = self.transport
        for malformed in (False, True):
            with self.subTest(malformed=malformed):
                parent = Path(self.tmp.name)/("malformed-history" if malformed else "late-history"); parent.mkdir()
                self.journal = str(parent/("au-phase10-"+e.TAIL+".exception")); self.make_authorization()
                self.signs = self.sends = 0
                created = self.run_action("CREATE"); now = [100.0]; completed = []; history_count = [0]
                def transport(raw):
                    status, headers, body = original_transport(raw)
                    if [query["method"] for query in json.loads(raw)] == ["getTransaction"]:
                        history_count[0] += 1
                        if malformed and history_count[0] == 2:
                            replies = json.loads(body); del replies[0]["result"]; body = e.encode(replies)
                    return status, headers, body
                def timed_core():
                    core = original_core_factory(); original_helpers = core.helpers
                    def helpers():
                        modules = original_helpers(); original_write = modules[1].Store.write
                        def write(store, name, raw):
                            result = original_write(store, name, raw)
                            if store.path.name == "00000002-post-sign-attempt" and name == "rpc-02.http.json":
                                completed.append(store.path); now[0] += 3.001  # AFTER real raw+HTTP file/directory fsync.
                            return result
                        modules[1].Store.write = write
                        return modules
                    core.helpers = helpers
                    return core
                reason = "rpc_reply" if malformed else "successor_postcapture_history_runtime"
                with mock.patch.object(e.time, "monotonic", side_effect=lambda: now[0]), \
                     mock.patch.object(e, "successor_core", side_effect=timed_core), \
                     mock.patch.object(self, "transport", side_effect=transport):
                    with self.assertRaisesRegex(ValueError, reason):
                        self.run_action("CAPTURE_SIGN_SEND", created["tail"])
                self.assertEqual(len(completed), 1)
                directory = completed[0]
                self.assertEqual(json.loads((directory/"rpc-02.http.json").read_bytes())["status"], 200)
                reply = json.loads((directory/"rpc-02.response.bin").read_bytes())[0]
                if malformed: self.assertNotIn("result", reply)
                else: self.assertIsNone(reply["result"])
                self.assertFalse((directory/"rpc-03.request.json").exists())
                self.assertFalse((directory.parent/"00000001-response"/"signer-response.json").exists())
                self.assertEqual(self.signs, 0); self.assertEqual(self.sends, 0)
                records = [json.loads(path.read_bytes()) for path in sorted(Path(self.journal).glob("*.json"))]
                self.assertEqual([record["kind"] for record in records], ["CREATE", "SIGN_ATTEMPT"])

    def test_postcapture_history_budget_is_cumulative_and_preserves_original_only_holds(self):
        original_core_factory = e.successor_core
        cases = (("accepted-boundary", (1.0, 1.0, 1.0), "SUBMITTED"),
                 ("late-after-response", (1.0, 2.001, 1.0), "SIGN_ATTEMPT"),
                 ("late-after-intent", (1.0, 1.0, 1.001), "INTENT"))
        for label, durations, expected in cases:
            with self.subTest(boundary=label):
                parent = Path(self.tmp.name)/label; parent.mkdir()
                self.journal = str(parent/("au-phase10-"+e.TAIL+".exception")); self.make_authorization()
                self.signs = self.sends = 0
                created = self.run_action("CREATE"); now = [100.0]; completed = []; prequote = []
                costs = dict(zip(("post-sign-attempt", "postsign", "predispatch"), durations))
                def timed_core():
                    core = original_core_factory(); original_helpers = core.helpers
                    def helpers():
                        modules = original_helpers(); original_write = modules[1].Store.write
                        def write(store, name, raw):
                            result = original_write(store, name, raw)
                            stage = store.path.name.split("-", 1)[1]
                            if stage == "history" and name == "rpc-01.http.json":
                                prequote.append(store.path); now[0] += 4.0  # Deliberately exceeds3; excluded before capture.
                            elif stage in costs and name == "rpc-02.http.json":
                                completed.append(store.path); now[0] += costs[stage]
                            return result
                        modules[1].Store.write = write
                        return modules
                    core.helpers = helpers
                    return core
                with mock.patch.object(e.time, "monotonic", side_effect=lambda: now[0]), \
                     mock.patch.object(e, "successor_core", side_effect=timed_core):
                    if expected == "SUBMITTED":
                        sent = self.run_action("CAPTURE_SIGN_SEND", created["tail"])
                        self.assertEqual(sent["status"], expected)
                    else:
                        with self.assertRaisesRegex(e.Stop, "successor_postcapture_history_runtime"):
                            self.run_action("CAPTURE_SIGN_SEND", created["tail"])
                self.assertEqual(len(prequote), 1)
                self.assertEqual(len(completed), 2 if expected == "SIGN_ATTEMPT" else 3)
                self.assertTrue(all(duration < 3 for duration in durations))
                for directory in completed:
                    self.assertEqual(json.loads((directory/"rpc-02.http.json").read_bytes())["status"], 200)
                    self.assertIsNone(json.loads((directory/"rpc-02.response.bin").read_bytes())[0]["result"])
                self.assertEqual(self.signs, 1)
                response = Path(self.journal+".evidence")/"00000001-response"/"signer-response.json"
                response_raw = response.read_bytes()
                self.assertEqual(json.loads(response_raw)["backendResponse"]["packets"][0]["wireBase64"], self.signed_wire)
                inspected = self.run_action("INSPECT", self.tail())
                self.assertEqual(inspected["status"], expected)
                if expected == "SUBMITTED":
                    self.assertEqual(self.sends, 1)
                    continue
                self.assertEqual(self.sends, 0)
                self.assertFalse((completed[-1]/"rpc-03.request.json").exists())
                with self.assertRaisesRegex(e.Stop, "successor_attempt_consumed_no_retry"):
                    self.run_action("CAPTURE_SIGN_SEND", inspected["tail"])
                original_transport = self.transport; recovery_calls = []
                def recovery_transport(raw):
                    recovery_calls.extend(json.loads(raw)); return original_transport(raw)
                with mock.patch.object(self, "transport", side_effect=recovery_transport):
                    recovered = self.run_action("RECOVER", inspected["tail"])
                self.assertEqual(recovered["status"], "UNKNOWN")
                self.assertEqual(response.read_bytes(), response_raw)
                self.assertEqual(self.signs, 1); self.assertEqual(self.sends, 0)
                records = [json.loads(path.read_bytes()) for path in sorted(Path(self.journal).glob("*.json"))]
                kinds = [record["kind"] for record in records]
                self.assertIn("ORIGINAL_RESPONSE_HOLD" if expected == "SIGN_ATTEMPT" else "INTENT", kinds)
                self.assertEqual(kinds[-2:], ["RECOVER_ATTEMPT", "UNKNOWN"])
                self.assertTrue(recovery_calls)
                for query in recovery_calls:
                    self.assertNotEqual(query["method"], "sendTransaction")
                    if query["method"] == "getSignatureStatuses": self.assertEqual(query["params"][0], [self.signature])
                    if query["method"] == "getTransaction": self.assertEqual(query["params"][0], self.signature)

    def test_explicit_recovery_after_interrupted_collection_and_grant_namespace_binding(self):
        created = self.run_action("CREATE"); sent = self.run_action("CAPTURE_SIGN_SEND", created["tail"])
        self.recovery_interrupt = True
        unknown = self.run_action("RECOVER", sent["tail"])
        self.assertEqual(unknown["status"], "UNKNOWN")
        self.recovered = True
        recovered = self.run_action("RECOVER", unknown["tail"])
        self.assertEqual(recovered["status"], "FINALIZED"); self.assertEqual(self.sends, 1); self.assertEqual(self.signs, 1)
        other = str(Path(self.tmp.name)/"different.exception")
        with self.assertRaisesRegex(e.Stop, "exact_successor_namespace"):
            e.successor_authorization(self.value, self.authorization_raw, self.authorization_pin, other)
        with self.assertRaisesRegex(e.Stop, "successor_authorization_binding"):
            e.successor_authorization(self.value, self.authorization_raw, self.authorization_pin, str(Path(self.tmp.name)/"another"/Path(self.journal).name))

    def private_package(self):
        self.value["schema"] = "piv1-au-phase10-package-v2"
        self.value["rpcTransport"] = {"schema": "piv1-phase10-private-rpc-binding-v1", "mode": "QUICKNODE_TESTNET_SAME_SLOT_FIVE_PLUS_FIVE",
                                      "sourceSha256": e.sha((HERE/e.RPC_SOURCE).read_bytes()), "endpointCredentialSha256": e.sha(b"SYNTHETIC_UNREAD_ENDPOINT_NO_NATIVE_AUTHORITY\n")}
        self.make_authorization()

    def test_private_split_forward_raw_replay_and_original_only_recovery(self):
        self.private_package(); created = self.run_action("CREATE")
        sent = self.run_action("CAPTURE_SIGN_SEND", created["tail"])
        records = [json.loads(p.read_bytes()) for p in sorted(Path(self.journal).glob("*.json"))]
        self.assertEqual(len(self.authorization["sourcePins"]), 7)
        self.assertEqual(records[1]["data"]["mode"], "OFFLINE_INJECTED_TEST")
        directory = Path(self.journal+".evidence")/"00000001-presign"
        raw = (directory/"rpc-02.response.bin").read_bytes(); replies = json.loads(raw)
        requests = json.loads((directory/"rpc-02.request.json").read_bytes())
        halves = [q["params"][0] for q in requests if q["method"] == "getMultipleAccounts"]
        self.assertEqual(halves, [self.keys[:5], self.keys[5:]])
        self.assertEqual(len(replies), 5); self.assertFalse(any("schema" in r["result"] for r in replies if isinstance(r["result"], dict)))
        # Use the actual quoted floor bound in the retained presign request.
        calls = [[(q["method"],q["params"]) for q in json.loads((directory/f"rpc-{i:02}.request.json").read_bytes())] for i in (1,2)]
        calls[1][2:4] = [("getMultipleAccounts", [self.keys, requests[2]["params"][1]])]
        replayed = e.retained_rpc(self.core, self.collector, directory, calls, value=self.value, mode="OFFLINE_INJECTED_TEST")
        split = [r for r in replayed if isinstance(r,dict) and r.get("schema") == "DERIVED_SPLIT_SNAPSHOT"]
        self.assertEqual(len(split), 1); self.assertEqual(split[0]["sharedRawResponseSha256"], e.sha(raw))
        self.assertEqual(len(split[0]["value"]), 10); self.assertFalse(split[0]["nativeCollectorProvenance"])
        (directory/"rpc-02.response.bin").write_bytes(raw+b" ")
        with self.assertRaisesRegex(e.Stop, "raw_receipt_changed"): self.run_action("INSPECT", sent["tail"])
        (directory/"rpc-02.response.bin").write_bytes(raw)
        request_raw = (directory/"rpc-02.request.json").read_bytes(); altered = copy.deepcopy(requests)
        altered[2]["params"][0][0] = self.keys[-1]; (directory/"rpc-02.request.json").write_bytes(e.encode(altered))
        with self.assertRaisesRegex(ValueError, "retained_request"):
            e.retained_rpc(self.core, self.collector, directory, calls, value=self.value, mode="OFFLINE_INJECTED_TEST")
        (directory/"rpc-02.request.json").write_bytes(request_raw)
        self.recovered = True; recovery_calls = []; transport = self.transport
        def observed(request):
            recovery_calls.extend(json.loads(request)); return transport(request)
        with mock.patch.object(self, "transport", side_effect=observed): recovered = self.run_action("RECOVER", sent["tail"])
        self.assertEqual(recovered["status"], "FINALIZED"); self.assertEqual(recovered["oldPossibleFeeLamports"], "5000")
        self.assertTrue(all(q["method"] != "sendTransaction" for q in recovery_calls))
        self.assertEqual([q["params"][0] for q in recovery_calls if q["method"] == "getTransaction"], [self.signature])
        collection = next(Path(self.journal+".evidence").glob("*-recovery/collection"))
        manifest_raw = (collection/"manifest.json").read_bytes(); manifest = json.loads(manifest_raw)
        self.assertEqual(manifest["schema"], "piv1-phase10-private-split-recovery-v1")
        self.assertEqual(manifest["transportMode"], "OFFLINE_INJECTED_TEST"); self.assertFalse(manifest["nativeCollectorProvenance"])
        self.assertEqual(len(json.loads((collection/"rpc-03.response.bin").read_bytes())), 2)
        state = e.successor_replay(self.core, self.collector, self.core.helpers()[3], [json.loads(p.read_bytes()) for p in sorted(Path(self.journal).glob("*.json"))], self.journal)
        # Direct retained parsing isolates the route binding; this is not a new journal-authentication claim.
        for change in (lambda r:r.update(transportMode=self.value["rpcTransport"]["mode"]),
                       lambda r:r["rpcTransport"].update(endpointCredentialSha256="f"*64)):
            altered = copy.deepcopy(manifest); change(altered); altered_raw = e.encode(altered)
            (collection/"manifest.json").write_bytes(altered_raw)
            with self.assertRaisesRegex(e.Stop, "recovery_route_binding"):
                e.retained_recovery(self.core, self.collector, self.core.helpers()[3], state, collection, e.sha(altered_raw))
        (collection/"manifest.json").write_bytes(manifest_raw)

    def test_private_split_refuses_concrete_pair_defects_and_legacy_grant_before_callback(self):
        self.private_package(); transport = self.transport
        for defect in ("slot", "version", "missing", "duplicate"):
            with self.subTest(defect=defect), tempfile.TemporaryDirectory() as temp:
                self.journal = str(Path(temp)/("au-phase10-"+e.TAIL+".exception")); self.make_authorization()
                def damaged(raw):
                    status, headers, body = transport(raw); queries = json.loads(raw)
                    indices = [i for i,q in enumerate(queries) if q["method"] == "getMultipleAccounts"]
                    if len(indices) == 2:
                        values = json.loads(body); target = values[indices[1]]
                        if defect == "slot": target["result"]["context"]["slot"] += 1
                        elif defect == "version": target["result"]["context"]["apiVersion"] = "wrong"
                        elif defect == "missing": target["result"]["value"].pop()
                        else: target["id"] = values[indices[0]]["id"]
                        body = e.encode(values)
                    return status, headers, body
                created = self.run_action("CREATE")
                with mock.patch.object(self, "transport", side_effect=damaged), self.assertRaises(ValueError):
                    self.run_action("CAPTURE_SIGN_SEND", created["tail"])
                self.assertEqual(self.signs, 0); self.assertEqual(self.sends, 0)
                directory = Path(self.journal+".evidence")/"00000001-presign"
                self.assertTrue((directory/"rpc-02.response.bin").is_file()); self.assertTrue((directory/"rpc-02.http.json").is_file())
        legacy = copy.deepcopy(self.authorization); legacy["schema"] = e.AUTH_SCHEMA; legacy["sourcePins"].pop(e.RPC_SOURCE)
        raw = e.encode(legacy)
        with self.assertRaisesRegex(e.Stop, "authorization_binding"):
            e.successor_authorization(self.value, raw, e.sha(raw), self.journal)


if __name__ == "__main__": unittest.main()
