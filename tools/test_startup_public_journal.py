"""Offline durable public journal tests. Existing historical signatures only."""
import base64
import copy
import importlib.util
import json
import multiprocessing
import os
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest import mock

HERE = Path(__file__).parent
spec = importlib.util.spec_from_file_location("journal", HERE / "startup_public_journal.py")
j = importlib.util.module_from_spec(spec)
spec.loader.exec_module(j)
REPO = Path(os.environ.get("PIV1_PUBLIC_CHECK_REPO", HERE.parent))
RAW = (REPO / "clients/cli/startup-public-check.fixture.json").read_bytes()
assert j.sha(RAW) == "0855cd85667a565f4b6184ef2b389c2e4ec4421582f2b52f58b2cf8897852c0d"
FIXTURE = json.loads(RAW)
ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"


def b58(raw):
    n = int.from_bytes(raw, "big"); text = ""
    while n: n, r = divmod(n, 58); text = ALPHABET[r] + text
    return "1" * (len(raw) - len(raw.lstrip(b"\0"))) + text


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()


def segment(index=0, identity=None, cap="25000"):
    t = FIXTURE["transactions"][index]
    wire = base64.b64decode(t["signedWireBase64"])
    message = wire[1 + 64 * t["signatureCount"]:]
    n = message[3]
    keys = [b58(message[4 + i * 32:36 + i * 32]) for i in range(n)]
    roles = [{"address": key, "signer": i < message[0], "writable": i < message[0] - message[1] if i < message[0] else i < n - message[2]} for i, key in enumerate(keys)]
    phase = {"id": identity or "historical_" + t["phase"], "packet": {"signedWireBase64": t["signedWireBase64"], "reviewed": {
        "messageBase64": base64.b64encode(message).decode(), "messageSha256": t["messageSha256"], "feePayer": keys[0], "requiredSigners": keys[:message[0]], "accountRoles": roles, "lookupTables": []}},
        "lifetime": {"recentBlockhash": b58(message[4 + 32*n:36 + 32*n]), "currentBlockHeight": "18446744073709551500", "lastValidBlockHeight": str(j.U64), "minimumRemainingBlockHeights": "40"},
        "feeQuoteLamports": t["feeLamports"], "feeCapLamports": t["feeLamports"], "expectedPoststate": copy.deepcopy(FIXTURE["fundPoststate"]["accounts"])}
    return {"id": "segment_" + str(index), "input": {"schema": "piv1-startup-public-check-v1", "frozen": {"packageSha256": "a"*64, "payloadSha256": "b"*64, "anchorUnixTimestamp": None, "transactionIndex": "1"}, "cumulativeFeeCapLamports": cap, "phases": [phase], "trace": []}}


def event(s, state, prior=None, bad_balance=False, slot=None):
    phase = s["input"]["phases"][0]; index = next(i for i, t in enumerate(FIXTURE["transactions"]) if t["signedWireBase64"] == phase["packet"]["signedWireBase64"])
    t = FIXTURE["transactions"][index]; wire = base64.b64decode(t["signedWireBase64"])
    observation = None
    if state == "FINALIZED":
        slot = slot or t["slot"]
        observation = {"status": {"signatureHex": wire[1:65].hex(), "slot": slot, "confirmationStatus": "finalized", "err": None}, "transaction": {"slot": slot, "wireBase64": t["signedWireBase64"], "feeLamports": t["feeLamports"], "err": None}, "poststate": {"contextSlot": slot, "accounts": copy.deepcopy(phase["expectedPoststate"])}}
        if bad_balance: observation["poststate"]["accounts"][next(iter(phase["expectedPoststate"]))]["lamports"] = "1"
    return {"previousDigest": j.sha(canonical(prior)) if prior else None, "phase": phase["id"], "state": state, "packetSha256": t["signedWireSha256"], "signatureHex": wire[1:65].hex(), "frozen": copy.deepcopy(s["input"]["frozen"]), "observation": observation}


def request(action, tail=None, s=None, e=None):
    return {"schema": j.SCHEMA, "action": action, "expectedTail": tail, "segment": s, "event": e}


def append_states(path, s, tail, states):
    prior = None
    for state in states:
        e = event(s, state, prior)
        result = j.apply(request("APPEND_EVENT", tail, e=e), path)
        tail = result["tail"]; prior = e
    return tail, prior


