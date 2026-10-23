"""Focused selected-startup manifest tests; no private inputs or signing.

The complete-host transport case uses explicit synthetic ELF/account and
first-packet-template seams with one retained historical signed transaction.
It proves mandatory oracle consumption and durable replay, not a real Buffer
create or native startup. Root separately compares the actual selected ELF and
all949 immutable messages against the retained independent wire commitments.
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
spec=importlib.util.spec_from_file_location('manifest_host_helpers',HERE/'test_startup_release_operator.py')
h=importlib.util.module_from_spec(spec);spec.loader.exec_module(h);o=h.o


class StartupManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        h.OperatorTests.setUpClass();cls.c=h.OperatorTests.c;cls.j=h.OperatorTests.j
        cls.a=o.startup_manifest();cls.image=b"\x7f"*894552

    def setUp(self):
        self.stack=__import__('contextlib').ExitStack();self.addCleanup(self.stack.close)
        self.stack.enter_context(mock.patch.object(o,'startup_manifest',return_value=self.a))
        self.stack.enter_context(mock.patch.object(self.a,'ELF_SHA',o.sha(self.image)))
        self.host=h.OperatorTests(methodName='test_nullable_imports_no_helpers')
        with mock.patch.object(h,'CLOCK_FIXTURE',{'preManifest':self.a.oracle()['preManifest']}):self.host.setUp()
        self.addCleanup(self.host.doCleanups)
        self.manifest=self.a.assemble(o.operator_api(),self.j,self.image,package_sha256='c'*64,parent_tail=self.host.manifest['parentTail'],historical_ledger=self.host.manifest['historicalLedger'])
        self.host.manifest=self.manifest

    def state(self,*,seal=None,facts=None):
        return {'manifest':self.manifest,'manifestSha256':o.sha(o.encode(self.manifest)),'seal':seal,'runtimeFacts':facts or {},'prepared':{},'signed':{}}

    def test_actual_full_949_create_reopen_and_record_bound(self):
        initial=self.host.create();self.assertEqual(initial['completedPhases'],0)
        self.assertTrue(initial['mandatoryStartupManifestConsumed'])
        self.assertLess(len(o.encode(self.manifest)),self.j.MAX_INPUT-65536)
        self.assertEqual(self.host.call('INSPECT')['tail'],initial['tail'])
        self.assertEqual(len(self.manifest['phases']),949)
        self.assertEqual(sum(int(p['feeCapLamports']) for p in self.manifest['phases']),4815000)
        self.assertEqual({p.name:p.read_bytes() for p in self.host.base.iterdir()},self.host.base_bytes)

    def test_missing_ordered_phase_and_caller_state_oracle_refused(self):
        for mutate in (lambda m:m['phases'].pop(936),lambda m:m['phases'].__setitem__(1,m['phases'][2]),lambda m:m['phases'][0]['expectedPrestate'].update({'x':{}})):
            bad=copy.deepcopy(self.manifest);mutate(bad)
            with self.assertRaises(ValueError):self.a.validate(o.operator_api(),self.j,bad)
        bad=copy.deepcopy(self.manifest);bad['schema']=o.SCHEMA
        with self.assertRaisesRegex(ValueError,'object_fields'):
            o.manifest_shape(self.c,self.j,bad,o.inspect_parent(self.j,self.host.base,bad['parentTail']))

    def test_each_noninitializer_source_message_and_late_write_are_bound(self):
        for index in (0,1,936,937,938,948):
            bad=copy.deepcopy(self.manifest);review=bad['phases'][index]['reviewed'];raw=bytearray(base64.b64decode(review['messageBase64']));raw[-1]^=1
            review['messageBase64']=base64.b64encode(raw).decode();review['messageSha256']=o.sha(raw)
            with self.assertRaisesRegex(ValueError,'startup_source_message_semantics'):self.a.validate(o.operator_api(),self.j,bad)
        first=self.a.write_review(self.image,1);last=self.a.write_review(self.image,936)
        self.assertEqual(base64.b64decode(first['messageBase64'])[-972:-956],struct.pack('<IIQ',1,0,956))
        self.assertEqual(base64.b64decode(last['messageBase64'])[-708:-692],struct.pack('<IIQ',1,893860,692))
        with self.assertRaisesRegex(ValueError,'startup_phase_fee_profile'):
            bad=copy.deepcopy(self.manifest);bad['phases'][1]['feeCapLamports']='4999';self.a.validate(o.operator_api(),self.j,bad)

    def test_source_fixed_phase_key_sets_and_every_cashflow(self):
        ref=self.a.oracle();r=ref['roles'];state=self.state()
        for index in (0,937,938):
            phase=self.manifest['phases'][index];self.assertEqual(len(self.a.required_keys(phase)),57)
        for index in (1,500,936):
            phase=self.manifest['phases'][index];self.assertEqual(len(self.a.required_keys(phase)),10)
            pre=self.a.expected(o.operator_api(),self.c,state,phase,'PRE');post=self.a.expected(o.operator_api(),self.c,state,phase,'POST')
            self.assertEqual(pre[r['pivProgram']],self.a.ABSENT);self.assertEqual(post[r['programData']],self.a.ABSENT)
            self.assertEqual(int(pre[r['g0']]['lamports'])-int(post[r['g0']]['lamports']),5000)
            self.assertEqual(pre[r['h2']],post[r['h2']]);self.assertNotEqual(pre[r['loaderBuffer']]['dataSha256'],post[r['loaderBuffer']]['dataSha256'])
        expected=[(0,999980000,4696464320),(1,999970000,151301960),(938,995260000,150428200),(939,995250000,149127720),(942,1003794960,132043680),(949,1003734960,102727000)]
        for completed,g,h2 in expected:self.assertEqual(self.a.actor_balances(completed),(g,h2))

    def test_all16_typed_targets_proposal_and_selected_loader_are_mandatory(self):
        ref=self.a.oracle();r=ref['roles'];state=self.state(seal={'placeholder':'state-generates-only-after-binding'})
        # Typed declarations are derived even before inspecting literal target data.
        with mock.patch.object(self.a,'storage_bytes',return_value={}):
            bootstrap=self.a.expected(o.operator_api(),self.c,state,self.manifest['phases'][937],'POST')
            self.assertEqual(bootstrap[r['programData']],{'startupRule':'PROGRAM_DATA','lamports':'4545203000'})
            execute=self.a.expected(o.operator_api(),self.c,state,self.manifest['phases'][947],'POST')
            self.assertEqual([execute[key]['startupRule'] for key in ref['targetAddresses']],['INITIALIZED_TARGET']*16)
            self.assertEqual(execute[r['proposal1']]['startupRule'],'PROPOSAL')
            heartbeat=self.a.expected(o.operator_api(),self.c,state,self.manifest['phases'][948],'POST')
            self.assertEqual(heartbeat[ref['targetAddresses'][3]]['startupRule'],'HEARTBEAT_REWARD_0')
            self.assertEqual(heartbeat[r['vault0']],self.a.ABSENT)

    def test_virtual_instructions_use_exact_compiled_roles(self):
        review=self.manifest['phases'][937]['reviewed'];data=self.a.message_instructions(review,self.c)
        self.assertEqual(struct.unpack_from('<H',data)[0],9)
        self.assertEqual(struct.unpack_from('<H',data,len(data)-2)[0],8)
        raw=bytearray(base64.b64decode(review['messageBase64']));raw[-1]^=1
        bad=copy.deepcopy(review);bad['messageBase64']=base64.b64encode(raw).decode()
        self.assertNotEqual(self.a.message_instructions(bad,self.c),data)
        bad=copy.deepcopy(review);bad['accountRoles'][0]['writable']=False
        self.assertNotEqual(self.a.message_instructions(bad,self.c),data)

    def test_selected_seal_rederives_nine_messages_and_all_bumps(self):
        ref=self.a.oracle();derived=self.j._node_request({'preManifest':ref['preManifest'],'anchor':'1234567890'},planner=True);plan=derived['plan']
        packets=[]
        for packet in plan['packets']:
            raw=base64.b64decode(packet['wireBase64']);at=1+raw[0]*64
            tables=[{'address':plan['alt']['address'],'addresses':plan['alt']['addresses']}] if raw[at]==128 else []
            packets.append({'wireBase64':packet['wireBase64'],'lookupTables':tables})
        described=o.describe(self.j,packets,state_plan=plan,governance=derived['postManifest']['initializer']['governance']);reviews=described.pop('packets')
        data={'anchorUnixTimestamp':'1234567890','reviewedPackets':reviews,'stateBinding':{**described,'plan':plan,'postManifest':derived['postManifest']}}
        self.a.validate_seal(o.operator_api(),self.j,self.state(),data)
        bad=copy.deepcopy(data);bad['stateBinding']['targetBumps'][0]^=1
        with self.assertRaisesRegex(ValueError,'startup_sealed_derived_bumps'):self.a.validate_seal(o.operator_api(),self.j,self.state(),bad)

    def test_exact_fee_profile_rejected_before_prepare_and_import(self):
        self.host.create();phase=self.manifest['phases'][0];review=phase['reviewed'];message=base64.b64decode(review['messageBase64']);signers=len(review['requiredSigners'])
        wire=base64.b64encode(bytes([signers])+bytes(64*signers)+message).decode()
        life={'recentBlockhash':self.c.b58encode(bytes(32)),'currentBlockHeight':'100','lastValidBlockHeight':'1000','minimumRemainingBlockHeights':'40'}
        bad={'id':phase['id'],'wireBase64':wire,'lifetime':life,'feeQuoteLamports':'9999'}
        with self.assertRaisesRegex(ValueError,'startup_exact_fee_profile'):self.host.call('PREPARE',[bad])
        bad['feeQuoteLamports']='10000';self.host.call('PREPARE',[bad]);self.assertEqual(self.host.call('INSPECT')['completedPhases'],0)

    def test_legacy_manifest_cannot_reach_fixed_origin_send(self):
        host=h.OperatorTests(methodName='test_nullable_imports_no_helpers');host.setUp();self.addCleanup(host.doCleanups);host.ready()
        with self.assertRaisesRegex(ValueError,'full_startup_required_for_live_transport'):
            host.call('SEND',{'id':host.phase['id']},live=True,**host.auth())
        host.refresh_tail();self.assertNotIn('sendTransaction',host.calls)

    def test_only_selected_large_image_gets_exact_bounded_normalization(self):
        self.assertEqual(self.c.MAX_ACCOUNT,1048576);key=self.a.large_key();ref=copy.deepcopy(self.a.oracle());data=bytes(1106125)
        ref['baseline'][key]['dataSha256']=o.sha(data)
        value={'owner':ref['baseline'][key]['owner'],'executable':False,'lamports':int(ref['baseline'][key]['lamports']),'data':[base64.b64encode(data).decode(),'base64']}
        with mock.patch.object(self.a,'oracle',return_value=ref):
            actual=self.a.normalize_account(self.c,key,value);self.assertEqual(self.a.digest(o.operator_api(),self.c,key,actual),ref['baseline'][key])
            for raw in (data[:-1],data+b'\0',data[:-1]+b'\1'):
                bad=copy.deepcopy(value);bad['data'][0]=base64.b64encode(raw).decode()
                with self.assertRaises(ValueError):self.a.normalize_account(self.c,key,bad)
            bad=copy.deepcopy(value);bad['owner']=self.a.SYSTEM
            with self.assertRaisesRegex(ValueError,'startup_large_account_identity'):self.a.normalize_account(self.c,key,bad)
            with self.assertRaisesRegex(ValueError,'base64_bound'):self.a.normalize_account(self.c,ref['roles']['squadsProgramData'],value)
        self.assertEqual(self.c.MAX_ACCOUNT,1048576)

    def test_full_send_recover_consume_mandatory_coverage_and_replay(self):
        # Public-byte/hash seams are explicit: source identities/message templates
        # stay selected, while account payloads and ELF are synthetic here.
        ref=copy.deepcopy(self.a.oracle());r=ref['roles'];raw_accounts={}
        for key,digest in ref['baseline'].items():
            if not digest['exists']:raw_accounts[key]={'exists':False,'owner':None,'executable':None,'lamports':None,'dataBase64':None};continue
            data=base64.b64decode(ref['multisigBase64']) if key==r['multisig'] else bytes(digest['dataBytes'])
            raw_accounts[key]={'exists':True,'owner':digest['owner'],'executable':digest['executable'],'lamports':digest['lamports'],'dataBase64':base64.b64encode(data).decode()}
            ref['baseline'][key]=dict(digest,dataSha256=o.sha(data))
        self.stack.enter_context(mock.patch.object(self.a,'oracle',return_value=ref))
        self.stack.enter_context(mock.patch.object(o,'helpers',return_value=(self.host.r,self.host.c,self.host.s,self.host.j)))
        phase=self.manifest['phases'][0];historical_review,life,unsigned=h.phase_from_fixture(self.c,1)
        self.host.prepared={'id':phase['id'],'wireBase64':unsigned,'lifetime':life,'feeQuoteLamports':'10000'}
        # The retained signed message is checked by the unchanged pinned checker;
        # this first-template seam isolates mandatory state integration.
        self.stack.enter_context(mock.patch.object(o,'phase_template',return_value=historical_review))
        self.host.create();self.host.call('PREPARE',[self.host.prepared])
        with self.assertRaisesRegex(o.Stop,'signing_response_required'):
            self.host.call('IMPORT',[{'id':phase['id'],'wireBase64':self.host.tx['signedWireBase64']}])
        handoff=self.host.call('HANDOFF',[phase['id']],**self.host.auth())
        response={'schema':'piv1-startup-signing-response-v1','requestId':handoff['request']['requestId'],'packets':[{'id':phase['id'],'wireBase64':self.host.tx['signedWireBase64']}]}
        self.host.call('IMPORT',response,**self.host.auth())
        slot=int(self.host.tx['slot']);bad_pre=[True];bad_post=[False];missing_keys=[False];version_drift=[False];large_drift=[False];sent=[False]
        def transport(raw):
            replies=[]
            for query in json.loads(raw):
                method=query['method']
                if method=='getVersion':value={**ref['version'],'solana-core':'changed'} if version_drift[0] else ref['version']
                elif method=='getMinimumBalanceForRentExemption':value=int(ref['rents'][str(query['params'][0])])
                elif method=='getMultipleAccounts':
                    accounts=copy.deepcopy(raw_accounts);context=slot+1 if sent[0] else slot-1
                    accounts[r['clock']]={'exists':True,'owner':'Sysvar1111111111111111111111111111111111111','executable':False,'lamports':ref['baseline'][r['clock']]['lamports'],'dataBase64':base64.b64encode(struct.pack('<QqQQq',context,0,0,0,100)).decode()}
                    if sent[0]:
                        g,h2=self.a.actor_balances(1);accounts[r['g0']]['lamports']=str(g);accounts[r['h2']]['lamports']=str(h2)
                        data=struct.pack('<IB',1,1)+self.c.b58decode(r['g0'],32)+bytes(len(self.image))
                        accounts[r['loaderBuffer']]={'exists':True,'owner':self.a.LOADER,'executable':False,'lamports':ref['rents']['894589'],'dataBase64':base64.b64encode(data).decode()}
                    if large_drift[0]:
                        key=r['stakePoolProgramData'];data=bytearray(base64.b64decode(accounts[key]['dataBase64']));data[-1]^=1;accounts[key]['dataBase64']=base64.b64encode(data).decode()
                    if bad_pre[0] or bad_post[0]:accounts[r['htfpRecipient']]={'exists':True,'owner':self.a.SYSTEM,'executable':False,'lamports':'1','dataBase64':''}
                    value={'context':{'slot':context},'value':[self.host.wire_account(accounts[key]) for key in query['params'][0]]}
                    if missing_keys[0]:value['value'].pop()
                else:
                    if method=='sendTransaction':sent[0]=True
                    _,_,body=self.host.transport(o.encode([query]));replies.extend(json.loads(body));continue
                self.host.calls.append(method);replies.append({'jsonrpc':'2.0','id':query['id'],'result':value})
            return 200,[],o.encode(replies)
        with self.assertRaisesRegex(ValueError,'prestate_mismatch'):
            self.host.call('SEND',{'id':phase['id']},_transport=transport,**self.host.auth())
        self.host.refresh_tail();self.assertNotIn('sendTransaction',self.host.calls);bad_pre[0]=False;missing_keys[0]=True
        with self.assertRaisesRegex(ValueError,'prestate_context'):
            self.host.call('SEND',{'id':phase['id']},_transport=transport,**self.host.auth())
        self.host.refresh_tail();missing_keys[0]=False;version_drift[0]=True
        self.assertFalse(any(json.loads(p.read_bytes())['kind']=='INTENT' for p in o.journal_path(self.host.base).glob('*.json')))
        with self.assertRaisesRegex(ValueError,'startup_runtime_version_changed'):
            self.host.call('SEND',{'id':phase['id']},_transport=transport,**self.host.auth())
        self.host.refresh_tail();version_drift[0]=False
        self.host.call('SEND',{'id':phase['id']},_transport=transport,**self.host.auth());bad_post[0]=True
        with self.assertRaisesRegex(ValueError,'poststate_mismatch'):
            self.host.call('RECOVER',{'id':phase['id']},_transport=transport,**self.host.auth())
        self.host.refresh_tail();bad_post[0]=False;large_drift[0]=True
        with self.assertRaisesRegex(ValueError,'startup_large_account_complete_image'):
            self.host.call('RECOVER',{'id':phase['id']},_transport=transport,**self.host.auth())
        self.host.refresh_tail();large_drift[0]=False
        report=self.host.call('RECOVER',{'id':phase['id']},_transport=transport,**self.host.auth());self.assertEqual(report['completedPhases'],1)
        records=[json.loads(p.read_bytes()) for p in sorted(o.journal_path(self.host.base).glob('*.json'))]
        intent=next(row for row in records if row['kind']=='INTENT');self.assertEqual(len(intent['data']['coverage']['requiredKeys']),57)
        self.assertEqual(intent['data']['coverage']['virtualInstructionsContract'],'NOT_REFERENCED_NOT_INSTANTIATED_RPC_ABSENT')
        bad=copy.deepcopy(records);terminal=next(row for row in bad if row['kind']=='FINALIZED');terminal['data']['coverage']['accountDigests'].pop(r['h2'])
        with self.assertRaisesRegex(ValueError,'startup_terminal_coverage_binding'):o.replay(self.j,bad)
        self.assertEqual(self.host.calls.count('sendTransaction'),1)
        terminal=next(row for row in records if row['kind']=='FINALIZED');receipt=terminal['data']['fullSnapshot']
        self.assertEqual(receipt['contextSlot'],terminal['data']['contextSlot']);self.assertEqual(len(receipt['requiredKeys']),57)
        bad=copy.deepcopy(records);terminal=next(row for row in bad if row['kind']=='FINALIZED');terminal['data']['fullSnapshot']['proofSubsetManifestSha256']='f'*64
        with self.assertRaisesRegex(ValueError,'startup_full_snapshot_proof_binding'):o.replay(self.j,bad)


if __name__=='__main__':unittest.main()
