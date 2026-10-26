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
        profiles=self.a.jito_profiles()
        for name,data in profiles.items():
            key=r[name];raw_accounts[key]['dataBase64']=base64.b64encode(data).decode();ref['baseline'][key]['dataSha256']=o.sha(data)
        key=r['validatorList'];data=bytearray(ref['baseline'][key]['dataBytes']);data[0]=2;struct.pack_into('<II',data,1,10000,1)
        raw_accounts[key]['dataBase64']=base64.b64encode(data).decode();ref['baseline'][key]['dataSha256']=o.sha(data)
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
        _,pool_counters=self.a.jito_pool(profiles['stakePool']);fixture_epoch=pool_counters['lastUpdateEpoch']
        slot=int(self.host.tx['slot']);bad_pre=[True];bad_post=[False];missing_keys=[False];version_drift=[False];large_drift=[False];sent=[False]
        def transport(raw):
            replies=[]
            for query in json.loads(raw):
                method=query['method']
                if method=='getVersion':value={**ref['version'],'solana-core':'changed'} if version_drift[0] else ref['version']
                elif method=='getMinimumBalanceForRentExemption':value=int(ref['rents'][str(query['params'][0])])
                elif method=='getMultipleAccounts':
                    accounts=copy.deepcopy(raw_accounts);context=slot+1 if sent[0] else slot-1
                    accounts[r['clock']]={'exists':True,'owner':'Sysvar1111111111111111111111111111111111111','executable':False,'lamports':ref['baseline'][r['clock']]['lamports'],'dataBase64':base64.b64encode(struct.pack('<QqQQq',context,0,fixture_epoch,0,100)).decode()}
                    for name,offset in (('stakePool',258),('poolMint',36),('managerFeeAccount',64)):
                        key=r[name];raw=bytearray(base64.b64decode(accounts[key]['dataBase64']));value=struct.unpack_from('<Q',raw,offset)[0]
                        struct.pack_into('<Q',raw,offset,value+(2 if name=='stakePool' else -1));accounts[key]['dataBase64']=base64.b64encode(raw).decode()
                    accounts[r['reserveStake']]['lamports']=str(int(accounts[r['reserveStake']]['lamports'])-7)
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

        # Only the original preflight prefix controls INTENT replay. Raw file
        # tampering, even with a forged receipt hash, cannot select new policy.
        index=next(i for i,row in enumerate(records) if row['kind']=='INTENT')
        intent=records[index]['data'];path=Path(intent['evidencePath'])
        for filename,receipt_key,change,reason in (
            ('rpc-02.request.json','requestSha256',lambda value:value[0]['params'][0].reverse(),'startup_preflight_request_binding'),
            ('rpc-02.response.bin','responseSha256',lambda value:value[0]['result']['context'].__setitem__('slot',slot),'startup_preflight_context_binding'),
        ):
            original=(path/filename).read_bytes();value=json.loads(original);change(value);modified=o.encode(value)
            bad=copy.deepcopy(records);bad[index]['data']['coverage']['preflightReceipt'][receipt_key]=o.sha(modified)
            (path/filename).write_bytes(modified)
            try:
                with self.assertRaisesRegex(ValueError,'startup_preflight_raw_hash'):o.replay(self.j,records)
                with self.assertRaisesRegex(ValueError,reason):o.replay(self.j,bad)
            finally:(path/filename).write_bytes(original)
        original=(path/'rpc-02.response.bin').read_bytes();value=json.loads(original)
        clock_index=self.a.required_keys(phase).index(r['clock']);wire=value[0]['result']['value'][clock_index]
        raw=bytearray(base64.b64decode(wire['data'][0]));raw[-1]^=1;wire['data'][0]=base64.b64encode(raw).decode();modified=o.encode(value)
        bad=copy.deepcopy(records);bad[index]['data']['coverage']['preflightReceipt']['responseSha256']=o.sha(modified)
        (path/'rpc-02.response.bin').write_bytes(modified)
        try:
            with self.assertRaisesRegex(ValueError,'startup_preflight_clock_binding'):o.replay(self.j,bad)
        finally:(path/'rpc-02.response.bin').write_bytes(original)
        self.assertEqual(self.host.call('INSPECT')['completedPhases'],1)
        # A self-consistent saved snapshot/manifest cannot adopt fresh mutable
        # bytes which are absent from the original retained raw POST response.
        terminal_index=next(i for i,row in enumerate(records) if row['kind']=='FINALIZED')
        receipt=records[terminal_index]['data']['fullSnapshot'];post=Path(receipt['path'])
        original_snapshot=(post/'snapshot.json').read_bytes();original_manifest=(post/'manifest.json').read_bytes()
        snapshot=json.loads(original_snapshot);key=r['managerFeeAccount'];raw=bytearray(base64.b64decode(snapshot['accounts'][key]['dataBase64']))
        struct.pack_into('<Q',raw,64,struct.unpack_from('<Q',raw,64)[0]+1);snapshot['accounts'][key]['dataBase64']=base64.b64encode(raw).decode();snapshot_raw=o.encode(snapshot)
        manifest=json.loads(original_manifest);manifest['binding']['snapshotSha256']=o.sha(snapshot_raw)
        for artifact in manifest['artifacts']:
            if artifact['name']=='snapshot.json':artifact['sha256']=o.sha(snapshot_raw);artifact['bytes']=len(snapshot_raw)
        manifest_raw=o.encode(manifest);bad=copy.deepcopy(records);bad_receipt=bad[terminal_index]['data']['fullSnapshot']
        bad_receipt['snapshotSha256']=o.sha(snapshot_raw);bad_receipt['manifestSha256']=o.sha(manifest_raw)
        (post/'snapshot.json').write_bytes(snapshot_raw);(post/'manifest.json').write_bytes(manifest_raw)
        try:
            with self.assertRaisesRegex(ValueError,'startup_full_snapshot_raw_account_binding'):o.replay(self.j,bad)
        finally:(post/'snapshot.json').write_bytes(original_snapshot);(post/'manifest.json').write_bytes(original_manifest)


class JitoFreshnessTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        h.OperatorTests.setUpClass();cls.c=h.OperatorTests.c;cls.a=o.startup_manifest()

    def setUp(self):
        self.ref=self.a.oracle();self.r=self.ref['roles'];self.phase={'kind':'BUFFER_CREATE'};self.accounts={}
        for name,data in self.a.jito_profiles().items():
            digest=self.ref['baseline'][self.r[name]]
            self.accounts[self.r[name]]={'exists':True,'owner':digest['owner'],'executable':False,'lamports':digest['lamports'],'dataBase64':base64.b64encode(data).decode()}
        name='validatorList';digest=self.ref['baseline'][self.r[name]];data=bytearray(digest['dataBytes']);data[0]=2;struct.pack_into('<II',data,1,10000,1)
        self.accounts[self.r[name]]={'exists':True,'owner':digest['owner'],'executable':False,'lamports':digest['lamports'],'dataBase64':base64.b64encode(data).decode()}
        raw=struct.pack('<QqQQq',120,0,2000,2000,100)
        self.clock={'contextSlot':'120','slot':'120','unixTimestamp':'100','dataBase64':base64.b64encode(raw).decode(),'dataSha256':o.sha(raw)}

    def raw(self,name):return bytearray(base64.b64decode(self.accounts[self.r[name]]['dataBase64']))
    def put(self,name,raw):self.accounts[self.r[name]]['dataBase64']=base64.b64encode(raw).decode()
    def verify(self):return self.a.jito_digests(self.c,self.phase,self.accounts,self.clock)

    def test_five_named_rules_accept_only_current_raw_counters(self):
        before=self.verify();self.assertEqual(len(before),5)
        raw=self.raw('stakePool')
        for offset in (258,266,419,427):struct.pack_into('<Q',raw,offset,struct.unpack_from('<Q',raw,offset)[0]+1)
        struct.pack_into('<Q',raw,274,2000);self.put('stakePool',raw)
        for name,offset in (('poolMint',36),('managerFeeAccount',64)):
            raw=self.raw(name);struct.pack_into('<Q',raw,offset,struct.unpack_from('<Q',raw,offset)[0]+1);self.put(name,raw)
        self.accounts[self.r['reserveStake']]['lamports']='999999999999'
        # Accepted opaque current validator membership/order/status/unused bytes
        # remain fully observed. This is no permissionless withdrawal decision.
        raw=self.raw('validatorList');raw[9+41]=123;raw[-1]=77;struct.pack_into('<I',raw,5,2);self.put('validatorList',raw)
        after=self.verify();self.assertEqual(set(before),set(after))
        self.assertTrue(all(before[key]!=after[key] for key in before))
        for name in self.a.JITO_KINDS:
            key=self.r[name];self.assertEqual(after[key]['dataSha256'],o.sha(base64.b64decode(self.accounts[key]['dataBase64'])))
        self.assertEqual(self.a.jito_digests(self.c,{'kind':'UPLOAD_WRITE'},None,None),{})

    def test_static_policy_authorities_fees_options_and_tails_fail_closed(self):
        original=copy.deepcopy(self.accounts)
        changes={'stakePool':(1,65,98,282,330,346,347,348,435,610),'poolMint':(4,44,45,46,81),'managerFeeAccount':(0,32,72,108,109,121,129,164),'reserveStake':(0,4,12,44,76,124,199)}
        for name,offsets in changes.items():
            for offset in offsets:
                with self.subTest(name=name,offset=offset):
                    self.accounts=copy.deepcopy(original);raw=self.raw(name);raw[offset]^=1;self.put(name,raw)
                    with self.assertRaises(ValueError):self.verify()
        self.accounts=copy.deepcopy(original)
        for name in self.a.JITO_KINDS:
            for field,value in (('owner',self.a.SYSTEM),('executable',True),('exists',False)):
                with self.subTest(name=name,field=field):
                    self.accounts=copy.deepcopy(original);self.accounts[self.r[name]][field]=value
                    with self.assertRaisesRegex(ValueError,'startup_jito_account_identity'):self.verify()
        self.accounts=copy.deepcopy(original);self.accounts[self.r['poolMint']]['lamports']='1'
        with self.assertRaisesRegex(ValueError,'startup_jito_account_lamports'):self.verify()

    def test_burn_lag_epoch_reserve_floor_and_geometry(self):
        _,pool=self.a.jito_pool(self.raw('stakePool'));raw=self.raw('poolMint');struct.pack_into('<Q',raw,36,pool['poolTokenSupply']-1);self.put('poolMint',raw);self.verify()
        struct.pack_into('<Q',raw,36,pool['poolTokenSupply']+1);self.put('poolMint',raw)
        with self.assertRaisesRegex(ValueError,'startup_jito_mint_security_or_supply'):self.verify()
        struct.pack_into('<Q',raw,36,pool['poolTokenSupply']);self.put('poolMint',raw)
        raw=self.raw('stakePool');struct.pack_into('<Q',raw,274,2001);self.put('stakePool',raw)
        with self.assertRaisesRegex(ValueError,'startup_jito_future_epoch'):self.verify()
        struct.pack_into('<Q',raw,274,2000);self.put('stakePool',raw)
        rent=struct.unpack_from('<Q',self.raw('reserveStake'),4)[0];self.accounts[self.r['reserveStake']]['lamports']=str(rent);self.verify()
        self.accounts[self.r['reserveStake']]['lamports']=str(rent-1)
        with self.assertRaisesRegex(ValueError,'startup_jito_reserve_rent'):self.verify()
        self.accounts[self.r['reserveStake']]['lamports']=str(rent)
        original=self.raw('validatorList')
        for offset,fmt,value in ((0,'<B',1),(1,'<I',9999),(5,'<I',10001)):
            raw=bytearray(original);struct.pack_into(fmt,raw,offset,value);self.put('validatorList',raw)
            with self.assertRaisesRegex(ValueError,'startup_jito_validator_list_geometry'):self.verify()
        self.put('validatorList',original[:-1])
        with self.assertRaisesRegex(ValueError,'startup_jito_account_layout'):self.verify()

    def test_semantic_source_and_original_profile_are_freshly_pinned(self):
        with mock.patch.object(self.a,'JITO_SOURCE_PINS',{'programs/piv1/src/integrations/jito_identity.rs':'0'*64}):
            with self.assertRaisesRegex(ValueError,'startup_jito_semantic_source_pin'):self.verify()
        profiles=copy.deepcopy(self.a.JITO_PROFILE_BASE64);raw=bytearray(base64.b64decode(profiles['poolMint']));raw[4]^=1;profiles['poolMint']=base64.b64encode(raw).decode()
        with mock.patch.object(self.a,'JITO_PROFILE_BASE64',profiles):
            with self.assertRaisesRegex(ValueError,'startup_jito_original_profile'):self.verify()


