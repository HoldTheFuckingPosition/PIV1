"""Replay-local rule code; public fixtures and explicit injected host seams.

No native signing, private file, actual RPC or transaction is used. Temporary
copies of pinned public sources exercise persistent provenance tampering.
"""
from contextlib import contextmanager
import copy
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest import mock

HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location("scope_lifetime_helpers",HERE/"test_startup_lifetime_capture.py")
t=importlib.util.module_from_spec(spec);spec.loader.exec_module(t);o=t.o


class ReplayRulesScopeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):t.LifetimeCaptureTests.setUpClass()

    def setUp(self):self.e=t.Environment(self)

    def history(self):
        e=self.e;e.capture();e.signed()
        e.host.call("SEND",{"id":e.pid},_transport=e.transport,**e.host.auth())
        return e.records()

    @contextmanager
    def public_sources(self):
        repo=Path(os.environ.get("PIV1_PUBLIC_CHECK_REPO",HERE.parent));pins=o.state_rules().SOURCE_PINS.copy()
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);rules=root/"tools/startup_state_rules.py";rules.parent.mkdir()
            rules.write_bytes((HERE/"startup_state_rules.py").read_bytes())
            operator=rules.with_name("startup_release_operator.py");operator.write_bytes((HERE/"startup_release_operator.py").read_bytes())
            for name in ("startup_manifest.py","startup_manifest.oracle.json","startup_signer_backend.cjs"):
                rules.with_name(name).write_bytes((HERE/name).read_bytes())
            for name in pins:
                target=root/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes((repo/name).read_bytes())
            with mock.patch.object(o,"__file__",str(operator)),mock.patch.dict(os.environ,{"PIV1_PUBLIC_CHECK_REPO":str(root)}):
                yield root,rules,pins

    def test_one_private_module_per_replay_and_independent_fresh_exit_loader(self):
        e=self.e;records=self.history();expected=o.replay(e.host.j,records);modules=[];original=o.state_rules
        def load():
            module=original();modules.append(module)
            module.saved_clock=mock.Mock(wraps=module.saved_clock)
            module.clock=mock.Mock(wraps=module.clock)
            return module
        with mock.patch.object(o,"state_rules",side_effect=load):
            self.assertEqual(o.replay(e.host.j,records),expected)
            self.assertEqual(len(modules),3)
            # INTENT and coverage share one module; raw PRE Clock validation
            # and the independent exit check each load their own fresh module.
            self.assertEqual(modules[0].saved_clock.call_count,2)
            modules[0].clock.assert_not_called()
            self.assertEqual(modules[1].clock.call_count,1)
            modules[1].saved_clock.assert_not_called()
            modules[2].saved_clock.assert_not_called()
            modules[2].clock.assert_not_called()
            modules[0].SOURCE_PINS.clear();modules[0].PERIOD=-1
            self.assertEqual(o.replay(e.host.j,records),expected)
        self.assertEqual(len(modules),6)
        self.assertEqual(len({id(module) for module in modules}),6)
        for module in modules[1:]:
            self.assertEqual(len(module.SOURCE_PINS),11);self.assertEqual(module.PERIOD,2592000)
        self.assertEqual(modules[3].saved_clock.call_count,2)
        modules[3].clock.assert_not_called()
        self.assertEqual(modules[4].clock.call_count,1)
        modules[4].saved_clock.assert_not_called()
        modules[5].saved_clock.assert_not_called()
        modules[5].clock.assert_not_called()

    def test_persistent_rule_or_primary_tamper_rejects_exit_despite_spoofed_local_pins(self):
        e=self.e;records=self.history();expected=o.replay(e.host.j,records)
        for target_name,reason in (("rules","startup_rules_source_pin"),("primary","startup_rule_source_pin")):
            with self.subTest(target=target_name),self.public_sources() as (root,rules,pins):
                target=rules if target_name=="rules" else root/next(iter(pins));saved=target.read_bytes()
                original=o.state_rules;modules=[];attempts=[];changed=[False];clock_calls=[0]
                def load():
                    attempts.append(True);module=original();modules.append(module)
                    self.assertEqual(len(module.SOURCE_PINS),11);self.assertEqual(module.PERIOD,2592000)
                    if len(modules)==1:
                        saved_clock=module.saved_clock
                        def clock(value):
                            saved_clock(value)
                            clock_calls[0]+=1
                            # Let the independent raw PRE load finish first;
                            # persistent tampering must be caught at replay exit.
                            if clock_calls[0]==2:
                                changed[0]=True;module.SOURCE_PINS.clear();module.PERIOD=-1
                                target.write_bytes(saved+b'\n# persistent public-source change\n')
                        module.saved_clock=clock
                    return module
                with mock.patch.object(o,"state_rules",side_effect=load):
                    with self.assertRaisesRegex(ValueError,reason):o.replay(e.host.j,records)
                    self.assertEqual(clock_calls[0],2)
                    self.assertEqual(len(attempts),3);self.assertEqual(len(modules),2)
                    target.write_bytes(saved)
                    # Failure did not retain either the spoofed module or an
                    # accepted provenance result for the next invocation.
                    self.assertEqual(o.replay(e.host.j,records),expected)
                self.assertEqual(len(attempts),6);self.assertEqual(len(modules),5)
                self.assertEqual(len({id(module) for module in modules}),5)
                self.assertIsNot(modules[0],modules[2]);self.assertEqual(len(modules[2].SOURCE_PINS),11)

    def test_saved_clock_coverage_and_retained_raw_checks_still_refuse(self):
        e=self.e;records=self.history();at=next(i for i,r in enumerate(records) if r["kind"]=="INTENT")
        bad=copy.deepcopy(records);bad[at]["data"]["runtimePreClock"]["slot"]="0"
        with self.assertRaisesRegex(ValueError,"startup_saved_clock"):o.replay(e.host.j,bad)
        bad=copy.deepcopy(records);bad[at]["data"]["coverage"]["accountDigests"][e.ref["roles"]["g0"]]["lamports"]="1"
        with self.assertRaisesRegex(o.Stop,"startup_intent_coverage_binding"):o.replay(e.host.j,bad)
        quote=next(r["data"]["quote"] for r in records if r["kind"]=="PREPARE")
        path=Path(quote["path"])/"rpc-02.response.bin";saved=path.read_bytes()
        try:
            path.write_bytes(saved+b' ')
            with self.assertRaisesRegex(o.Stop,"quote_artifact_hash"):o.replay(e.host.j,records)
        finally:path.write_bytes(saved)
        self.assertEqual(o.replay(e.host.j,records)["reserved"],10000)

    def test_paths_without_rule_use_remain_lazy(self):
        e=self.e
        with mock.patch.object(o,"state_rules",side_effect=AssertionError("unused rules must not load")):
            self.assertEqual(o.replay(e.host.j,e.records())["cursor"],0)
            # Actual generic controller fixtures retain their original shape.
            host=t.t.h.OperatorTests(methodName="test_nullable_imports_no_helpers")
            host.setUp();self.addCleanup(host.doCleanups)
            host.create();self.assertFalse(host.call("INSPECT")["mandatoryStartupManifestConsumed"])

    def test_fsync_and_public_callback_boundaries_start_fresh_replay_scopes(self):
        e=self.e;e.capture();e.signed();modules=[];boundaries=[];original=o.state_rules;write=e.host.j.write_file
        def load():
            module=original()
            self.assertEqual(len(module.SOURCE_PINS),11);self.assertEqual(module.PERIOD,2592000)
            modules.append(module);return module
        def mutate_loaded(label):
            boundaries.append((label,len(modules)))
            for module in modules:module.SOURCE_PINS.clear();module.PERIOD=-1
        def commit(fd,name,raw):
            write(fd,name,raw);record=json.loads(raw)
            if record.get("kind")=="INTENT":mutate_loaded("intent-fsynced")
        def transport(raw):
            methods=[q["method"] for q in json.loads(raw)]
            if "getBlockHeight" in methods and e.records()[-1]["kind"]=="INTENT":mutate_loaded("post-intent-public-callback")
            return e.transport(raw)
        with mock.patch.object(o,"state_rules",side_effect=load),mock.patch.object(e.host.j,"write_file",side_effect=commit):
            result=e.host.call("SEND",{"id":e.pid},_transport=transport,**e.host.auth())
        self.assertEqual([name for name,_ in boundaries],["intent-fsynced","post-intent-public-callback"])
        self.assertTrue(all(len(modules)>count for _,count in boundaries))
        self.assertEqual(len({id(module) for module in modules}),len(modules))
        self.assertEqual(result["status"],"UNKNOWN_OR_SUBMITTED_RECOVERY_ONLY")
        self.assertEqual(e.calls.count("sendTransaction"),1)


if __name__=="__main__":unittest.main()
