"""Unused-rule loading and equivalent intent bookkeeping; public fixtures only."""
import base64
import copy
import importlib.util
import json
import os
from pathlib import Path
import struct
import tempfile
import unittest
from unittest import mock

HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location("scheduling_manifest_helpers",HERE/"test_startup_manifest.py")
t=importlib.util.module_from_spec(spec);spec.loader.exec_module(t);o=t.o


class ReplaySchedulingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        t.StartupManifestTests.setUpClass();cls.c=t.StartupManifestTests.c

    def test_write_coverage_is_fresh_per_stage_and_rejects_account_and_clock_drift(self):
        full=t.StartupManifestTests(methodName="test_actual_full_949_create_reopen_and_record_bound")
        full.setUp();self.addCleanup(full.doCleanups)
        a=full.a;state=full.state();phase=full.manifest['phases'][1];api=o.operator_api();roles=a.oracle()['roles']
        raw=struct.pack('<QqQQq',100,0,0,0,100)
        clock={'slot':'100','contextSlot':'100','unixTimestamp':'100',
               'dataBase64':base64.b64encode(raw).decode(),'dataSha256':o.sha(raw)}
        with mock.patch.object(a,'expected',wraps=a.expected) as derive:
            pre=a.coverage(api,self.c,state,phase,'PRE',phase['reviewed'],{},clock)
            post=a.coverage(api,self.c,state,phase,'POST',phase['reviewed'],{},clock)
            self.assertEqual([call.args[-1] for call in derive.call_args_list],['PRE','POST'])
        self.assertEqual(int(pre['accountDigests'][roles['g0']]['lamports'])-int(post['accountDigests'][roles['g0']]['lamports']),5000)
        self.assertNotEqual(pre['accountDigests'][roles['loaderBuffer']]['dataSha256'],post['accountDigests'][roles['loaderBuffer']]['dataSha256'])
        original=copy.deepcopy(pre)
        pre['accountDigests'][roles['g0']]['lamports']='0'
        self.assertEqual(a.coverage(api,self.c,state,phase,'PRE',phase['reviewed'],{},clock),original)
        absent={'exists':False,'owner':None,'executable':None,'lamports':None,'dataBase64':None}
        accounts={key:copy.deepcopy(absent) for key in a.required_keys(phase)}
        with self.assertRaisesRegex(ValueError,'startup_mandatory_account_predicate'):
            a.coverage(api,self.c,state,phase,'POST',phase['reviewed'],{},clock,accounts)
        bad=copy.deepcopy(clock);bad['dataSha256']='0'*64
        with self.assertRaisesRegex(ValueError,'startup_saved_clock'):
            a.coverage(api,self.c,state,phase,'POST',phase['reviewed'],{},bad)

    def test_standalone_typed_materialization_rederives_selected_stage_and_facts(self):
        full=t.StartupManifestTests(methodName="test_actual_full_949_create_reopen_and_record_bound")
        full.setUp();self.addCleanup(full.doCleanups)
        a=full.a;state=full.state();phase=copy.deepcopy(full.manifest['phases'][937]);api=o.operator_api();roles=a.oracle()['roles']
        # Caller-supplied maps cannot erase or replace a source-derived rule.
        phase['expectedPoststate']={roles['programData']:copy.deepcopy(a.ABSENT)}
        self.assertEqual(a.exact_typed(api,self.c,state,phase,'PRE',{}),{})
        facts={'deploymentSlot':'55'}
        result=a.exact_typed(api,self.c,state,phase,'POST',facts)
        raw=struct.pack('<IQB',3,55,1)+self.c.b58decode(roles['vault0'],32)+full.image
        self.assertEqual(result,{roles['programData']:a.account(a.LOADER,a.oracle()['rents']['894597'],raw)})
        with self.assertRaisesRegex(ValueError,'startup_deployment_fact_missing'):
            a.exact_typed(api,self.c,state,phase,'POST',{})
        changed=a.exact_typed(api,self.c,state,phase,'POST',{'deploymentSlot':'56'})
        self.assertNotEqual(changed[roles['programData']]['dataSha256'],result[roles['programData']]['dataSha256'])

    def test_unused_rules_are_lazy_but_typed_and_literal_coverage_still_check_sources(self):
        full=t.StartupManifestTests(methodName="test_actual_full_949_create_reopen_and_record_bound")
        full.setUp();self.addCleanup(full.doCleanups)
        a=full.a;state=full.state();phase=full.manifest['phases'][0];api=o.operator_api()
        accessor=mock.Mock(wraps=o.state_rules);api.state_rules=accessor
        self.assertEqual(a.exact_typed(api,self.c,state,phase,'PRE',{}),{})
        accessor.assert_not_called()
        roles=a.oracle()['roles'];facts={'deploymentSlot':'55'}
        result=a.exact_typed(api,self.c,state,full.manifest['phases'][937],'POST',facts)
        self.assertEqual(accessor.call_count,1)
        raw=struct.pack('<IQB',3,55,1)+self.c.b58decode(roles['vault0'],32)+full.image
        self.assertEqual(result,{roles['programData']:a.account(a.LOADER,a.oracle()['rents']['894597'],raw)})
        accessor.reset_mock()
        # The unchanged source-derived phase has two typed descriptors. Storage
        # bytes are irrelevant to this renderer; no native Squads proof is claimed.
        with mock.patch.object(a,'storage_bytes',return_value={}):
            facts['proposal']={'approvedCount':0,'status':1,'timestamp':'100'}
            result=a.exact_typed(api,self.c,state,full.manifest['phases'][943],'POST',facts)
            self.assertEqual(set(result),{roles['programData'],roles['proposal1']})
            self.assertEqual(accessor.call_count,1)
            with self.assertRaisesRegex(ValueError,'startup_deployment_fact_missing'):
                a.exact_typed(api,self.c,state,full.manifest['phases'][937],'POST',{})
            with self.assertRaisesRegex(ValueError,'startup_proposal_fact_missing'):
                a.exact_typed(api,self.c,state,full.manifest['phases'][943],'POST',{'deploymentSlot':'55'})
        # This Clock-only source-load regression uses the unchanged WRITE subset.
        phase=full.manifest['phases'][1]
        clock_raw=struct.pack('<QqQQq',100,0,0,0,100)
        clock={'slot':'100','contextSlot':'100','unixTimestamp':'100',
               'dataBase64':base64.b64encode(clock_raw).decode(),'dataSha256':o.sha(clock_raw)}
        api=o.operator_api();rules=o.state_rules();repo=Path(os.environ.get('PIV1_PUBLIC_CHECK_REPO',HERE.parent))
        rules_raw=(HERE/'startup_state_rules.py').read_bytes()
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);path=root/'tools/startup_state_rules.py';path.parent.mkdir()
            path.write_bytes(rules_raw)
            for name in rules.SOURCE_PINS:
                target=root/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes((repo/name).read_bytes())
            with mock.patch.object(o,'__file__',str(path.with_name('startup_release_operator.py'))),mock.patch.dict(os.environ,{'PIV1_PUBLIC_CHECK_REPO':str(root)}):
                # Literal coverage still reaches its independent fresh Clock
                # rule load, including rules-file and all primary source pins.
                a.coverage(api,self.c,state,phase,'PRE',phase['reviewed'],{},clock)
                path.write_bytes(rules_raw+b'\n# source changed\n')
                with self.assertRaisesRegex(o.Stop,'startup_rules_source_pin'):
                    a.coverage(api,self.c,state,phase,'PRE',phase['reviewed'],{},clock)
                path.write_bytes(rules_raw);primary=root/next(iter(rules.SOURCE_PINS))
                primary.write_bytes(primary.read_bytes()+b'\n// primary changed\n')
                with self.assertRaisesRegex(ValueError,'startup_rule_source_pin'):
                    a.coverage(api,self.c,state,phase,'PRE',phase['reviewed'],{},clock)

    def test_intent_cardinality_matches_scan_and_malformed_terminals_still_refuse(self):
        host=t.h.OperatorTests(methodName='test_nullable_imports_no_helpers')
        host.setUp();self.addCleanup(host.doCleanups);host.ready()
        host.call('SEND',{'id':host.phase['id']},_transport=host.transport,**host.auth())
        host.call('RECOVER',{'id':host.phase['id']},_transport=host.transport,**host.auth())
        records=[json.loads(p.read_bytes()) for p in sorted(o.journal_path(host.base).glob('*.json'))]
        for length in range(1,len(records)+1):
            state=o.replay(host.j,records[:length])
            self.assertLessEqual(state['terminal'].keys(),state['intents'].keys())
            self.assertEqual(any(pid not in state['terminal'] for pid in state['intents']),len(state['intents'])!=len(state['terminal']))
        terminal=records[-1];self.assertEqual(terminal['kind'],'FINALIZED')
        at=next(i for i,r in enumerate(records) if r['kind']=='INTENT')
        malformed=(
            (records+[copy.deepcopy(terminal)],'terminal_order'),
            (records[:at]+[copy.deepcopy(terminal)],'terminal_order'),
            (records[:at]+[{'kind':'UNKNOWN','data':{'id':host.phase['id']}}],'unknown_without_intent'),
            (records[:at+1]+[copy.deepcopy(records[1])],'unresolved_intent_blocks_transition'))
        for chain,reason in malformed:
            with self.subTest(reason=reason),self.assertRaisesRegex(o.Stop,reason):o.replay(host.j,chain)


if __name__=='__main__':unittest.main()
