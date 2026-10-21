"""Offline generated properties and public-response contracts; no network/signing."""
import copy
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest import mock

HERE = Path(__file__).parent
spec = importlib.util.spec_from_file_location("collector", HERE / "collect_startup_public_evidence.py")
c = importlib.util.module_from_spec(spec)
spec.loader.exec_module(c)
REPO = Path(os.environ.get("PIV1_PUBLIC_CHECK_REPO", HERE.parent))
FIXTURE_PATH = REPO / "clients/cli/startup-public-check.fixture.json"
FIXTURE_RAW = FIXTURE_PATH.read_bytes()
assert c.sha(FIXTURE_RAW) == "0855cd85667a565f4b6184ef2b389c2e4ec4421582f2b52f58b2cf8897852c0d"
FIXTURE = json.loads(FIXTURE_RAW)


def request():
    t = FIXTURE["transactions"][0]
    return {"schema": c.SCHEMA, "wireBase64": t["signedWireBase64"], "expectedWireSha256": t["signedWireSha256"],
            "expectedMessageSha256": t["messageSha256"], "signature": t["signature"], "expectedFeeLamports": t["feeLamports"],
            "accountKeys": list(FIXTURE["fundPoststate"]["accounts"]), "minimumContextSlot": "454845162", "includeExpiry": False}


class Replies:
    def __init__(self, value=None, mutate=None, http_status=200, fail_at=None, output=None):
        self.value = value or request()
        self.mutate = mutate
        self.http_status = http_status
        self.fail_at = fail_at
        self.calls = []
        self.output = output

    def __call__(self, raw):
        requests = json.loads(raw)
        self.calls.append(requests)
        if self.output:
            self.assert_started(len(self.calls))
        if self.fail_at == len(self.calls):
            raise c.TransportFailure("offline_partial_failure", b'{"partial":')
        responses = []
        for q in requests:
            method = q["method"]
            slot = int(self.value["minimumContextSlot"])
            if method == "getGenesisHash":
                result = c.GENESIS
            elif method == "getSignatureStatuses":
                assert q["params"] == [[self.value["signature"]], {"searchTransactionHistory": True}]
                result = {"context": {"slot": slot}, "value": [{"slot": slot, "confirmationStatus": "finalized", "err": None}]}
            elif method == "getTransaction":
                assert q["params"][1] == {"encoding": "base64", "commitment": "finalized", "maxSupportedTransactionVersion": 0}
                result = {"slot": slot, "version": "legacy", "transaction": [self.value["wireBase64"], "base64"], "meta": {"fee": int(self.value["expectedFeeLamports"]), "err": None,
                          "unusedTokenBalances": [{"uiAmount": 0.5}]}}
            elif method == "getMultipleAccounts":
                assert q["params"][1] == {"encoding": "base64", "commitment": "finalized", "minContextSlot": slot}
                accounts = FIXTURE["fundPoststate"]["accounts"]
                result = {"context": {"slot": slot}, "value": [{"owner": accounts[k]["owner"], "executable": False, "lamports": int(accounts[k]["lamports"]), "data": ["", "base64"]} for k in self.value["accountKeys"]]}
            elif method == "getBlockHeight":
                result = c.U64
            elif method == "isBlockhashValid":
                result = {"context": {"slot": slot}, "value": False}
            else:
                raise AssertionError("unapproved RPC method")
            response = {"jsonrpc": "2.0", "id": q["id"], "result": result}
            if self.mutate:
                self.mutate(method, response)
            responses.append(response)
        return self.http_status, [("content-type", "application/json")], json.dumps(responses).encode()

    def assert_started(self, number):
        assert (self.output / f"http-{number:02}.started.json").is_file()
        assert (self.output / f"http-{number:02}.request.json").is_file()


