"""Preparation-only portable host tests; synthetic history, no valid signatures.

Synthetic pin overrides are explicit and local to each test. Actual AU55/tail,
artifact and wire are checked separately by the root's once-only public CLI run.
No original operator import, subprocess, network, keys or signing occurs here.
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
        self.assertIn("NOT_IMPLEMENTED", result["remainingIntegrationBlocker"])
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


if __name__ == "__main__": unittest.main()
