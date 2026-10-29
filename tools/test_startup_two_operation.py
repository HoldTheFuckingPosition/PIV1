"""Focused dedicated-client tests. RPC/native/signature admission are mocked in
lifecycle tests; separate real pinned SDK/Ed25519 tests use public fixture bytes.
No test opens a private profile, signs, sends, or attests public execution speed.
"""
import base64
import copy
import json
import os
from pathlib import Path
import struct
import tempfile
import time
import types
import unittest
from unittest import mock

from tools import startup_two_operation as m


class Harness(m.Execution):
    def __init__(self,root,fault=None):
        self.fixture_root=Path(root);self.fault=fault;self.events=[];self.height=100;self.slot=1000;self.completed=0;self.sign_count=0;self.submit_count=0;self.observations=0
        ref=m.parse(m.read(m.REPO/'tools/startup_manifest.candidate-oracle.json'))
        p={'schema':'piv1-dedicated-two-operation-v1','cluster':'testnet','genesisHash':m.GENESIS,'operations':['CREATE_BUFFER','WRITE_FIRST_CHUNK'],
            'authority':'D-074','feeCapLamports':20000,'perOperationFeeCapLamports':10000,'rentLamports':m.RENT,'historicalUnknownReserveLamports':5000,
            'notBeforeUnix':0,'expiresUnix':int(time.time())+300,'sourcePins':{'tools/startup_release_operator.py':'a'*64,'tools/startup_signer_backend.cjs':'b'*64},
            'initialG0Lamports':999925000,'initialH2Lamports':4583191240,'baseParentPath':'/public/mock-parent'}
        raw=m.encode(p);package=self.fixture_root/'package.json';package.write_bytes(raw)
        auth={'schema':'piv1-launcher-release-authorization-v2','scope':'EXACT_BOUNDED_TESTNET_RELEASE','manifestSha256':m.sha(raw),
            'expiresUnixTimestamp':str(int(time.time())+300),'operatorSourceSha256':'a'*64,'signerBackendSha256':'b'*64,'baseParentTail':None}
        ar=m.encode(auth);ap=self.fixture_root/'auth.json';ap.write_bytes(ar)
        super().__init__(package,m.sha(raw),ap,m.sha(ar),'/not-read/mock-profile',self.fixture_root/'evidence')
        self.ref=ref;self.roles=ref['roles'];self.image=bytes(892352)
        self.sources={'tools/startup_two_operation.cjs':m.read(m.REPO/'tools/startup_two_operation.cjs')}
        self.c=m.module('collect_startup_public_evidence',m.read(m.REPO/'tools/collect_startup_public_evidence.py'))
        self.transport_leaf=types.SimpleNamespace(public_rpc_transport=self.transport,invoke_signer=self.native_mock)
        self.history=self.fixture_root/'closed-history';self.history.write_bytes(b'closed immutable history\n')
        self.history_bytes=self.history.read_bytes();self.send_intent_seen=False
    def audit_static(self):
        self.events.append(('static',self.index,self.submit_count));self.authorized();self.store.audit()
        m.need(self.history.read_bytes()==self.history_bytes,'historical_pin')
        return self.image
    def account(self,owner,lamports,data=b'',executable=False):
        return {'owner':owner,'lamports':lamports,'executable':executable,'data':[m.b64(data),'base64']}
    def accounts(self,completed):
        r=self.roles;a={k:None for k in [m.SYSTEM]+[r[n] for n in m.NAMES]}
        a[r['g0']]=self.account(m.SYSTEM,self.p['initialG0Lamports']-sum(m.FEES[:completed]))
        a[r['h2']]=self.account(m.SYSTEM,self.p['initialH2Lamports']-(m.RENT if completed else 0))
        if completed:
            data=struct.pack('<IB',1,1)+self.c.b58decode(r['g0'],32)+self.image[:956 if completed==2 else 0]+bytes(892352-(956 if completed==2 else 0))
            a[r['loaderBuffer']]=self.account(m.LOADER,m.RENT,data)
        for key,data in [(m.SYSTEM,b'solana_system_program'),(r['loaderProgram'],b'solana_bpf_loader_upgradeable_program'),(r['computeBudget'],b'compute_budget_program')]:
            a[key]=self.account('NativeLoader1111111111111111111111111111111',1,data,True)
        a[r['rent']]=self.account(m.SYSVAR,1009200,base64.b64decode('2BMAAAAAAAAAAAAAAADwPzI='))
        a[r['clock']]=self.account(m.SYSVAR,1169280,struct.pack('<QqQQq',self.slot,1,1,1,100+self.slot))
        return a
    def transport(self,c,raw):
        calls=m.parse(raw)
        if self.fault=='current_intent' and any(r['method']=='getFeeForMessage' for r in calls) and hasattr(self,'intent_name'):
            (self.store.path/self.intent_name).write_bytes(b'corrupted current intent\n')
        self.events.append(('rpc',[r['method'] for r in calls]));self.slot+=1;self.height+=1;rows=[]
        for request in calls:
            method=request['method'];p=request['params'];result=None
            if method=='getGenesisHash':result='wrong' if self.fault=='cluster' else m.GENESIS
            elif method=='getLatestBlockhash':result={'context':{'slot':self.slot},'value':{'blockhash':self.roles['vault0'],'lastValidBlockHeight':300}}
            elif method=='getBlockHeight':result=270 if self.fault=='lifetime' and self.native_reserved==1 else self.height
            elif method=='getMinimumBalanceForRentExemption':result=m.RENT
            elif method=='getFeeForMessage':result={'context':{'slot':self.slot},'value':m.FEES[self.index]}
            elif method=='isBlockhashValid':result={'context':{'slot':self.slot},'value':True}
            elif method=='getMultipleAccounts':
                a=self.accounts(self.completed)
                if self.fault=='authority':a[self.roles['g0']]['owner']=m.LOADER
                if self.fault=='amount':a[self.roles['h2']]['lamports']+=1
                if self.fault=='payload' and self.completed==2:
                    d=bytearray(base64.b64decode(a[self.roles['loaderBuffer']]['data'][0]));d[37]^=1;a[self.roles['loaderBuffer']]['data'][0]=m.b64(d)
                result={'context':{'slot':self.slot},'value':[a[k] for k in p[0]]}
            elif method=='sendTransaction':
                self.assert_intent();self.submit_count+=1;m.need(self.submit_count<=2,'test_duplicate_send');self.sent_wire=p[0]
                self.inclusion_slot=self.slot;self.completed=self.index+1
                self.receipt_pre=self.accounts(self.index);self.receipt_post=self.accounts(self.index+1)
                if self.fault=='closed_history':self.history.write_bytes(b'changed closed file after current send\n')
                if self.fault=='send_error':raise self.c.TransportFailure('mock_ambiguous_send')
                result=self.c.b58encode(base64.b64decode(p[0])[1:65])
            elif method=='getSignatureStatuses':
                self.observations+=1
                result={'context':{'slot':self.slot},'value':[None if self.fault=='unknown' else {'slot':self.inclusion_slot,'err':({'InstructionError':[0,'Custom']} if self.fault=='failed_tx' else None),'confirmationStatus':'finalized'}]}
            elif method=='getTransaction':
                addresses=[x['address'] for x in self.bound['reviewed']['accountRoles']]
                balances=lambda a:[a[k]['lamports'] if a[k] else 0 for k in addresses]
                result={'slot':self.inclusion_slot,'version':'legacy','transaction':[self.sent_wire,'base64'],
                    'meta':{'err':None,'fee':m.FEES[self.index],'preBalances':balances(self.receipt_pre),'postBalances':balances(self.receipt_post)}}
                if self.fault=='failed_tx':
                    result['meta']['err']={'InstructionError':[0,'Custom']}
                    result['meta']['postBalances']=result['meta']['preBalances'].copy();result['meta']['postBalances'][0]-=m.FEES[self.index]
                if self.fault=='receipt':result['meta']['postBalances'][0]+=1
            else:raise AssertionError(method)
            rows.append({'jsonrpc':'2.0','id':request['id'],'result':result})
        return 200,[],m.encode(rows)
    def assert_intent(self):
        files=sorted(self.store.path.glob('*-SEND_INTENT.json'));m.need(len(files)==self.index+1,'test_intent_before_send')
        row=m.parse(files[-1].read_bytes());m.need(row['data']['wireBase64']==self.signed_wire,'test_intent_original')
        self.events.append(('send',self.index,self.last_slot))
    def native_mock(self,j,payload,_raw_capture):
        self.sign_count+=1;self.events.append(('native',self.index));packet=payload['packets'][0]
        wire=bytearray(base64.b64decode(packet['wireBase64']));wire[1:1+64*wire[0]]=bytes([self.index+1])*(64*wire[0]);self.signed_wire=m.b64(wire)
        response={'schema':'piv1-startup-signing-response-v1','requestId':payload['request']['requestId'],'packets':[{'id':packet['id'],'wireBase64':self.signed_wire}]}
        _raw_capture(m.encode(response),True);return response


class DedicatedTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix='piv1-two-operation-test-',dir='/home/jerem/piv1-pilot-checkpoints');self.addCleanup(self.tmp.cleanup)
    def harness(self,fault=None):return Harness(self.tmp.name,fault)
    def execute(self,h):
        real=m.Sdk.call
        def call(sdk,value):
            if value['action']!='VERIFY':return real(sdk,value)
            wire=base64.b64decode(value['wireBase64']);msg=wire[1+64*wire[0]:]
            return {'wireSha256':m.sha(wire),'messageSha256':m.sha(msg),'signatureHex':wire[1:65].hex()}
        with mock.patch.object(m.Sdk,'call',call):return h.run()
    def test_exact_two_operations_no_generic_hotpath(self):
        h=self.harness();result=self.execute(h)
        self.assertEqual(result['status'],'FINALIZED_EXACT_TWO_OPERATIONS');self.assertEqual(h.submit_count,2);self.assertEqual(h.sign_count,2)
        self.assertEqual(result['feeLamports'],15000);self.assertEqual(result['rentLamports'],m.RENT)
        print(json.dumps({'scope':'HOST_MOCK_ONLY_REAL_SDK_BIND_MOCKED_NATIVE_TLS_VERIFY_STATIC_AUDIT','timings':result['timings']},sort_keys=True))
        self.assertEqual([e for e in h.events if e[0]=='static'],[('static',0,0),('static',0,1),('static',1,1),('static',1,2)])
        source=m.read(Path(m.__file__)).decode()
        for forbidden in ('_candidate_governed_buffer_range(','.replay(','.candidate_validate(','.candidate_assemble(','.helpers('):self.assertNotIn(forbidden,source)
        for phase in (0,1):
            rows=[t['stage'] for t in result['timings'] if t['operation']==phase]
            self.assertLess(rows.index('static_audit_before_original'),rows.index('rpc_wait_original'))
            self.assertLess(rows.index('durable_send_intent'),rows.index('rpc_wait_before_send'))
            self.assertLess(rows.index('rpc_wait_send'),rows.index('rpc_send_request_persistence'))
    def test_wrong_cluster_authority_amount_lifetime_refuse_before_send(self):
        for fault in ('cluster','authority','amount','lifetime'):
            with self.subTest(fault=fault),tempfile.TemporaryDirectory(dir=self.tmp.name) as root:
                h=Harness(root,fault)
                with self.assertRaises(m.Stop):self.execute(h)
                self.assertEqual(h.submit_count,0);self.assertEqual(h.sign_count,0)
    def test_current_intent_corruption_during_freshness_is_not_static_exception(self):
        h=self.harness('current_intent')
        with self.assertRaisesRegex(m.Stop,'current_durable_record_changed'):self.execute(h)
        self.assertEqual(h.submit_count,0);self.assertEqual(h.sign_count,1)
    def test_failed_intent_is_never_submitted(self):
        h=self.harness();original=h.store.record
        def fail(kind,data):
            if kind=='SEND_INTENT':raise OSError('mock fsync uncertainty')
            return original(kind,data)
        h.store.record=fail
        with self.assertRaises(m.Stop):self.execute(h)
        self.assertEqual(h.sign_count,1);self.assertEqual(h.submit_count,0)
    def test_ambiguous_send_and_null_status_hold_no_next(self):
        for fault in ('send_error','unknown'):
            with self.subTest(fault=fault),tempfile.TemporaryDirectory(dir=self.tmp.name) as root:
                h=Harness(root,fault)
                with self.assertRaises(m.Stop):self.execute(h)
                self.assertEqual(h.submit_count,1);self.assertEqual(h.sign_count,1)
                self.assertFalse(list(h.store.path.glob('*-FINALIZED.json')))
    def test_exact_receipt_and_written_payload_are_required(self):
        for fault,sends in (('receipt',1),('payload',2)):
            with self.subTest(fault=fault),tempfile.TemporaryDirectory(dir=self.tmp.name) as root:
                h=Harness(root,fault)
                with self.assertRaises(m.Stop):self.execute(h)
                self.assertEqual(h.submit_count,sends)
                hold=m.parse((h.store.path/'hold.json').read_bytes());self.assertEqual(hold['confirmedFeeLamports'],sum(m.FEES[:sends]))
    def test_disclosed_static_timing_finds_corruption_after_send_blocks_result(self):
        h=self.harness('closed_history')
        with self.assertRaisesRegex(m.Stop,'historical_pin'):self.execute(h)
        self.assertEqual(h.submit_count,1);self.assertEqual(h.sign_count,1)
        hold=m.parse((h.store.path/'hold.json').read_bytes());self.assertEqual(hold['confirmedFeeLamports'],10000);self.assertEqual(hold['confirmedRentLamports'],m.RENT)
        self.assertFalse(list(h.store.path.glob('*-FINALIZED.json')))
    def test_finalized_failed_original_charges_fee_and_holds(self):
        h=self.harness('failed_tx')
        with self.assertRaisesRegex(m.Stop,'finalized_failed_transaction_hold'):self.execute(h)
        hold=m.parse((h.store.path/'hold.json').read_bytes())
        self.assertEqual(hold['confirmedFeeLamports'],10000);self.assertEqual(hold['confirmedRentLamports'],0)
        self.assertEqual(h.submit_count,1);self.assertEqual(h.sign_count,1)
    def test_freshness_stall_after_capture_is_refused(self):
        h=self.harness();original=h.store.write
        def stall(name,raw):
            original(name,raw)
            if 'before_sign.response' in name:h.last_rpc_response_at-=2
        h.store.write=stall
        with self.assertRaisesRegex(m.Stop,'fresh_sample_aged_hold'):self.execute(h)
        self.assertEqual(h.sign_count,0);self.assertEqual(h.submit_count,0)
    def test_send_request_capture_failure_stays_consumed_hold(self):
        h=self.harness();original=h.store.write
        def fail(name,raw):
            if '-send.request' in name:raise OSError('mock post-send persistence uncertainty')
            original(name,raw)
        h.store.write=fail
        with self.assertRaises(m.Stop):self.execute(h)
        self.assertEqual(h.submit_count,1);self.assertEqual(h.sign_count,1)
        self.assertEqual(len(list(h.store.path.glob('*-SEND_INTENT.json'))),1)
        with self.assertRaises(FileExistsError):m.Store(h.store.path)
    def test_real_sdk_exact_templates_and_public_ed25519(self):
        h=self.harness();source=m.read(m.REPO/'tools/startup_two_operation.cjs')
        for index in (0,1):
            worker=m.Sdk(source,index,h.template(index))
            try:
                bound=worker.call({'action':'BIND','blockhash':h.roles['vault0']})
                raw=h.template(index);offset=4+raw[3]*32
                expected=raw[:offset]+h.c.b58decode(h.roles['vault0'],32)+raw[offset+32:]
                self.assertEqual(base64.b64decode(bound['reviewed']['messageBase64']),expected)
                with self.assertRaises(m.Stop):worker.call({'action':'VERIFY','wireBase64':bound['wireBase64']})
            finally:worker.close()
        fixture=m.parse(m.read(m.REPO/'clients/cli/startup-public-check.fixture.json'))['transactions'][0]
        wire=base64.b64decode(fixture['signedWireBase64']);message=wire[1+64*wire[0]:];offset=4+32*message[3]
        worker=m.Sdk(source,0,message)
        try:
            worker.call({'action':'BIND','blockhash':h.c.b58encode(message[offset:offset+32])})
            identity=worker.call({'action':'VERIFY','wireBase64':fixture['signedWireBase64']});self.assertEqual(identity['wireSha256'],fixture['signedWireSha256'])
            damaged=bytearray(wire);damaged[1]^=1
            with self.assertRaises(m.Stop):worker.call({'action':'VERIFY','wireBase64':m.b64(damaged)})
        finally:worker.close();h.store.close()

if __name__=='__main__':unittest.main()
