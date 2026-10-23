"""Compiled-code reuse retains fresh rule globals and every source check."""
import importlib.util
import os
from pathlib import Path
import tempfile
import unittest
from unittest import mock

HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location("rule_code_cache_operator",HERE/"startup_release_operator.py")
o=importlib.util.module_from_spec(spec);spec.loader.exec_module(o)


class RuleCodeCacheTests(unittest.TestCase):
    def test_compiled_reuse_fresh_globals_and_warm_source_tamper_refusals(self):
        repo=Path(os.environ.get("PIV1_PUBLIC_CHECK_REPO",HERE.parent))
        with mock.patch.dict(os.environ,{"PIV1_PUBLIC_CHECK_REPO":str(repo)}):baseline=o.state_rules()
        pins=baseline.SOURCE_PINS.copy();self.assertEqual(len(pins),11)
        raw=(HERE/"startup_state_rules.py").read_bytes()
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);rules=root/"tools/startup_state_rules.py";rules.parent.mkdir()
            rules.write_bytes(raw)
            for name in pins:
                target=root/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes((repo/name).read_bytes())
            reads={};original_read=Path.read_bytes
            def read(path):reads[str(path)]=reads.get(str(path),0)+1;return original_read(path)
            compiler=mock.Mock(wraps=compile)
            with mock.patch.object(o,"__file__",str(rules.with_name("startup_release_operator.py"))),mock.patch.dict(os.environ,{"PIV1_PUBLIC_CHECK_REPO":str(root)}),mock.patch.object(o,"_STATE_RULE_CODE",{}),mock.patch.object(o,"compile",compiler,create=True),mock.patch.object(Path,"read_bytes",autospec=True,side_effect=read):
                first=o.state_rules();first.SOURCE_PINS.clear();first.PERIOD=-1
                second=o.state_rules()
                self.assertIsNot(first,second)
                self.assertIsNot(first.SOURCE_PINS,second.SOURCE_PINS)
                self.assertEqual(second.SOURCE_PINS,pins);self.assertEqual(second.PERIOD,2592000)
                self.assertEqual(compiler.call_count,1)
                self.assertEqual(set(o._STATE_RULE_CODE),{(str(rules),o.STATE_RULES_SHA)})
                self.assertEqual(reads[str(rules)],2)
                self.assertTrue(all(reads[str(root/name)]==2 for name in pins))
                rules.write_bytes(raw+b'\n# changed after compiled cache warmed\n')
                with self.assertRaisesRegex(o.Stop,"startup_rules_source_pin"):o.state_rules()
                self.assertEqual(compiler.call_count,1)
                rules.write_bytes(raw);o.state_rules()
                self.assertEqual(compiler.call_count,1)
                self.assertTrue(all(reads[str(root/name)]==3 for name in pins))
                name=next(iter(pins));primary=root/name;primary_raw=original_read(primary)
                primary.write_bytes(primary_raw+b'\n// changed primary source\n')
                with self.assertRaisesRegex(ValueError,"startup_rule_source_pin"):o.state_rules()
                self.assertEqual(compiler.call_count,1)
                primary.write_bytes(primary_raw)
                alternate=rules.with_name("same-bytes.py");alternate.write_bytes(raw);rules.unlink();rules.symlink_to(alternate)
                with self.assertRaisesRegex(o.Stop,"startup_rules_source_pin"):o.state_rules()
                self.assertEqual(compiler.call_count,1)


if __name__=="__main__":unittest.main()
