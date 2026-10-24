"""Focused offline checks for replay-local source work and phase lookup.

Only public temporary source/journal bytes and synthetic host declarations are
used. No keys, signatures, native backend, RPC or lifetime trial are involved.
"""
import base64
import copy
import importlib.util
import json
import os
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock


HERE = Path(__file__).parent
SPEC = importlib.util.spec_from_file_location("replay_source_work_operator", HERE / "startup_release_operator.py")
o = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(o)


class ReplaySourceWorkTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.operator = self.root / "startup_release_operator.py"
        self.write(self.operator, b"public operator source\n")
        self.j = SimpleNamespace(parse=json.loads, digest=lambda value:value, u64=int)

    @staticmethod
    def write(path, raw):
        with path.open("wb") as stream:
            stream.write(raw); stream.flush(); os.fsync(stream.fileno())

    def authorization(self, *, expires="18446744073709551615"):
        state = {"manifestSha256":"manifest", "manifest":{"parentTail":"parent", "proposedFeeBudget":{"namespace":"budget"}}}
        raw = o.encode({"schema":"piv1-launcher-release-authorization-v1", "manifestSha256":"manifest", "baseParentTail":"parent",
            "operatorSourceSha256":o.sha(self.operator.read_bytes()), "expiresUnixTimestamp":expires,
            "scope":"EXACT_BOUNDED_TESTNET_RELEASE", "feeBudgetNamespace":"budget"})
        return state, raw, o.sha(raw)

    def phases(self):
        return [{"id":pid, "kind":"BUFFER_CREATE", "reviewed":{"requiredSigners":["signer"], "messageSha256":pid},
            "minimumRemainingBlockHeights":"40", "feeCapLamports":fee} for pid,fee in (("first","5"),("second","7"))]

    def prepared(self, phases):
        return [{"id":p["id"], "reviewed":copy.deepcopy(p["reviewed"]), "wireBase64":base64.b64encode(b"public unsigned host bytes").decode(),
            "feeQuoteLamports":p["feeCapLamports"], "lifetime":{"minimumRemainingBlockHeights":"40", "recentBlockhash":"original", "lastValidBlockHeight":"100", "currentBlockHeight":"50"}} for p in phases]

    def prepare_state(self, phases):
        return {"manifest":{"phases":phases,"parentTail":"parent"}, "manifestSha256":"manifest", "cursor":0,
            "prepared":{}, "preparedAt":{}, "signed":{}, "intents":{}, "quoteByPhase":{}, "handoff":None,
            "baseParentPath":str(self.root/"base"), "seal":None}

    def quote(self, state):
        root = o.journal_path(state["baseParentPath"])
        root = root.with_name(root.name+".evidence")
        return {"schema":"piv1-startup-lifetime-quote-v1", "path":str(root/"00000001-prepare-00"),
            "operatorSequence":"1", "operatorTail":"previous", "operatorSourceSha256":o.sha(self.operator.read_bytes()),
            "baseParentPath":state["baseParentPath"], "baseParentTail":"parent", "manifestSha256":"manifest", "authorization":{},
            "authorizationSha256":"auth", "minimumContextSlot":"0", "transportMode":"OFFLINE_INJECTED_TEST",
            "orderedPreparedSha256":[], "manifestSha256Receipt":"receipt"}

    def test_equal_bytes_share_only_first_digest_and_changed_bytes_are_not_adopted(self):
        original, changed = b"first public bytes", b"different public bytes"
        expected = {raw:o.sha(raw) for raw in (original,changed)}
        with mock.patch.object(o,"sha",wraps=o.sha) as actual:
            hashers = o._replay_source_hashers()
            for hasher in hashers:
                self.assertEqual(hasher(original),expected[original])
                self.assertEqual(hasher(bytes(bytearray(original))),expected[original])
                self.assertEqual(hasher(changed),expected[changed])
                self.assertEqual(hasher(changed),expected[changed])
                self.assertEqual(hasher(original),expected[original])
            self.assertEqual(actual.call_count,9)
            fresh = o._replay_source_hashers()
            self.assertEqual(fresh[0](original),expected[original])
            self.assertEqual(actual.call_count,10)

    def test_authorization_freshly_reads_equal_changed_and_restored_source(self):
        state, raw, pinned = self.authorization()
        original = self.operator.read_bytes()
        hashers = o._replay_source_hashers()
        read = Path.read_bytes
        with mock.patch.object(o,"__file__",str(self.operator)), mock.patch.object(Path,"read_bytes",autospec=True,side_effect=read) as reads:
            self.assertEqual(o.authorization(self.j,state,raw,pinned,_replay_source_sha=hashers),pinned)
            self.assertEqual(o.authorization(self.j,state,raw,pinned,_replay_source_sha=hashers),pinned)
            self.write(self.operator,b"changed public operator source\n")
            with self.assertRaisesRegex(o.Stop,"launcher_authorization_binding"):
                o.authorization(self.j,state,raw,pinned,_replay_source_sha=hashers)
            self.write(self.operator,original)
            self.assertEqual(o.authorization(self.j,state,raw,pinned,_replay_source_sha=hashers),pinned)
            self.assertEqual([call.args[0] for call in reads.call_args_list],[self.operator]*4)

    def test_authorization_retains_prior_binding_failure_and_current_expiry(self):
        state, raw, pinned = self.authorization(expires="0")
        for hashers in (None,o._replay_source_hashers()):
            bad = copy.deepcopy(state); bad["manifestSha256"] = "other"
            with mock.patch.object(Path,"read_bytes",side_effect=AssertionError("source read after earlier refusal")):
                with self.assertRaisesRegex(o.Stop,"launcher_authorization_binding"):
                    o.authorization(self.j,bad,raw,pinned,_replay_source_sha=hashers)
            with mock.patch.object(o,"__file__",str(self.operator)):
                with self.assertRaisesRegex(o.Stop,"launcher_authorization_expired"):
                    o.authorization(self.j,state,raw,pinned,_replay_source_sha=hashers)
                self.assertEqual(o.authorization(self.j,state,raw,pinned,allow_expired=True,_replay_source_sha=hashers),pinned)

    def test_manifest_and_backend_keep_fresh_reads_pin_failures_and_symlink_order(self):
        sentinel = object(); hashers = o._replay_source_hashers()
        cases = (("startup_manifest.py","MANIFEST_SHA",o.startup_manifest,"startup_manifest_source_pin",sentinel),
            ("startup_signer_backend.cjs","SIGNER_BACKEND_SHA",o.signer_backend_source,"signer_backend_source_pin",None))
        with mock.patch.object(o,"__file__",str(self.operator)), mock.patch.object(o,"_MANIFEST_MODULE",sentinel):
            for name,pin_name,check,reason,result in cases:
                with self.subTest(source=name):
                    path = self.root/name; original = b"public pinned source "+name.encode()
                    self.write(path,original)
                    with mock.patch.object(o,pin_name,o.sha(original)):
                        self.assertEqual(check(_replay_source_sha=hashers),path if result is None else result)
                        self.write(path,original+b"changed")
                        with self.assertRaisesRegex(o.Stop,reason): check(_replay_source_sha=hashers)
                        self.write(path,original)
                        self.assertEqual(check(_replay_source_sha=hashers),path if result is None else result)
                        path.unlink(); path.symlink_to(self.operator)
                        with mock.patch.object(Path,"read_bytes",side_effect=AssertionError("symlink was read")):
                            with self.assertRaisesRegex(o.Stop,reason): check(_replay_source_sha=hashers)

    def test_full_replay_is_fresh_after_durable_append_callback_and_returns_same_state(self):
        path = self.root/"startup_manifest.py"; source = b"public selected manifest source\n"
        self.write(path,source)
        phases = self.phases(); entries = self.prepared(phases)
        manifest = {"schema":"piv1-selected-startup-release-manifest-v1", "phases":phases,
            "historicalLedger":{}, "proposedFeeBudget":{"feeCapLamports":"20"}, "parentTail":"parent"}
        first = {"kind":"CREATE", "digest":"a"*64, "data":{"manifest":manifest,"historicalLedger":{},
            "contextFloor":"0", "lastFinalizedSlot":"0", "baseParentPath":str(self.root/"base")}}
        adapter = SimpleNamespace(validate=mock.Mock())
        def durable_callback(fd,name,raw):
            self.write(self.root/name,raw)
            self.write(path,source+b"changed after durable write")
        j = SimpleNamespace(u64=int,MAX_INPUT=1024*1024,write_file=durable_callback)
        journal = object.__new__(o.Journal)
        journal.j=j; journal.chain=[first]; journal.fd=None; journal.total=0; journal.identity={}; journal.stable=mock.Mock()
        factory = o._replay_source_hashers
        with mock.patch.object(o,"__file__",str(self.operator)), mock.patch.object(o,"MANIFEST_SHA",o.sha(source)), \
                mock.patch.object(o,"_MANIFEST_MODULE",adapter), mock.patch.object(o,"helpers",return_value=(None,SimpleNamespace(),None,j)), \
                mock.patch.object(o,"semantic",side_effect=lambda value:value), mock.patch.object(o,"_replay_source_hashers",wraps=factory) as factories:
            journal.append("PREPARE",entries)
            self.assertTrue((self.root/"00000001.json").is_file())
            with self.assertRaisesRegex(o.Stop,"startup_manifest_source_pin"): o.replay(j,journal.chain)
            self.write(path,source)
            actual = o.replay(j,journal.chain)
            self.assertEqual(factories.call_count,3)
            with mock.patch.object(o,"_replay_source_hashers",return_value=(o.sha,o.sha,o.sha)):
                baseline = o.replay(j,journal.chain)
            self.assertEqual(actual,baseline)
            self.assertEqual(o.encode(actual),o.encode(baseline))
            self.assertEqual(actual["prepared"],{p["id"]:p for p in entries})

    def test_phase_index_keeps_prepared_state_signing_request_and_order_identical(self):
        phases = self.phases(); entries = self.prepared(phases); ids = [p["id"] for p in phases]
        states = []
        with mock.patch.object(o,"semantic",side_effect=lambda value:value):
            for positions in (None,{pid:index for index,pid in enumerate(ids)}):
                state = self.prepare_state(phases)
                o.replay_prepare(None,self.j,state,state["manifest"],ids,{"digest":"prepared"},entries,True,_phase_positions=positions)
                request = o.signing_request(SimpleNamespace(GENESIS="genesis"),state,"previous",ids,"auth",_phase_positions=positions)
                with self.assertRaisesRegex(o.Stop,"handoff_order"):
                    o.signing_request(SimpleNamespace(GENESIS="genesis"),state,"previous",list(reversed(ids)),"auth",_phase_positions=positions)
                states.append((state,request))
        self.assertEqual(states[0],states[1])

    def test_quote_lookup_retains_unknown_unhashable_and_missing_id_failures(self):
        state = self.prepare_state(self.phases()); quote = self.quote(state)
        record = {"sequence":"1","previousDigest":"previous"}
        for phases,entry in ((state["manifest"]["phases"],{"id":"unknown"}), (state["manifest"]["phases"],{"id":[]}),
                (state["manifest"]["phases"],{}),([],{})):
            with self.subTest(entry=entry,empty=not phases):
                state["manifest"]["phases"] = phases
                failures=[]
                for positions in (None,{"first":0,"second":1}):
                    with mock.patch.object(o,"__file__",str(self.operator)):
                        try: o.verify_quote(None,self.j,state,[entry],quote,record,_replay_source_sha=o._replay_source_hashers(),_phase_positions=positions)
                        except Exception as exc: failures.append((type(exc),str(exc)))
                self.assertEqual(len(failures),2)
                self.assertEqual(failures[0],failures[1])
                self.assertIn(failures[0][0],(StopIteration,KeyError))
        quote["operatorTail"]="wrong"
        with mock.patch.object(Path,"read_bytes",side_effect=AssertionError("source read after predecessor refusal")):
            with self.assertRaisesRegex(o.Stop,"quote_source_or_predecessor_binding"):
                o.verify_quote(None,self.j,state,[],quote,record,_replay_source_sha=o._replay_source_hashers(),_phase_positions={})

    def test_indexed_quote_retains_selected_fee_and_original_lifetime_margin(self):
        phases=self.phases(); state=self.prepare_state(phases); state["floor"]=10
        entry=self.prepared(phases)[1]
        c=SimpleNamespace(GENESIS="genesis",u64=int,b58decode=lambda text,size:bytes(size))
        initial=["genesis",{"context":{"slot":100},"value":{"blockhash":"original","lastValidBlockHeight":100}}]
        good=[{"context":{"slot":110},"value":7},50,{"context":{"slot":120},"value":True}]
        for positions in (None,{"first":0,"second":1}):
            with mock.patch.object(o,"rebind_unsigned",return_value=(entry["wireBase64"],entry["reviewed"])):
                self.assertEqual(o.quote_values(c,self.j,state,[entry],initial,good,_phase_positions=positions),120)
                bad=copy.deepcopy(good); bad[0]["value"]=5
                with self.assertRaisesRegex(o.Stop,"quote_fee_mismatch"):
                    o.quote_values(c,self.j,state,[entry],initial,bad,_phase_positions=positions)
                bad=copy.deepcopy(good); bad[1]=61; late=copy.deepcopy(entry); late["lifetime"]["currentBlockHeight"]="61"
                with self.assertRaisesRegex(o.Stop,"quote_lifetime_margin"):
                    o.quote_values(c,self.j,state,[late],initial,bad,_phase_positions=positions)


if __name__ == "__main__": unittest.main()