class CandidateManifestTests(unittest.TestCase):
    """Pure candidate APIs use the real pinned source loader, not parent journals."""

    @classmethod
    def setUpClass(cls):
        cls.a = o.startup_manifest()

    def test_complete_finite_domain_has_contiguous_upload_and_initializer_order(self):
        from collections import Counter
        phases = [self.a.candidate_phase(index) for index in range(947)]
        self.assertEqual(Counter(p['kind'] for p in phases), {
            'BUFFER_CREATE': 1, 'UPLOAD_WRITE': 934, 'BOOTSTRAP': 1,
            'RECIPIENTS': 1, 'INITIALIZER': 9, 'HEARTBEAT': 1})
        writes = [p for p in phases if p['kind'] == 'UPLOAD_WRITE']
        end = 0
        for phase in writes:
            self.assertEqual(phase['writeOffset'], end)
            self.assertTrue(0 < phase['writeBytes'] <= 956)
            end += phase['writeBytes']
        self.assertEqual(end, self.a.candidate_profile()['artifact']['bytes'])
        self.assertEqual(writes[-1]['writeBytes'], 404)
        self.assertEqual([p['initializerIndex'] for p in phases if p['kind'] == 'INITIALIZER'], list(range(9)))
        self.assertEqual([self.a.candidate_phase_index(p) for p in phases], list(range(947)))
        self.assertEqual(len({p['id'] for p in phases}), 947)

    def test_fixed_identity_cannot_be_overridden_or_mutated_through_return_values(self):
        profile = self.a.candidate_profile()
        self.a.validate_candidate_profile(profile)
        self.assertFalse(self.a.full(profile))
        for mutate in (
                lambda p: p['artifact'].update(sha256=self.a.ELF_SHA),
                lambda p: p['artifact'].update(bytes=892352.0),
                lambda p: p['loaderBuffer'].update(address=self.a.oracle()['roles']['loaderBuffer']),
                lambda p: p['roles'].update(htfpRecipient=self.a.oracle()['roles']['htfpRecipient']),
                lambda p: p['initializer'].update(selector='PIV1IN01'),
                lambda p: p.update(selectedForDeployment=True)):
            changed = copy.deepcopy(profile); mutate(changed)
            with self.assertRaisesRegex(ValueError, 'candidate_profile_identity'):
                self.a.validate_candidate_profile(changed)
        profile['roles'].clear()
        self.a.validate_candidate_profile(self.a.candidate_profile())
        self.assertEqual((self.a.COUNT, self.a.ELF_BYTES, self.a.ELF_SHA),
                         (949, 894552, 'e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99'))
        self.assertEqual(self.a.sha((HERE/'startup_manifest.oracle.json').read_bytes()), self.a.ORACLE_SHA)

    def test_phase_scope_fields_types_and_legacy_dispatch_are_disjoint(self):
        for index in (-1, 947, True, 1.0, '1'):
            with self.assertRaises(ValueError): self.a.candidate_phase(index)
        phase = self.a.candidate_phase(937)
        for name, value in (('id', 'startup_0937'), ('kind', 'BOOTSTRAP'),
                            ('initializerIndex', False), ('writeOffset', 0),
                            ('candidateScope', self.a.PROFILE_SCHEMA)):
            changed = {**phase, name: value}
            with self.assertRaises(ValueError): self.a.candidate_phase_index(changed)
        with self.assertRaises(ValueError): self.a.phase_index(phase)
        with self.assertRaises(ValueError): self.a.selected_image(self.a.candidate_profile())
        with self.assertRaises(ValueError): self.a.candidate_phase_index({**phase, 'extra': None})