def race_append(path, tail, e, queue):
    try: queue.put(("ok", j.apply(request("APPEND_EVENT", tail, e=e), path)["tail"]))
    except Exception as exc: queue.put(("stop", str(exc)))


def copy_local_journal(source, target):
    # Test-only copy is deliberately a new host-local instance. Re-anchor its
    # lock inode and recompute receipts, rather than pretending an old external
    # receipt authenticates a relocated whole-directory clone.
    shutil.copytree(source, target)
    lock = (target / "lock").stat(); previous = None; final = None
    for name in sorted(target.glob("*.json")):
        record = json.loads(name.read_bytes())
        record["lockIdentity"] = {"device": str(lock.st_dev), "inode": str(lock.st_ino), "ctimeNs": str(lock.st_ctime_ns)}
        record["previousDigest"] = previous
        record["digest"] = j.sha(j.encode({k: v for k, v in record.items() if k != "digest"}))
        name.write_bytes(j.encode(record)); previous = record["digest"]
        final = {"sequence": record["sequence"], "digest": record["digest"]}
    return final


class JournalTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.storage = tempfile.TemporaryDirectory()
        cls.seed = Path(cls.storage.name) / "seed"
        cls.s = segment()
        cls.initial_tail = j.apply(request("CREATE", s=cls.s), cls.seed)["tail"]
        cls.complete = Path(cls.storage.name) / "complete"
        copied_tail = copy_local_journal(cls.seed, cls.complete)
        cls.complete_tail, cls.final_event = append_states(cls.complete, cls.s, copied_tail, ["PREPARED", "SUBMISSION_START_RECORDED", "FINALIZED"])

    @classmethod
    def tearDownClass(cls): cls.storage.cleanup()

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.path = Path(self.temporary.name) / "journal"

    def clone(self, complete=False):
        return copy_local_journal(self.complete if complete else self.seed, self.path)

    def test_template_and_actual_reopen_inspect_fixed_node_checks(self):
        self.assertEqual(j.apply(j.TEMPLATE, self.path)["status"], "INCOMPLETE")
        tail = self.clone(True)
        result = j.apply(request("INSPECT", tail), self.path)
        self.assertEqual(result["tail"], tail)
        self.assertEqual(result["actualFeeLamports"], "5000")
        self.assertFalse(result["liveReady"]); self.assertFalse(result["authorized"])
        self.assertEqual(result["checkerReports"][0]["status"], "VERIFIED_PUBLIC_DECLARATIONS_ONLY")
        for path in self.path.iterdir(): self.assertEqual(path.stat().st_mode & 0o777, 0o600)

    def test_linked_unique_second_segment_and_global_fee_totals(self):
        tail = self.clone(True); s = segment(1)
        tail = j.apply(request("START_SEGMENT", tail, s=s), self.path)["tail"]
        tail, _ = append_states(self.path, s, tail, ["PREPARED", "SUBMISSION_START_RECORDED", "FINALIZED"])
        result = j.apply(request("INSPECT", tail), self.path)
        self.assertEqual(result["segments"], 2)
        self.assertEqual(result["reservedFeeLamports"], "15000")
        self.assertEqual(result["actualFeeLamports"], "15000")
        # Phase2's raw subset is deliberately synthetic declarations; no claim
        # that fund's balances are create's actual historical balances.

    def test_cas_stale_tail_missing_lock_and_existing_directory_reject(self):
        tail = self.clone(True)
        with mock.patch.object(j, "check_segments", side_effect=AssertionError("must reject before checker")):
            with self.assertRaisesRegex(j.Stop, "tail_compare_and_swap"): j.apply(request("INSPECT", self.initial_tail), self.path)
        (self.path / "lock").unlink()
        with self.assertRaises(FileNotFoundError): j.apply(request("INSPECT", tail), self.path)
        self.assertFalse((self.path / "lock").exists())
        with self.assertRaises(FileExistsError): j.apply(request("CREATE", s=self.s), self.path)



    def test_fifo_record_rejects_without_blocking_before_fstat(self):
        tail = self.clone()
        record = self.path / "00000000.json"; record.unlink(); os.mkfifo(record, 0o600)
        context = multiprocessing.get_context("fork"); queue = context.Queue()
        child = context.Process(target=race_append, args=(str(self.path), tail, event(self.s, "PREPARED"), queue))
        child.start(); child.join(3)
        if child.is_alive():
            child.terminate(); child.join(); self.fail("special-file read blocked before type check")
        self.assertEqual(child.exitcode, 0)
        kind, reason = queue.get(timeout=1)
        self.assertEqual(kind, "stop"); self.assertEqual(reason, "record_mode_owner_link")

    def test_replaced_same_content_lock_is_rejected_from_durable_inode_binding(self):
        tail = self.clone(True)
        lock = self.path / "lock"; lock.unlink()
        lock.write_bytes(j.LOCK_BYTES); lock.chmod(0o600)
        with self.assertRaisesRegex(j.Stop, "stable_lock_replaced"):
            j.apply(request("INSPECT", tail), self.path)

    def test_generated_truncation_canonical_digest_and_gap_corruption_fail_before_node(self):
        tail = self.clone(True); record = self.path / "00000002.json"; raw = record.read_bytes()
        with mock.patch.object(j, "check_segments", side_effect=AssertionError("corrupt prefix must not reach checker")):
            for cut in [0, 1, 2, 7, len(raw)//2, len(raw)-1]:
                record.write_bytes(raw[:cut])
                with self.assertRaises(j.Stop): j.apply(request("INSPECT", tail), self.path)
            record.write_bytes(raw + b" ")
            with self.assertRaisesRegex(j.Stop, "record_order_or_canonical"): j.apply(request("INSPECT", tail), self.path)
            changed = json.loads(raw); changed["data"]["packetSha256"] = "0"*64; record.write_bytes(j.encode(changed))
            with self.assertRaisesRegex(j.Stop, "record_digest"): j.apply(request("INSPECT", tail), self.path)
            record.write_bytes(raw); record.unlink()
            with self.assertRaisesRegex(j.Stop, "record_gap_or_missing"): j.apply(request("INSPECT", tail), self.path)

    def test_unknown_and_mismatch_never_link_or_change_durable_prefix(self):
        tail = self.clone(); tail, prior = append_states(self.path, self.s, tail, ["PREPARED", "SUBMISSION_START_RECORDED", "UNKNOWN"])
        before = {p.name: p.read_bytes() for p in self.path.iterdir()}
        with self.assertRaisesRegex(j.Stop, "predecessor_unresolved"): j.apply(request("START_SEGMENT", tail, s=segment(1)), self.path)
        bad = event(self.s, "FINALIZED", prior, bad_balance=True)
        with self.assertRaises(j.Stop) as error: j.apply(request("APPEND_EVENT", tail, e=bad), self.path)
        self.assertEqual(error.exception.diagnostics["reason"], "poststate_mismatch")
        self.assertEqual(before, {p.name: p.read_bytes() for p in self.path.iterdir()})
        good = event(self.s, "FINALIZED", prior)
        result = j.apply(request("APPEND_EVENT", tail, e=good), self.path)
        self.assertEqual(result["actualFeeLamports"], "5000")

    def test_freeze_anchor_cap_and_global_phase_packet_replay_reject(self):
        tail = self.clone(True)
        for field, value in [("anchorUnixTimestamp", "1"), ("payloadSha256", "c"*64), ("transactionIndex", "2")]:
            s = segment(1); s["input"]["frozen"][field] = value
            with self.assertRaisesRegex(j.Stop, "cross_segment_freeze_or_cap"): j.apply(request("START_SEGMENT", tail, s=s), self.path)
        s = segment(1); s["input"]["cumulativeFeeCapLamports"] = "50001"
        with self.assertRaisesRegex(j.Stop, "cross_segment_freeze_or_cap"): j.apply(request("START_SEGMENT", tail, s=s), self.path)
        s = segment(1, identity=self.s["input"]["phases"][0]["id"])
        with self.assertRaisesRegex(j.Stop, "global_phase_replay"): j.apply(request("START_SEGMENT", tail, s=s), self.path)
        s = segment(0, identity="different_phase"); s["id"] = "different_segment"
        with self.assertRaisesRegex(j.Stop, "global_packet_replay"): j.apply(request("START_SEGMENT", tail, s=s), self.path)
        self.assertEqual(len(list(self.path.iterdir())), 5)

    def test_cross_segment_decreasing_and_equal_inclusion_slots_stop(self):
        tail = self.clone(True); s = segment(1)
        tail = j.apply(request("START_SEGMENT", tail, s=s), self.path)["tail"]
        tail, prior = append_states(self.path, s, tail, ["PREPARED", "SUBMISSION_START_RECORDED"])
        for slot in [FIXTURE["transactions"][0]["slot"], str(int(FIXTURE["transactions"][0]["slot"])-1)]:
            e = event(s, "FINALIZED", prior, slot=slot)
            with self.assertRaisesRegex(j.Stop, "global_slot_not_later"): j.apply(request("APPEND_EVENT", tail, e=e), self.path)

    def test_cross_segment_fee_reservation_cannot_reset(self):
        s = segment(cap="10000")
        tail = j.apply(request("CREATE", s=s), self.path)["tail"]
        tail, _ = append_states(self.path, s, tail, ["PREPARED", "SUBMISSION_START_RECORDED", "FINALIZED"])
        nxt = segment(1, cap="10000")
        tail = j.apply(request("START_SEGMENT", tail, s=nxt), self.path)["tail"]
        e = event(nxt, "PREPARED"); tail = j.apply(request("APPEND_EVENT", tail, e=e), self.path)["tail"]
        with self.assertRaisesRegex(j.Stop, "global_fee_cap"): j.apply(request("APPEND_EVENT", tail, e=event(nxt, "SUBMISSION_START_RECORDED", e)), self.path)

    def test_concurrent_append_serializes_one_winner_and_stale_cas_loser(self):
        tail = self.clone(); e = event(self.s, "PREPARED")
        context = multiprocessing.get_context("fork"); queue = context.Queue()
        children = [context.Process(target=race_append, args=(str(self.path), tail, e, queue)) for _ in range(2)]
        for child in children: child.start()
        for child in children: child.join(30); self.assertEqual(child.exitcode, 0)
        outcomes = [queue.get(timeout=2) for _ in children]
        self.assertEqual(sorted(item[0] for item in outcomes), ["ok", "stop"])
        self.assertIn("tail_compare_and_swap", [item[1] for item in outcomes if item[0] == "stop"])
        self.assertEqual(len(list(self.path.iterdir())), 3)

    def test_partial_record_and_failed_fsync_are_never_repaired_or_adopted(self):
        tail = self.clone(); e = event(self.s, "PREPARED")
        real_fsync = j.os.fsync
        calls = 0
        def fail_directory(fd):
            nonlocal calls
            calls += 1
            if calls == 2: raise OSError("offline directory fsync failure")
            return real_fsync(fd)
        with mock.patch.object(j.os, "fsync", fail_directory):
            with self.assertRaisesRegex(j.Stop, "commit_uncertain_no_retry_or_adoption"): j.apply(request("APPEND_EVENT", tail, e=e), self.path)
        # A fully written but unacknowledged record is not automatically adopted.
        with self.assertRaisesRegex(j.Stop, "tail_compare_and_swap"): j.apply(request("APPEND_EVENT", tail, e=e), self.path)
        record = self.path / "00000001.json"; record.write_bytes(record.read_bytes()[:9])
        with self.assertRaises(j.Stop): j.apply(request("INSPECT", tail), self.path)
        self.assertEqual(len(record.read_bytes()), 9)

    def test_source_checker_failure_reserve_and_symlinks_stop_before_commit(self):
        tail = self.clone(); e = event(self.s, "PREPARED")
        with mock.patch.object(j, "check_segments", side_effect=j.Stop("checker_timeout")):
            with self.assertRaisesRegex(j.Stop, "checker_timeout"): j.apply(request("APPEND_EVENT", tail, e=e), self.path)
        with mock.patch.object(j, "pins", side_effect=j.Stop("source_pin")):
            with self.assertRaisesRegex(j.Stop, "source_pin"): j.apply(request("APPEND_EVENT", tail, e=e), self.path)
        with mock.patch.object(j, "space", side_effect=j.Stop("reserve_2gib")):
            with self.assertRaisesRegex(j.Stop, "commit_uncertain"): j.apply(request("APPEND_EVENT", tail, e=e), self.path)
        self.assertEqual(len(list(self.path.iterdir())), 2)
        lock = self.path / "lock"; saved = self.path / "saved"; lock.rename(saved); lock.symlink_to(saved)
        with self.assertRaises(OSError): j.apply(request("INSPECT", tail), self.path)


if __name__ == "__main__": unittest.main()