class CollectorTests(unittest.TestCase):
    def run_collect(self, value=None, **kwargs):
        value = value or request()
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "public"
            transport = Replies(value=value, output=output, **kwargs)
            report = c.collect(value, output, _transport=transport)
            manifest = json.loads((output / "manifest.json").read_bytes())
            self.assertEqual(manifest["counts"], report["counts"])
            self.assertEqual(os.stat(output).st_mode & 0o777, 0o700)
            for item in manifest["artifacts"]:
                raw = (output / item["name"]).read_bytes()
                self.assertEqual(c.sha(raw), item["sha256"])
                self.assertEqual(os.stat(output / item["name"]).st_mode & 0o777, 0o600)
            return report, copy.deepcopy(transport.calls), {p.name: p.read_bytes() for p in output.iterdir()}

    def test_positive_public_wire_and_current_declared_subset(self):
        r, calls, files = self.run_collect()
        self.assertEqual(r["status"], "COLLECTED_PUBLIC_EVIDENCE_ONLY")
        self.assertEqual(r["counts"]["logicalReadsStarted"], 4)
        self.assertEqual(r["counts"]["httpAttemptsStarted"], 3)
        self.assertEqual(r["observation"]["transaction"]["feeLamports"], "5000")
        self.assertEqual(r["observation"]["poststate"]["accounts"], FIXTURE["fundPoststate"]["accounts"])
        self.assertFalse(r["authorized"])
        self.assertFalse(r["liveReady"])
        self.assertFalse(r["cryptographicSignaturesVerified"])
        self.assertIn("http-02.response.bin", files)
        self.assertIsInstance(json.loads(files["http-01.http.json"])["responseCompletedUtc"], str)
        self.assertEqual(c.packet(request()["wireBase64"])["signature"], request()["signature"])
        self.assertEqual(c.checker_pin()["sha256"], c.CHECKER_SHA256)

    def test_nullable_and_strict_input_no_network(self):
        self.assertEqual(c.validate(c.TEMPLATE)["status"], "INCOMPLETE")
        for field, value in [("endpoint", "https://user:password@elsewhere.invalid"), ("schema", "different")]:
            r = request(); r[field] = value
            with self.assertRaises(c.Stop): c.validate(r)
        for field, value in [("expectedFeeLamports", -1), ("minimumContextSlot", "01"), ("minimumContextSlot", "18446744073709551616"), ("includeExpiry", 1)]:
            r = request(); r[field] = value
            with self.assertRaises(c.Stop): c.validate(r)
        r = request(); r["accountKeys"] *= 2
        with self.assertRaisesRegex(c.Stop, "duplicate_account"): c.validate(r)
        r = request(); r["signature"] = "https://user:password@elsewhere.invalid"
        with self.assertRaises(c.Stop): c.validate(r)
        for field in ["expectedWireSha256", "expectedMessageSha256"]:
            r = request(); r[field] = "0" * 64
            with self.assertRaises(c.Stop): c.validate(r)

    def test_generated_base58_base64_roundtrips_and_invalid_canonical_encodings(self):
        for size in (32, 64):
            for tag in range(256):
                raw = bytes([tag]) * size
                self.assertEqual(c.b58decode(c.b58encode(raw), size), raw)
                text = c.base64.b64encode(raw).decode()
                self.assertEqual(c.b64(text, size), raw)
        for value in ["AB==", "AA==\n", "AA", "AA===", "_A=="]:
            with self.assertRaises(c.Stop): c.b64(value, 32)
        for value in ["0" * 32, "1" * 33, "1" * 45]:
            with self.assertRaises(c.Stop): c.b58decode(value, 32)
        raw = c.packet(request()["wireBase64"])["wire"]
        for prefix in (b"\x81\x00", b"\x80\x80\x80", b"\xff\xff\x04"):
            with self.assertRaises(c.Stop): c.packet(c.base64.b64encode(prefix + raw[1:]).decode())

    def test_u64_precision_and_relevant_float_bool_overflow_rejection(self):
        value = request(); value["minimumContextSlot"] = value["expectedFeeLamports"] = str(c.U64)
        r, _, _ = self.run_collect(value)
        self.assertEqual(r["status"], "COLLECTED_PUBLIC_EVIDENCE_ONLY")
        self.assertEqual(r["observation"]["status"]["slot"], str(c.U64))
        self.assertEqual(r["observation"]["transaction"]["feeLamports"], str(c.U64))
        for bad in [True, False, -1, 1.5, 5000.0, c.U64 + 1, "5000"]:
            def mutate(method, response, bad=bad):
                if method == "getTransaction": response["result"]["meta"]["fee"] = bad
            r, _, _ = self.run_collect(mutate=mutate)
            self.assertEqual(r["status"], "STOP")
        for version in [True, False, 0.0, 1, "0", None]:
            def mutate(method, response, version=version):
                if method == "getTransaction": response["result"]["version"] = version
            self.assertEqual(self.run_collect(mutate=mutate)[0]["reason"], "transaction_version")
        with self.assertRaises(c.Stop): c.strict_json(b'{"a":NaN}')
        with self.assertRaises(c.Stop): c.strict_json(b'{"a":1,"a":2}')
        self.assertEqual(c.strict_json(b'{"unusedUiAmount":0.5}')["unusedUiAmount"], c.Decimal("0.5"))

    def test_status_id_wire_slot_fee_genesis_and_context_mismatches_stop(self):
        changes = [
            ("getGenesisHash", lambda x: x.update(result="different")),
            ("getTransaction", lambda x: x.update(id=True)),
            ("getTransaction", lambda x: x.update(id=999)),
            ("getTransaction", lambda x: x["result"].update(slot=1)),
            ("getTransaction", lambda x: x["result"].update(version=0)),
            ("getTransaction", lambda x: x["result"].update(transaction=[FIXTURE["transactions"][1]["signedWireBase64"], "base64"])),
            ("getTransaction", lambda x: x["result"]["meta"].update(fee=5001)),
            ("getSignatureStatuses", lambda x: x["result"]["context"].update(slot=1)),
            ("getMultipleAccounts", lambda x: x["result"]["context"].update(slot=1)),
            ("getSignatureStatuses", lambda x: x["result"]["value"][0].update(confirmationStatus="invented")),
        ]
        for name, change in changes:
            def mutate(method, response):
                if method == name: change(response)
            r, _, _ = self.run_collect(mutate=mutate)
            self.assertEqual(r["status"], "STOP")

    def test_null_pending_pruned_expired_and_failed_cannot_advance(self):
        for target in ("getTransaction", "getSignatureStatuses"):
            def mutate(method, response):
                if method == target:
                    if target == "getTransaction": response["result"] = None
                    else: response["result"]["value"] = [None]
            r, calls, _ = self.run_collect(mutate=mutate)
            self.assertEqual(r["status"], "UNKNOWN")
            self.assertEqual(len(calls), 2)
        def pending(method, response):
            if method == "getSignatureStatuses": response["result"]["value"][0]["confirmationStatus"] = "confirmed"
        self.assertEqual(self.run_collect(mutate=pending)[0]["status"], "UNKNOWN")
        value = request(); value["includeExpiry"] = True
        r, calls, _ = self.run_collect(value)
        self.assertEqual(r["status"], "COLLECTED_PUBLIC_EVIDENCE_ONLY")
        self.assertFalse(r["expiry"]["blockhashValid"])
        self.assertFalse(r["expiry"]["nonInclusionProved"])
        self.assertEqual((r["counts"]["logicalReadsStarted"], len(calls)), (6, 4))
        def expired_unknown(method, response):
            if method == "getSignatureStatuses": response["result"]["value"] = [None]
        unknown, _, _ = self.run_collect(value, mutate=expired_unknown)
        self.assertEqual(unknown["status"], "UNKNOWN")
        self.assertFalse(unknown["expiry"]["nonInclusionProved"])
        def failed(method, response):
            if method == "getSignatureStatuses": response["result"]["value"][0]["err"] = {"InstructionError": [0, "Custom"]}
            if method == "getTransaction": response["result"]["meta"]["err"] = {"InstructionError": [0, "Custom"]}
        r, _, _ = self.run_collect(mutate=failed)
        self.assertEqual(r["reason"], "FINALIZED_TRANSACTION_FAILURE")
        self.assertIsNone(r["observation"]["poststate"])

    def test_absence_raw_u64_account_bound_and_encoding(self):
        def absent(method, response):
            if method == "getMultipleAccounts": response["result"]["value"][0] = None
        r, _, _ = self.run_collect(mutate=absent)
        a = r["observation"]["poststate"]["accounts"][request()["accountKeys"][0]]
        self.assertEqual(a, {"exists": False, "owner": None, "executable": None, "lamports": None, "dataBase64": None})
        for field, bad in [("lamports", True), ("lamports", c.U64 + 1), ("executable", 1), ("owner", "invalid"), ("data", ["AB==", "base64"]), ("data", ["", "jsonParsed"])]:
            account = {"owner": "11111111111111111111111111111111", "executable": False, "lamports": 0, "data": ["", "base64"]}; account[field] = bad
            with self.assertRaises(c.Stop): c.normalize_account(account)
        account["data"] = [c.base64.b64encode(bytes(c.MAX_ACCOUNT + 1)).decode(), "base64"]
        with self.assertRaises(c.Stop): c.normalize_account(account)

    def test_raw_partial_rpc_transport_failure_and_counters_preserved(self):
        def error(method, response):
            if method == "getTransaction": response.pop("result"); response["error"] = {"code": -32000, "message": "untrusted server text"}
        r, _, files = self.run_collect(mutate=error)
        self.assertEqual(r["reason"], "rpc_error")
        self.assertEqual(r["counts"]["rpcErrors"], 1)
        self.assertIn(b"untrusted server text", files["http-02.response.bin"])
        r, _, files = self.run_collect(fail_at=2)
        self.assertEqual(r["counts"]["httpAttemptsStarted"], 2)
        self.assertEqual(r["counts"]["httpResponsesCompleted"], 1)
        self.assertEqual(r["counts"]["httpTransportFailures"], 1)
        self.assertEqual(files["http-02.partial-response.bin"], b'{"partial":')

    def test_redirect_proxy_exclusive_directory_symlinks_and_fsync(self):
        r, calls, _ = self.run_collect(http_status=302)
        self.assertEqual(r["reason"], "redirect_forbidden")
        self.assertEqual(len(calls), 1)
        with mock.patch.dict(os.environ, {"HTTPS_PROXY": "http://not-allowed.invalid"}):
            with self.assertRaisesRegex(c.Stop, "proxy_environment_unsupported"): c.tls_post(b"[]")
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); existing = root / "existing"; existing.mkdir()
            with self.assertRaises(FileExistsError): c.collect(request(), existing, _transport=Replies())
            link = root / "link"; link.symlink_to(existing, target_is_directory=True)
            with self.assertRaises(OSError): c.collect(request(), link / "child", _transport=Replies())
            output = root / "new"
            with mock.patch.object(c.os, "fsync", wraps=os.fsync) as syncing:
                r = c.collect(request(), output, _transport=Replies(output=output))
                self.assertGreaterEqual(syncing.call_count, 10)
            self.assertEqual(r["status"], "COLLECTED_PUBLIC_EVIDENCE_ONLY")


    def test_started_record_failure_does_not_invoke_transport_or_count_attempt(self):
        original = c.Store.write
        def fail_started(store, name, raw):
            if name.endswith(".started.json"):
                raise OSError("offline persistence failure")
            return original(store, name, raw)
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "public"
            transport = Replies()
            with mock.patch.object(c.Store, "write", fail_started):
                result = c.collect(request(), output, _transport=transport)
            self.assertEqual(result["status"], "STOP")
            self.assertEqual(result["counts"]["httpAttemptsStarted"], 0)
            self.assertEqual(result["counts"]["logicalReadsStarted"], 0)
            self.assertEqual(transport.calls, [])
            self.assertTrue((output / "http-01.request.json").exists())
            self.assertTrue((output / "manifest.json").exists())

    def test_truncated_fixed_content_length_is_partial_transport_failure(self):
        class Response:
            status = 200
            length = 3
            def getheaders(self): return [("Content-Length", "3")]
            def read(self, _):
                if self.length == 3:
                    self.length = 1
                    return b"[]"
                return b""
        connection = mock.Mock()
        connection.getresponse.return_value = Response()
        with mock.patch.dict(os.environ, {key: "" for key in c.PROXIES}):
            with mock.patch.object(c.http.client, "HTTPSConnection", return_value=connection):
                with self.assertRaisesRegex(c.TransportFailure, "incomplete_http_body") as error:
                    c.tls_post(b"[]")
        self.assertEqual(error.exception.partial, b"[]")
        connection.request.assert_called_once()
        connection.close.assert_called_once()


    def test_falsey_offline_hook_never_falls_back_to_real_tls(self):
        class FalseyReplies(Replies):
            def __bool__(self): return False
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "public"
            transport = FalseyReplies(output=output)
            with mock.patch.object(c.http.client, "HTTPSConnection", side_effect=AssertionError("real TLS forbidden")):
                result = c.collect(request(), output, _transport=transport)
                self.assertEqual(result["status"], "COLLECTED_PUBLIC_EVIDENCE_ONLY")
                self.assertEqual(result["transportMode"], "OFFLINE_INJECTED_TEST")
                with self.assertRaisesRegex(c.Stop, "offline_transport_callable"):
                    c.collect(request(), Path(temporary) / "rejected", _transport=False)
            self.assertFalse((Path(temporary) / "rejected").exists())


if __name__ == "__main__":
    unittest.main()