if __name__=='__main__':unittest.main()


class CandidatePreviewManifestTests(unittest.TestCase):
    """Actual public image/current pinned helpers; no journals, RPC or hash mocks."""
    @classmethod
    def setUpClass(cls):
        cls.a = o.startup_manifest()
        cls.ref = cls.a.candidate_oracle()
        cls.image = Path(cls.ref['provenance']['elfPublicPath']).read_bytes()
        cls.manifest = cls.a.candidate_assemble(cls.image)
        cls.j,cls.rules,cls.c = cls.a._candidate_helpers()
        cls.binding = cls.j.candidate_clock_binding(cls.ref['preManifest'],cls.ref['clock'])

    def expected(self,index,stage='POST',**kw):
        return self.a.candidate_expected(self.manifest,index,stage,
            funding_mode='CONDITIONAL_H2_CREDIT',**kw)

    def test_complete947_actual_image_inverse_write_partition_and_default_isolation(self):
        m = self.a.candidate_validate(self.manifest); phases = m['phases']
        self.assertEqual(len(phases),947)
        self.assertEqual([p['index'] for p in phases],list(range(947)))
        self.assertEqual([p['initializerIndex'] for p in phases if p['reviewed'] is None],list(range(9)))
        recovered = bytearray()
        for p in phases:
            review = p['reviewed']
            if review is None: continue
            message = base64.b64decode(review['messageBase64'])
            self.assertEqual(self.a.sha(message),review['messageSha256'])
            self.assertLessEqual(1+64*len(review['requiredSigners'])+len(message),1232)
            if p['kind'] == 'UPLOAD_WRITE':
                size = p['writeBytes']; chunk = message[-size:]; header = message[-size-16:-size]
                tag,offset,encoded_size = struct.unpack('<IIQ',header)
                self.assertEqual((tag,offset,encoded_size),(1,len(recovered),size))
                recovered.extend(chunk)
        self.assertEqual(bytes(recovered),self.image)
        self.assertEqual(self.a.sha(recovered),self.a.candidate_profile()['artifact']['sha256'])
        self.assertTrue(all(m[k] is False for k in self.a.CANDIDATE_PREVIEW_FLAGS))
        self.assertFalse(self.a.full(m))
        with self.assertRaisesRegex(ValueError,'startup_full_manifest_required'):
            self.a.validate(o.operator_api(),self.j,m)
        self.assertEqual(m['historicalStartupFeeProjectionLamports'],'4805000')
        self.assertNotIn('parentTail',m)

    def test_whole_manifest_regeneration_rejects_authority_fee_roles_identity_and_flags(self):
        changes=[('liveReady',True),('runtimePredicateVerification',True),('controllerReady',True),('schema',self.a.SCHEMA)]
        for name,value in changes:
            bad=copy.deepcopy(self.manifest);bad[name]=value
            with self.assertRaises(ValueError):self.a.candidate_validate(bad)
        for mutate in (
            lambda m:m['fundingPrecondition'].__setitem__('fundingObserved',True),
            lambda m:m['historicalUncertainty'].__setitem__('resolved',True),
            lambda m:m['phases'][0]['reviewed']['accountRoles'][0].__setitem__('writable',not m['phases'][0]['reviewed']['accountRoles'][0]['writable']),
            lambda m:m['phases'][0].__setitem__('index',False),
            lambda m:m['phases'][934].__setitem__('writeBytes',956),
            lambda m:m['phases'][0].__setitem__('historicalFeeProjectionLamports','0')):
            bad=copy.deepcopy(self.manifest);mutate(bad)
            with self.assertRaises(ValueError):self.a.candidate_validate(bad)
        with self.assertRaisesRegex(ValueError,'candidate_image_identity'):
            self.a.candidate_assemble(self.image[:-1]+bytes([self.image[-1]^1]))

    def test_observed_insufficiency_conditional_credit_and_source_are_not_laundered(self):
        observed=self.a.candidate_expected(self.manifest,0,'PRE')
        r=self.ref['roles'];f=self.manifest['fundingPrecondition']
        self.assertEqual(observed['accounts'][r['h2']]['lamports'],'151301960')
        with self.assertRaisesRegex(ValueError,'candidate_observed_funding_insufficient'):
            self.a.candidate_expected(self.manifest,0,'POST')
        conditional=self.expected(0,'PRE')
        self.assertEqual(f['requiredH2InitialLamports'],'4583191240')
        self.assertEqual(f['conditionalH2CreditLamports'],'4431889280')
        self.assertEqual(conditional['accounts'][r['h2']]['lamports'],'4583191240')
        source=conditional['accounts'][r['source']]
        self.assertTrue(source['unresolved']);self.assertTrue(source['outside947Startup'])
        self.assertEqual(source['observedBaseline'],self.ref['baseline'][r['source']])
        self.assertIsNone(source['currentSourceFeeLamports'])
        self.assertEqual(self.ref['baseline'][r['h2']]['lamports'],'151301960')
        with self.assertRaisesRegex(ValueError,'candidate_funding_mode'):
            self.a.candidate_expected(self.manifest,0,'PRE',funding_mode='caller_credit')

    def test_loader_atomic_credit_and_full_snapshot_policy(self):
        r=self.ref['roles'];pre=self.expected(935,'PRE');post=self.expected(935)
        data=struct.pack('<IB',1,1)+self.c.b58decode(r['g0'],32)+self.image
        self.assertEqual(pre['accounts'][r['loaderBuffer']]['dataSha256'],self.a.sha(data))
        self.assertFalse(post['accounts'][r['loaderBuffer']]['exists'])
        self.assertEqual(len(pre['requiredSnapshotKeys']),60)
        self.assertEqual(len(self.expected(934)['requiredSnapshotKeys']),10)
        before=int(pre['accounts'][r['h2']]['lamports']);after=int(post['accounts'][r['h2']]['lamports'])
        rents=self.ref['rents']
        self.assertEqual(after-before,int(rents['892389'])-int(rents['36'])-int(rents['892397']))
        pd=post['accounts'][r['programData']]
        self.assertTrue(pd['unresolved']);self.assertFalse(pd['runtimePredicateVerification'])
        self.assertEqual(pd['sourceRule'],self.rules.candidate_program_data_rule())
        self.assertNotIn('deploymentSlot',pd)

    def test_real_clock_binding_compact_storage_target_bytes_and_unresolved_runtime(self):
        r=self.ref['roles']
        with self.assertRaisesRegex(ValueError,'candidate_clock_binding_required'):self.expected(937)
        before=self.expected(939,'PRE',clock_binding=self.binding)
        after=self.expected(939,clock_binding=self.binding)
        self.assertEqual(before['accounts'][r['governanceBuffer1']]['dataBytes'],1557)
        self.assertEqual(after['accounts'][r['transaction1']]['dataBytes'],1546)
        self.assertFalse(after['accounts'][r['governanceBuffer1']]['exists'])
        self.assertEqual(int(after['accounts'][r['g0']]['lamports'])-int(before['accounts'][r['g0']]['lamports']),8559800-10000)
        final=self.expected(945,clock_binding=self.binding)
        heartbeat=self.expected(946,clock_binding=self.binding)
        proposal=final['accounts'][r['proposal1']]
        self.assertTrue(proposal['unresolved']);self.assertIsNone(proposal['timestamp']);self.assertEqual(proposal['status'],5)
        self.assertEqual(len(proposal['approved']),4)
        for address,size in zip(self.ref['targetAddresses'],self.ref['accountSizes']['pivTargets']):
            self.assertEqual(final['accounts'][address]['dataBytes'],size)
            self.assertEqual(final['accounts'][address]['lamports'],self.ref['rents'][str(size)])
        reward=heartbeat['accounts'][self.ref['targetAddresses'][3]]
        self.assertTrue(reward['unresolved']);self.assertIsNone(reward['period'])
        self.assertEqual(reward['phase'],'candidate_0946');self.assertEqual(reward['stage'],'POST')
        bad=copy.deepcopy(self.binding);bad['liveReady']=True
        with self.assertRaises(ValueError):self.expected(945,clock_binding=bad)


class CandidateRuntimeAdapterTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.a=o.startup_manifest();cls.ref=cls.a.candidate_oracle();cls.j,cls.r,cls.c=cls.a._candidate_helpers()
        cls.manifest=cls.a.candidate_assemble(Path(cls.ref['provenance']['elfPublicPath']).read_bytes())
        cls.slot=int(cls.ref['baselineContextSlot'])+1
        cls.accounts={cls.r.PROGRAM_DATA:{'exists':False,'owner':None,'executable':None,'lamports':None,'dataBase64':None},
            cls.r.CLOCK:{'exists':True,'owner':cls.r.SYSVAR,'executable':False,'lamports':'1',
                'dataBase64':base64.b64encode(struct.pack('<QqQQq',cls.slot,0,0,0,1793016001)).decode()}}

    def test_real_source_bound_handoff_is_partial_unbound_and_manifest_committed(self):
        result=self.a.candidate_runtime_handoff(self.manifest,935,'PRE',self.accounts,context_slot=str(self.slot))
        self.assertEqual(result['manifestSha256'],self.a.sha(self.a.canonical(self.manifest)))
        self.assertEqual(result['runtimeFacts']['initializerBindingState'],'UNBOUND_TO_INITIALIZER')
        self.assertFalse(result['completePrerequisitesAuthenticated']);self.assertFalse(result['fullSnapshotVerified'])
        self.assertFalse(self.a.full(self.manifest))

    def test_preview_mask_flag_and_phase_confusion_do_not_become_runtime_policy(self):
        bad=copy.deepcopy(self.manifest);bad['runtimePredicateVerification']=True
        with self.assertRaises(ValueError):self.a.candidate_runtime_handoff(bad,935,'PRE',self.accounts,context_slot=str(self.slot))
        with self.assertRaises(ValueError):self.a.candidate_runtime_handoff(self.manifest,True,'PRE',self.accounts,context_slot=str(self.slot))
        accounts=copy.deepcopy(self.accounts);accounts[self.r.VAULT]=copy.deepcopy(accounts[self.r.PROGRAM_DATA])
        with self.assertRaisesRegex(ValueError,'candidate_runtime_account_set'):
            self.a.candidate_runtime_handoff(self.manifest,935,'PRE',accounts,context_slot=str(self.slot))
