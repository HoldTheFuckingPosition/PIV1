"""Offline synthetic Clock commitments; existing historical signatures only."""
import base64
import copy
import importlib.util
import json
import multiprocessing
import os
from pathlib import Path
import struct
import tempfile
import unittest
from unittest import mock

HERE = Path(__file__).parent
REPO = Path(os.environ.get("PIV1_PUBLIC_CHECK_REPO", HERE.parent))
spec = importlib.util.spec_from_file_location("clock_journal", HERE / "startup_public_journal.py")
j = importlib.util.module_from_spec(spec); spec.loader.exec_module(j)
helper_spec = importlib.util.spec_from_file_location("historical_journal_helpers", REPO / "tools/test_startup_public_journal.py")
h = importlib.util.module_from_spec(helper_spec); helper_spec.loader.exec_module(h)
RAW = (HERE / "startup_clock_binding.fixture.json").read_bytes()
assert j.sha(RAW) == "1e627e75208408177b6d07042bdfe307d791b0d0502b7c48961b56630c64670b"
FIXTURE = json.loads(RAW)


def freeze(tail, pre=None):
    return h.request("FREEZE_INITIALIZER_PROFILE", tail, s=copy.deepcopy(pre or FIXTURE["preManifest"]))


def seal(tail, clock=None):
    return h.request("SEAL_INITIALIZER_CLOCK", tail, s=copy.deepcopy(clock or FIXTURE["clock"]))


def changed_clock(**values):
    result = copy.deepcopy(FIXTURE["clock"])
    fields = dict(zip(("slot", "epochStartTimestamp", "epoch", "leaderScheduleEpoch", "unixTimestamp"), struct.unpack("<QqQQq", base64.b64decode(result["dataBase64"]))))
    fields.update(values)
    result["dataBase64"] = base64.b64encode(struct.pack("<QqQQq", *fields.values())).decode()
    return result


def race_seal(path, tail, queue):
    try: queue.put(("ok", j.apply(seal(tail), path)["tail"]))
    except Exception as exc: queue.put(("stop", str(exc)))


class ClockBindingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.storage = tempfile.TemporaryDirectory()
        cls.seed = Path(cls.storage.name) / "complete_parent"
        cls.segment = h.segment()
        cls.segment["input"]["frozen"]["transactionIndex"] = FIXTURE["preManifest"]["initializer"]["governance"]["transactionIndex"]
        tail = j.apply(h.request("CREATE", s=cls.segment), cls.seed)["tail"]
        prior = None
        for state in ("PREPARED", "SUBMISSION_START_RECORDED", "FINALIZED"):
            event = h.event(cls.segment, state, prior)
            tail = j.apply(h.request("APPEND_EVENT", tail, e=event), cls.seed)["tail"]; prior = event
        cls.completed_tail = tail

    @classmethod
    def tearDownClass(cls): cls.storage.cleanup()

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(); self.addCleanup(self.temporary.cleanup)
        self.path = Path(self.temporary.name) / "journal"
        self.tail = h.copy_local_journal(self.seed, self.path)

    def profile(self):
        return j.apply(freeze(self.tail), self.path)["tail"]

    def test_exact_profile_commit_then_clock_seal_reopen_and_native_compact_roles(self):
        before = self.tail
        result = j.apply(freeze(before), self.path); profile_tail = result["tail"]
        self.assertEqual(result["initializerProfile"]["parentTail"], before)
        self.assertEqual(result["initializerProfile"]["preManifest"], FIXTURE["preManifest"])
        self.assertIsNone(result["initializerProfile"]["preManifest"]["initializer"]["initialization"]["anchorUnixTimestamp"])
        sealed = j.apply(seal(profile_tail), self.path)
        binding = sealed["initializerSeal"]
        self.assertEqual(binding["profileTail"], profile_tail)
        self.assertEqual(binding["parentTail"], before)
        self.assertEqual(binding["decodedClock"]["unixTimestamp"], "1234567890")
        self.assertEqual(binding["decodedClock"]["epochStartTimestamp"], "-44")
        self.assertEqual(binding["contextFloor"], "9007199254740997")
        native = bytes.fromhex(binding["unsignedInitializerPlan"]["native"]["hex"])
        self.assertEqual(len(native), 315); self.assertEqual(native[:8], b"PIV1IN01")
        self.assertEqual(struct.unpack("<q", native[299:307])[0], 1234567890)
        expected_post = copy.deepcopy(FIXTURE["preManifest"]); expected_post["initializer"]["initialization"]["anchorUnixTimestamp"] = "1234567890"
        self.assertEqual(binding["postManifest"], expected_post)
        self.assertEqual(len(binding["unsignedInitializerPlan"]["packets"]), 9)
        self.assertFalse(sealed["authorized"]); self.assertFalse(sealed["liveReady"])
        self.assertFalse(sealed["nextSegmentAllowedByDeclarations"])
        inspected = j.apply(h.request("INSPECT", sealed["tail"]), self.path)
        self.assertEqual(inspected["initializerSeal"], binding)
        self.assertEqual(inspected["actualFeeLamports"], "5000")
        self.assertEqual(inspected["checkerProcessInvocations"], 1)
        self.assertEqual(inspected["initializerPlannerProcessInvocations"], 1)
        self.assertEqual(inspected["nodeProcessInvocations"], 2)


    def test_profile_reserves_one_seal_record_slot_before_any_mutation(self):
        count = len(list(self.path.glob("*.json")))
        before = {p.name: p.read_bytes() for p in self.path.iterdir()}
        with mock.patch.object(j, "MAX_RECORDS", count + 1):
            with mock.patch.object(j, "check_segments", side_effect=AssertionError("slot guard must run before checker")):
                with self.assertRaisesRegex(j.Stop, "profile_requires_seal_record_slot"):
                    j.apply(freeze(self.tail), self.path)
        self.assertEqual(before, {p.name: p.read_bytes() for p in self.path.iterdir()})
        # Exactly two remaining slots permit PROFILE followed by its one SEAL.
        with mock.patch.object(j, "MAX_RECORDS", count + 2):
            profile_tail = j.apply(freeze(self.tail), self.path)["tail"]
            sealed = j.apply(seal(profile_tail), self.path)
        self.assertEqual(sealed["journalMutationPolicy"], "TERMINALLY_SEALED_NO_CHILD_CONTROLLER")
        self.assertEqual(len(list(self.path.glob("*.json"))), count + 2)

    def test_profile_must_durably_precede_clock_and_parent_must_be_complete(self):
        with self.assertRaisesRegex(j.Stop, "clock_profile_missing_or_already_sealed"):
            j.apply(seal(self.tail), self.path)
        other = Path(self.temporary.name) / "unfinished"
        tail = j.apply(h.request("CREATE", s=self.segment), other)["tail"]
        with self.assertRaisesRegex(j.Stop, "clock_parent_not_complete"): j.apply(freeze(tail), other)
        other = Path(self.temporary.name) / "anchored"
        s = copy.deepcopy(self.segment); s["input"]["frozen"]["anchorUnixTimestamp"] = "0"
        tail = j.apply(h.request("CREATE", s=s), other)["tail"]
        prior = None
        for state in ("PREPARED", "SUBMISSION_START_RECORDED", "FINALIZED"):
            e = h.event(s, state, prior); tail = j.apply(h.request("APPEND_EVENT", tail, e=e), other)["tail"]; prior = e
        with self.assertRaisesRegex(j.Stop, "clock_parent_anchor_must_be_null"): j.apply(freeze(tail), other)

    def test_profile_shape_anchor_and_parent_index_are_checked_not_success_labels(self):
        before = {p.name: p.read_bytes() for p in self.path.iterdir()}
        for mutate in [lambda m: m["initializer"]["initialization"].update(anchorUnixTimestamp="0"), lambda m: m["initializer"]["protocol"].update(reserve=None), lambda m: m.update(success=True)]:
            profile = copy.deepcopy(FIXTURE["preManifest"]); mutate(profile)
            with self.assertRaises(j.Stop): j.apply(freeze(self.tail, profile), self.path)
        profile = copy.deepcopy(FIXTURE["preManifest"])
        profile["initializer"]["governance"]["transactionIndex"] = "74"
        profile["initializer"]["governance"]["currentTransactionIndex"] = "73"
        with self.assertRaisesRegex(j.Stop, "profile_parent_transaction_index"): j.apply(freeze(self.tail, profile), self.path)
        self.assertEqual(before, {p.name: p.read_bytes() for p in self.path.iterdir()})

    def test_profile_and_seal_block_old_mutations_reprofile_and_recapture(self):
        tail = self.profile()
        for req in [h.request("START_SEGMENT", tail, s=h.segment(1)), h.request("APPEND_EVENT", tail, e=h.event(self.segment, "PREPARED")), freeze(tail)]:
            with self.assertRaises(j.Stop): j.apply(req, self.path)
        sealed_tail = j.apply(seal(tail), self.path)["tail"]
        before = {p.name: p.read_bytes() for p in self.path.iterdir()}
        for req in [seal(sealed_tail), freeze(sealed_tail), h.request("APPEND_EVENT", sealed_tail, e=h.event(self.segment, "PREPARED")), h.request("START_SEGMENT", sealed_tail, s=h.segment(1))]:
            with self.assertRaises(j.Stop): j.apply(req, self.path)
        self.assertEqual(before, {p.name: p.read_bytes() for p in self.path.iterdir()})

    def test_generated_clock_bounds_identity_and_all_byte_lengths_reject_without_commit(self):
        tail = self.profile(); before = {p.name: p.read_bytes() for p in self.path.iterdir()}
        for field, value in [("address", "11111111111111111111111111111111"), ("owner", "11111111111111111111111111111111"), ("executable", True), ("contextSlot", True), ("contextSlot", "18446744073709551616"), ("contextSlot", "01")]:
            clock = copy.deepcopy(FIXTURE["clock"]); clock[field] = value
            with self.assertRaises(j.Stop): j.apply(seal(tail, clock), self.path)
        raw = base64.b64decode(FIXTURE["clock"]["dataBase64"])
        # These pure parser properties avoid a Node process per generated case.
        for n in range(0, 81):
            if n == 40: continue
            clock = copy.deepcopy(FIXTURE["clock"]); clock["dataBase64"] = base64.b64encode((raw + bytes(40))[:n]).decode()
            with self.assertRaises(j.Stop): j._clock(clock, 0)
        noncanonical = copy.deepcopy(FIXTURE["clock"]); noncanonical["dataBase64"] = noncanonical["dataBase64"][:-4] + "AB=="
        with self.assertRaisesRegex(j.Stop, "clock_base64"): j._clock(noncanonical, 0)
        for timestamp in (-1, -(1<<63)):
            with self.assertRaisesRegex(j.Stop, "clock_negative_anchor_unsupported"): j._clock(changed_clock(unixTimestamp=timestamp), 0)
        self.assertEqual(before, {p.name: p.read_bytes() for p in self.path.iterdir()})

    def test_u64_i64_edges_and_context_floor_include_profile_and_parent_poststate(self):
        clock = changed_clock(slot=j.U64, epoch=j.U64, leaderScheduleEpoch=j.U64, unixTimestamp=(1<<63)-1, epochStartTimestamp=-(1<<63)); clock["contextSlot"] = str(j.U64)
        decoded, _ = j._clock(clock, j.U64)
        self.assertEqual(decoded["unixTimestamp"], "9223372036854775807")
        self.assertEqual(decoded["epochStartTimestamp"], "-9223372036854775808")
        tail = self.profile(); max_binding = j.apply(seal(tail, clock), self.path)["initializerSeal"]
        max_native = bytes.fromhex(max_binding["unsignedInitializerPlan"]["native"]["hex"])
        self.assertEqual(struct.unpack("<q", max_native[299:307])[0], (1<<63)-1)
        with self.assertRaises(j.Stop): j._clock(changed_clock(slot=9007199254740996), 9007199254740997)
        with self.assertRaises(j.Stop): j._clock(changed_clock(slot=9007199254741002), 0)
        # A fresh synthetic parent poststate context later than ALT/reference
        # must also dominate the allowed Clock slot, even when inclusion is old.
        other = Path(self.temporary.name) / "late_parent"
        tail = j.apply(h.request("CREATE", s=self.segment), other)["tail"]; prior = None
        for state in ("PREPARED", "SUBMISSION_START_RECORDED", "FINALIZED"):
            e = h.event(self.segment, state, prior)
            if state == "FINALIZED": e["observation"]["poststate"]["contextSlot"] = "9007199254741002"
            tail = j.apply(h.request("APPEND_EVENT", tail, e=e), other)["tail"]; prior = e
        tail = j.apply(freeze(tail), other)["tail"]
        with self.assertRaisesRegex(j.Stop, "clock_context_or_slot_stale"): j.apply(seal(tail), other)

    def test_independent_byte_role_and_compact_checks_reject_tampering(self):
        tail = self.profile(); binding = j.apply(seal(tail), self.path)["initializerSeal"]
        plan, post = binding["unsignedInitializerPlan"], binding["postManifest"]
        with self.assertRaisesRegex(j.Stop, "native_clock_binding"): j._bindings(plan, post, 1234567891)
        for role_index in [0, 5, 7, 20, len(plan["native"]["roles"])-1]:
            bad = copy.deepcopy(plan); bad["native"]["roles"][role_index]["writable"] = not bad["native"]["roles"][role_index]["writable"]
            with self.assertRaisesRegex(j.Stop, "compact_role_privileges"): j._bindings(bad, post, 1234567890)
        bad = copy.deepcopy(plan); raw = bytearray.fromhex(bad["compact"]["hex"]); raw[-2] ^= 1; bad["compact"]["hex"] = raw.hex(); bad["compact"]["sha256"] = j.sha(raw)
        with self.assertRaisesRegex(j.Stop, "compact_native_binding"): j._bindings(bad, post, 1234567890)

    def test_two_concurrent_sealers_have_one_commit_and_stale_cas_loser(self):
        tail = self.profile(); context = multiprocessing.get_context("fork"); queue = context.Queue()
        children = [context.Process(target=race_seal, args=(str(self.path), tail, queue)) for _ in range(2)]
        for child in children: child.start()
        for child in children: child.join(30); self.assertEqual(child.exitcode, 0)
        outcomes = [queue.get(timeout=1) for _ in children]
        self.assertEqual(sorted(x[0] for x in outcomes), ["ok", "stop"])
        self.assertIn("tail_compare_and_swap", [x[1] for x in outcomes if x[0] == "stop"])

    def test_uncertain_seal_fsync_never_adopts_or_recaptures(self):
        tail = self.profile(); calls = 0; real = j.os.fsync
        def fail_directory(fd):
            nonlocal calls
            calls += 1
            if calls == 2: raise OSError("offline seal directory fsync failure")
            return real(fd)
        with mock.patch.object(j.os, "fsync", fail_directory):
            with self.assertRaisesRegex(j.Stop, "commit_uncertain_no_retry_or_adoption"): j.apply(seal(tail), self.path)
        with self.assertRaisesRegex(j.Stop, "tail_compare_and_swap"): j.apply(seal(tail), self.path)


if __name__ == "__main__": unittest.main()
