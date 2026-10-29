"""Focused continuation controls; public SDK bind real, all native/TLS mocked.

No live network, private profile/key reads or signature creation occurs. Positive
VERIFY is a host fixture, and source/history audit is mocked except store checks.
"""
import base64
import copy
import os
from pathlib import Path
import struct
import tempfile
import time
import types
import unittest
from unittest import mock

from tools import startup_buffer_upload as m
from tools import startup_two_operation as b
from tools.test_startup_two_operation import Harness as TwoHarness


class Harness(m.UploadExecution):
    def __init__(self,root,fault=None,stop_after=1,start_phase=2):
        self.fixture_root=Path(root);self.fault=fault;self.events=[];self.height=100;self.slot=1000;self.completed=start_phase;self.sign_count=0;self.submit_count=0;self.observations=0;self.receipt_requests=0;self.early_receipt_requests=0;self.last_status_finalized=False;self.stop_after=stop_after
        ref=b.parse(b.read(b.REPO/'tools/startup_manifest.candidate-oracle.json'))
        self.p={'schema':'piv1-dedicated-buffer-upload-v1','cluster':'testnet','genesisHash':b.GENESIS,'authority':'D-074','securityPolicy':'D-075','firstPhase':2,'lastPhase':934,'startOffset':956,'endOffset':892352,'nativeCap':933,'sendCap':933,'feeCapLamports':m.CAP,'perOperationFeeCapLamports':5000,'rentLamports':0,'historicalUnknownReserveLamports':5000,'wallSeconds':28800,'notBeforeUnix':0,'expiresUnix':int(time.time())+28800,'sourcePins':{'tools/startup_release_operator.py':'a'*64,'tools/startup_signer_backend.cjs':'b'*64},'initialG0Lamports':999910000,'initialH2Lamports':49204880,'baseParentPath':'/public/mock-parent','baseParentTail':None}
        self.raw=b.encode(self.p);self.package_path=self.fixture_root/'package.json';self.package_path.write_bytes(self.raw);self.package_sha=b.sha(self.raw)
        self.auth={'schema':'piv1-launcher-release-authorization-v2','scope':'EXACT_BOUNDED_TESTNET_RELEASE','manifestSha256':self.package_sha,'expiresUnixTimestamp':str(int(time.time())+28800),'operatorSourceSha256':'a'*64,'signerBackendSha256':'b'*64,'baseParentTail':None};self.auth_raw=b.encode(self.auth);self.authorization_path=self.fixture_root/'auth.json';self.authorization_path.write_bytes(self.auth_raw);self.authorization_sha=b.sha(self.auth_raw);self.profile_path='/not-read/mock-profile'
        self.start=time.monotonic();self.deadline=self.start+28800;self.timings=[];self.index=start_phase;self.sent=start_phase-2;self.native=start_phase-2;self.native_reserved=start_phase-2;self.fees=(start_phase-2)*5000;self.observed_fees=self.fees;self.reserved=self.fees;self.observed_rent=0;self.last_slot=0;self.last_clock=None;self.rpc_count=0;self.current=None;self.guard_busy=False;self.guard_ready=True;self.worker=None
        self.store=m.UploadStore(self.fixture_root/'upload',self.package_sha,self.authorization_sha,self.fixture_root/'progress.anchor.jsonl');self.store.boundary=self.guard
        self.ref=ref;self.roles=ref['roles'];self.image=bytes((i%251 for i in range(m.END)));self.sources={'tools/startup_two_operation.cjs':b.read(b.REPO/'tools/startup_two_operation.cjs')};self.c=b.module('collect_startup_public_evidence',b.read(b.REPO/'tools/collect_startup_public_evidence.py'));self.transport_leaf=types.SimpleNamespace(public_rpc_transport=self.transport,invoke_signer=self.native_mock)
        self.history=self.fixture_root/'history';self.history.write_bytes(b'history\n');self.history_bytes=self.history.read_bytes();self.finish_real=self.store.finish
        def finish(result):
            self.finish_real(result)
            if self.submit_count>=self.stop_after and self.index<934:self.deadline=time.monotonic()+119
        self.store.finish=finish
    def source_audit(self):
        b.need(self.history.read_bytes()==self.history_bytes,'source_pin')
    def audit_static(self):
        self.source_audit();self.authorized();self.store.audit(full=True)
    account=TwoHarness.account
    def accounts(self,completed):
        r=self.roles;a={k:None for k in [b.SYSTEM]+[r[n] for n in b.NAMES]};a[r['g0']]=self.account(b.SYSTEM,self.p['initialG0Lamports']-(completed-2)*5000);a[r['h2']]=self.account(b.SYSTEM,self.p['initialH2Lamports']);uploaded=min((completed-1)*956,m.END);data=struct.pack('<IB',1,1)+self.c.b58decode(r['g0'],32)+self.image[:uploaded]+bytes(m.END-uploaded);a[r['loaderBuffer']]=self.account(b.LOADER,b.RENT,data)
        for key,data in [(b.SYSTEM,b'solana_system_program'),(r['loaderProgram'],b'solana_bpf_loader_upgradeable_program'),(r['computeBudget'],b'compute_budget_program')]:a[key]=self.account('NativeLoader1111111111111111111111111111111',1,data,True)
        a[r['rent']]=self.account(b.SYSVAR,1009200,base64.b64decode('2BMAAAAAAAAAAAAAAADwPzI='));a[r['clock']]=self.account(b.SYSVAR,1169280,struct.pack('<QqQQq',self.slot,1,1,1,100+self.slot));return a
    def transport(self,c,raw):
        calls=b.parse(raw);self.slot+=1;self.height+=1;rows=[]
        for q in calls:
            method=q['method'];p=q['params'];result=None
            if method=='getGenesisHash':result=b.GENESIS
            elif method=='getLatestBlockhash':result={'context':{'slot':self.slot},'value':{'blockhash':self.roles['vault0'],'lastValidBlockHeight':1000}}
            elif method=='getBlockHeight':result=970 if self.fault=='lifetime' else self.height
            elif method=='getMinimumBalanceForRentExemption':result=b.RENT
            elif method=='getFeeForMessage':result={'context':{'slot':self.slot},'value':5001 if self.fault=='fee' else 5000}
            elif method=='isBlockhashValid':result={'context':{'slot':self.slot},'value':True}
            elif method=='getMultipleAccounts':
                a=self.accounts(self.completed)
                if self.fault=='authority':a[self.roles['loaderBuffer']]['owner']=b.SYSTEM
                if self.fault=='payload' and self.submit_count:
                    x=bytearray(base64.b64decode(a[self.roles['loaderBuffer']]['data'][0]));x[37]^=1;a[self.roles['loaderBuffer']]['data'][0]=b.b64(x)
                result={'context':{'slot':self.slot},'value':[a[k] for k in p[0]]}
            elif method=='sendTransaction':
                self.store.current_record(self.intent_name);self.submit_count+=1;self.sent_wire=p[0];self.inclusion_slot=self.slot;self.receipt_pre=self.accounts(self.index);self.completed=self.index+1;self.receipt_post=self.accounts(self.completed)
                if self.fault=='current_sidecar':
                    name=next(n for n in self.store.expected if n.endswith('-PREPARED.json'));(self.store.path/name).write_bytes(b'bad')
                if self.fault=='source':self.history.write_bytes(b'bad')
                if self.fault=='ambiguous':raise self.c.TransportFailure('mock_ambiguous')
                result=self.c.b58encode(base64.b64decode(p[0])[1:65])
            elif method=='getSignatureStatuses':
                self.observations+=1;pending=self.fault=='unknown' or self.fault=='null_then_final' and self.observations==1
                status=None if pending else {'slot':self.inclusion_slot,'err':({'InstructionError':[0,'MockFailure']} if self.fault=='failed_receipt' else None),'confirmationStatus':'finalized'}
                self.last_status_finalized=type(status) is dict and status['confirmationStatus']=='finalized';result={'context':{'slot':self.slot},'value':[status]}
            elif method=='getTransaction':
                self.receipt_requests+=1
                if not self.last_status_finalized:
                    # Retained public failure shape: an unavailable transaction
                    # may have no result field. It must never be requested early.
                    self.early_receipt_requests+=1;rows.append({'jsonrpc':'2.0','id':q['id']});continue
                if self.fault=='malformed_receipt':rows.append({'jsonrpc':'2.0','id':q['id']});continue
                if self.fault=='missing_receipt':result=None
                else:
                    addresses=[x['address'] for x in self.bound['reviewed']['accountRoles']];balances=lambda a:[a[k]['lamports'] if a[k] else 0 for k in addresses];result={'slot':self.inclusion_slot,'version':'legacy','transaction':[self.sent_wire,'base64'],'meta':{'err':({'InstructionError':[0,'MockFailure']} if self.fault=='failed_receipt' else None),'fee':5000,'preBalances':balances(self.receipt_pre),'postBalances':balances(self.receipt_post)}}
            else:raise AssertionError(method)
            rows.append({'jsonrpc':'2.0','id':q['id'],'result':result})
        return 200,[],b.encode(rows)
    def native_mock(self,j,payload,_raw_capture):
        self.sign_count+=1;packet=payload['packets'][0];wire=bytearray(base64.b64decode(packet['wireBase64']));wire[1:1+64*wire[0]]=bytes([7])*(64*wire[0]);self.signed_wire=b.b64(wire);response={'schema':'piv1-startup-signing-response-v1','requestId':payload['request']['requestId'],'packets':[{'id':packet['id'],'wireBase64':self.signed_wire}]};_raw_capture(b.encode(response),True);return response


class UploadTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix='piv1-buffer-upload-test-',dir='/home/jerem/piv1-pilot-checkpoints');self.addCleanup(self.tmp.cleanup)
    def execute(self,h):
        real=b.Sdk.call
        def call(sdk,value):
            if value['action']!='VERIFY':return real(sdk,value)
            wire=base64.b64decode(value['wireBase64']);return {'wireSha256':b.sha(wire),'messageSha256':b.sha(wire[1+64*wire[0]:]),'signatureHex':wire[1:65].hex()}
        with mock.patch.object(b.Sdk,'call',call):return h.run()
    def test_exact_offsets_last_404_and_final_full_readback(self):
        self.assertEqual([m.extent(2),m.extent(933),m.extent(934)],[(956,956),(890992,956),(891948,404)])
        self.assertEqual(sum(m.extent(p)[1] for p in range(2,935)),m.END-m.START)
        h=Harness(self.tmp.name,start_phase=934);template=h.template(934);self.assertEqual(template[-404:],h.image[-404:]);h.snapshot({'context':{'slot':1000},'value':[h.accounts(935)[k] for k in sorted({b.SYSTEM}|{h.roles[n] for n in b.NAMES})]},935,0);h.store.close()
    def test_clean_two_phase_pause_and_store_reopen(self):
        h=Harness(self.tmp.name,stop_after=2);result=self.execute(h);self.assertEqual(result['status'],'PAUSED_CLEAN');self.assertEqual(result['nextOffset'],2868);self.assertEqual(h.submit_count,2)
        s=m.UploadStore(h.store.path,h.package_sha,h.authorization_sha,h.store.anchor_path)
        try:s.audit(full=True);self.assertEqual(list(s.closed),[2,3]);self.assertEqual(s.closed[3]['endOffset'],2868)
        finally:s.close()
        reopened=m.UploadStore(h.store.path,h.package_sha,h.authorization_sha,h.store.anchor_path);h.store=reopened;h.store.boundary=h.guard;h.deadline=time.monotonic()+1000;h.fees=h.observed_fees=h.sent=h.native=h.native_reserved=h.reserved=0;h.last_slot=0;h.last_clock=None;h.rpc_count=0
        real=b.Sdk.call
        def verify(sdk,value):
            if value['action']!='VERIFY':return real(sdk,value)
            wire=base64.b64decode(value['wireBase64']);return {'wireSha256':b.sha(wire),'messageSha256':b.sha(wire[1+64*wire[0]:]),'signatureHex':wire[1:65].hex()}
        try:
            with mock.patch.object(b.Sdk,'call',verify):h.semantic_replay()
            self.assertEqual((h.index,h.fees,h.sent,h.native),(4,10000,2,2))
        finally:reopened.close()
    def test_fee_authority_lifetime_refuse_before_native(self):
        for fault in ('fee','authority','lifetime'):
            with self.subTest(fault=fault),tempfile.TemporaryDirectory(dir=self.tmp.name) as root:
                h=Harness(root,fault)
                with self.assertRaises(b.Stop):self.execute(h)
                self.assertEqual((h.submit_count,h.sign_count),(0,0))
    def test_durable_intent_failure_cannot_send(self):
        h=Harness(self.tmp.name);record=h.store.record
        def fail(kind,data):
            if kind=='SEND_INTENT':raise OSError('fsync mock')
            return record(kind,data)
        h.store.record=fail
        with self.assertRaises(b.Stop):self.execute(h)
        self.assertEqual(h.sign_count,1);self.assertEqual(h.submit_count,0)
    def test_current_sidecar_and_source_fault_stop_after_callback(self):
        for fault in ('current_sidecar','source'):
            with self.subTest(fault=fault),tempfile.TemporaryDirectory(dir=self.tmp.name) as root:
                h=Harness(root,fault)
                with self.assertRaises(b.Stop):self.execute(h)
                self.assertEqual(h.submit_count,1);self.assertEqual(h.sign_count,1);self.assertFalse(h.store.closed);self.assertTrue(any('-send.response.json' in n for n in h.store.expected))
    def test_null_then_finalized_retains_absence_without_replacement(self):
        h=Harness(self.tmp.name,'null_then_final');result=self.execute(h);self.assertEqual(result['status'],'PAUSED_CLEAN');self.assertEqual(h.observations,2);self.assertEqual(h.submit_count,1);self.assertEqual(h.sign_count,1)
        raw=[b.parse(b.read(h.store.path/n)) for n in h.store.expected if '-status.response.json' in n];self.assertEqual(raw[0][1]['result']['value'],[None]);self.assertEqual(len(raw[0]),2);self.assertEqual((h.receipt_requests,h.early_receipt_requests),(1,0));requests=[b.parse(b.read(h.store.path/n)) for n in h.store.expected if '-status.request.json' in n];self.assertTrue(all([q['method'] for q in batch]==['getGenesisHash','getSignatureStatuses'] for batch in requests))
    def test_finalized_missing_or_malformed_receipt_holds_without_retry(self):
        for fault,reason in (('missing_receipt','transaction_identity_hold'),('malformed_receipt','rpc_reply')):
            with self.subTest(fault=fault),tempfile.TemporaryDirectory(dir=self.tmp.name) as root:
                h=Harness(root,fault)
                with self.assertRaisesRegex(b.Stop,reason):self.execute(h)
                self.assertEqual((h.observations,h.sign_count,h.submit_count,h.receipt_requests,h.early_receipt_requests),(1,1,1,1,0));self.assertEqual(h.observed_fees,0);self.assertEqual(h.reserved,5000);self.assertFalse(h.store.closed)
    def test_receipt_readback_share_original_confirmation_deadline(self):
        h=Harness(self.tmp.name);transport=h.transport;real_clock=time.monotonic;offset=[0]
        def delayed(c,raw):
            out=transport(c,raw)
            if any(q['method']=='getTransaction' for q in b.parse(raw)):offset[0]=31
            return out
        h.transport_leaf.public_rpc_transport=delayed
        with mock.patch.object(m.time,'monotonic',lambda:real_clock()+offset[0]),self.assertRaisesRegex(b.Stop,'confirmation_bound_or_cluster'):self.execute(h)
        self.assertEqual((h.observations,h.sign_count,h.submit_count,h.receipt_requests),(1,1,1,1));self.assertEqual(h.observed_fees,0);self.assertEqual(h.reserved,5000);self.assertFalse(h.store.closed);self.assertTrue(any('-receipt.response.json' in n for n in h.store.expected))
    def test_finalized_failed_receipt_retains_exact_fee_and_holds(self):
        h=Harness(self.tmp.name,'failed_receipt')
        with self.assertRaisesRegex(b.Stop,'finalized_failed_transaction_hold'):self.execute(h)
        self.assertEqual((h.observations,h.sign_count,h.submit_count,h.receipt_requests),(1,1,1,1));self.assertEqual(h.observed_fees,5000);self.assertEqual(h.fees,0);self.assertFalse(h.store.closed)
    def test_unknown_and_ambiguous_never_resume_or_replace(self):
        for fault in ('unknown','ambiguous'):
            with self.subTest(fault=fault),tempfile.TemporaryDirectory(dir=self.tmp.name) as root:
                h=Harness(root,fault)
                with self.assertRaises(b.Stop):self.execute(h)
                self.assertEqual(h.submit_count,1);self.assertEqual(h.sign_count,1);self.assertEqual(h.receipt_requests,0)
                with self.assertRaises(b.Stop):m.UploadStore(h.store.path,h.package_sha,h.authorization_sha,h.store.anchor_path)
    def test_wrong_payload_retains_observed_fee_without_cursor(self):
        h=Harness(self.tmp.name,'payload')
        with self.assertRaisesRegex(b.Stop,'buffer_exact_readback'):self.execute(h)
        self.assertEqual(h.observed_fees,5000);self.assertEqual(h.fees,0);self.assertFalse(h.store.closed)
    def test_partial_journal_and_closed_file_corruption_fail_reopen(self):
        h=Harness(self.tmp.name);self.execute(h);jp=h.store.path/'journal.jsonl';jp.write_bytes(jp.read_bytes()+b'{')
        with self.assertRaisesRegex(b.Stop,'partial_journal'):m.UploadStore(h.store.path,h.package_sha,h.authorization_sha,h.store.anchor_path)
    def test_clean_boundary_failure_is_durable_halt(self):
        h=Harness(self.tmp.name);h.store.pause=lambda:(_ for _ in ()).throw(b.Stop('mock_clean_terminal_fault'))
        with self.assertRaisesRegex(b.Stop,'mock_clean_terminal_fault'):self.execute(h)
        self.assertIsNone(h.store.phase);self.assertEqual(h.store.rows[-1]['kind'],'HALT')
        with self.assertRaises(b.Stop):m.UploadStore(h.store.path,h.package_sha,h.authorization_sha,h.store.anchor_path)
    def test_native_callback_fault_retains_literal_output(self):
        h=Harness(self.tmp.name);native=h.native_mock
        def fault(j,payload,_raw_capture):
            h.history.write_bytes(b'bad source after native entry')
            return native(j,payload,_raw_capture)
        h.transport_leaf.invoke_signer=fault
        with self.assertRaises(b.Stop):self.execute(h)
        self.assertEqual(h.submit_count,0);self.assertEqual(h.sign_count,1);self.assertTrue(any(n.endswith('/native-2.json') for n in h.store.expected))
    def test_aged_final_sample_and_readonly_send_entry_fail_closed(self):
        h=Harness(self.tmp.name);capture=h.store.capture
        def aged(action):
            out=capture(action)
            if any('-before_send.response.json' in n for n in h.store.expected):h.last_rpc_response_at-=2
            return out
        h.store.capture=aged
        with self.assertRaisesRegex(b.Stop,'fresh_sample_aged_hold'):self.execute(h)
        self.assertEqual((h.submit_count,h.sign_count),(0,1))
        with self.assertRaisesRegex(b.Stop,'readonly_rpc_only'):h.rpc('other',[('sendTransaction',[])])
    def test_semantic_reopen_rejects_rehashed_wrong_original_balances(self):
        h=Harness(self.tmp.name);self.execute(h);folder=h.store.path/'phase-0002';name=next(n for n in h.store.phase_names[2] if '-receipt.response.json' in n);raw=b.parse(b.read(folder/name));raw[1]['result']['meta']['postBalances'][0]+=1;changed=b.encode(raw);(folder/name).write_bytes(changed);rows=copy.deepcopy(h.store.rows)
        for row in rows:
            if row['kind']=='FILE' and row['data']['name']==name:row['data']['sha256']=b.sha(changed);row['data']['bytes']=len(changed)
        tail=None;journal=[];anchors=[]
        for row in rows:
            row['previous']=tail;encoded=b.encode(row);journal.append(encoded);anchors.append(b.encode({'sequence':row['sequence'],'digest':b.sha(encoded),'previous':tail}));tail=b.sha(encoded)
        (h.store.path/'journal.jsonl').write_bytes(b''.join(journal));h.store.anchor_path.write_bytes(b''.join(anchors));h.store=m.UploadStore(h.store.path,h.package_sha,h.authorization_sha,h.store.anchor_path);h.store.boundary=h.guard;h.deadline=time.monotonic()+1000;h.fees=h.observed_fees=h.sent=h.native=h.native_reserved=h.reserved=h.rpc_count=0;h.last_slot=0;h.last_clock=None
        real=b.Sdk.call
        def verify(sdk,value):
            if value['action']!='VERIFY':return real(sdk,value)
            wire=base64.b64decode(value['wireBase64']);return {'wireSha256':b.sha(wire),'messageSha256':b.sha(wire[1+64*wire[0]:]),'signatureHex':wire[1:65].hex()}
        try:
            with mock.patch.object(b.Sdk,'call',verify),self.assertRaisesRegex(b.Stop,'replay_exact_receipt'):h.semantic_replay()
        finally:h.store.close()
    def test_closed_raw_journal_and_anchor_remain_fresh_before_next_native(self):
        for target in ('journal','anchor'):
            with self.subTest(target=target),tempfile.TemporaryDirectory(dir=self.tmp.name) as root:
                h=Harness(root,stop_after=2);real=h.transport;changed=[]
                def transport(c,raw):
                    result=real(c,raw)
                    if h.index==3 and any(q['method']=='getFeeForMessage' for q in b.parse(raw)):
                        self.assertIsNotNone(h.store._closed_prefix);path=h.store.path/'journal.jsonl' if target=='journal' else h.store.anchor_path;data=path.read_bytes();path.write_bytes(b'!'+data[1:]);changed.append(target)
                    return result
                h.transport_leaf.public_rpc_transport=transport
                with self.assertRaisesRegex(b.Stop,'raw_journal_changed'):self.execute(h)
                self.assertEqual((h.sign_count,h.submit_count),(1,1));self.assertEqual(list(h.store.closed),[2]);self.assertTrue(changed)
    def test_d075_closed_corruption_deferred_only_until_terminal(self):
        h=Harness(self.tmp.name,stop_after=2);real=h.transport;changed=[]
        def transport(c,raw):
            out=real(c,raw)
            if h.index==3 and any(r['method']=='sendTransaction' for r in b.parse(raw)):
                name=next(n for n in h.store.expected if n.startswith('phase-0002/') and n.endswith('-PREPARED.json'));(h.store.path/name).write_bytes(b'old altered');changed.append(name)
            return out
        h.transport_leaf.public_rpc_transport=transport
        with self.assertRaisesRegex(b.Stop,'evidence_changed'):self.execute(h)
        self.assertEqual(h.submit_count,2);self.assertEqual(h.sign_count,2);self.assertEqual(list(h.store.closed),[2]);self.assertTrue(changed)



class RetirementHarness(m.RetiringUploadExecution):
    """D077 host fixture: native/TLS/positive lifecycle crypto/history mocked."""
    def __init__(self,root,mode='retire_once',stop_after=1):
        self.retirements=0;Harness.__init__(self,root,stop_after=stop_after);self.store.close();self.mode=mode;self.chain_spent=0;self.building_send=False;self.completed=self.index=72;self.normal_exit_pins={};self.refusal=None
        self.p.update(schema='piv1-dedicated-buffer-upload-retirement-v1',securityPolicies=['D-075','D-077'],firstPhase=72,startOffset=67876,nativeCap=933,sendCap=863,retirementCap=70,feeCapLamports=4315000,reservationCapLamports=4665000,oldPhase72ReserveLamports=5000,initialG0Lamports=999560000,externalEvidenceReserveBytes=268435456,priorHistorySha256='d'*64)
        self.p['sourcePins']['tools/startup_buffer_upload.py']='c'*64
        exits=self.fixture_root/'normal-exits';exits.mkdir(mode=0o700);s=exits.stat();self.normal_exit_directory_identity={'dev':s.st_dev,'ino':s.st_ino,'uid':s.st_uid,'mode':0o700};attestor=self.fixture_root/'attestor.py';attestor.write_bytes(b'# inert public host attestor fixture\n');self.p.update(normalExitDirectory=str(exits),normalExitDirectoryIdentity=self.normal_exit_directory_identity,workerResultDirectory=str(self.fixture_root),exitAttestorPath=str(attestor),exitAttestorSha256=b.sha(attestor.read_bytes()))
        self.raw=b.encode(self.p);self.package_path.write_bytes(self.raw);self.package_sha=b.sha(self.raw);self.auth['manifestSha256']=self.package_sha;self.auth_raw=b.encode(self.auth);self.authorization_path.write_bytes(self.auth_raw);self.authorization_sha=b.sha(self.auth_raw)
        self.store=m.RetiringStore(self.fixture_root/'retiring-evidence',self.package_sha,self.authorization_sha,self.fixture_root/'retiring.anchor.jsonl');self.store.boundary=self.guard
        old={'identity':{'wireSha256':'e'*64,'signatureHex':bytes([8]*64).hex()},'lifetime':{'recentBlockhash':self.roles['vault0'],'lastValidBlockHeight':'1000','currentBlockHeight':'100','minimumRemainingBlockHeights':'40'}};self.prior=types.SimpleNamespace(old_intent=old,audit=lambda full=False:None);self.store_disposition=self.store.disposition
        def disposition(kind,data):
            self.store_disposition(kind,data)
            if kind=='FINALIZED' and self.submit_count>=self.stop_after:self.deadline=time.monotonic()+119
        self.store.disposition=disposition
    source_audit=Harness.source_audit
    audit_static=Harness.audit_static
    account=Harness.account
    native_mock=Harness.native_mock
    def accounts(self,completed):
        saved=self.p['initialG0Lamports'];spent=self.chain_spent+(5000 if self.building_send and completed==self.index+1 else 0);self.p['initialG0Lamports']=saved+(completed-2)*5000-spent
        try:return Harness.accounts(self,completed)
        finally:self.p['initialG0Lamports']=saved
    def transport(self,c,raw):
        calls=b.parse(raw);expiry=any(q['method']=='getSignatureStatuses' for q in calls) and any(q['method']=='isBlockhashValid' for q in calls)
        if expiry:
            self.slot+=1;self.height+=1;rows=[]
            for q in calls:
                method=q['method']
                if method=='getGenesisHash':result=b.GENESIS
                elif method=='getBlockHeight':result=990 if self.mode=='still_valid' and self.store.phase else 1001+self.height
                elif method=='isBlockhashValid':result={'context':{'slot':self.slot},'value':bool(self.mode=='still_valid' and self.store.phase)}
                elif method=='getMultipleAccounts':
                    a=self.accounts(self.completed)
                    if self.mode=='changed_account' and self.store.phase:a[self.roles['g0']]['lamports']-=1
                    result={'context':{'slot':self.slot},'value':[a[k] for k in q['params'][0]]}
                elif method=='getSignatureStatuses':result={'context':{'slot':self.slot},'value':[{'slot':self.slot,'err':None,'confirmationStatus':'finalized'} if self.mode=='chain_status' and self.store.phase else None]}
                else:raise AssertionError(method)
                rows.append({'jsonrpc':'2.0','id':q['id'],'result':result})
            if self.mode=='malformed_expiry' and self.store.phase:rows[1].pop('result')
            return 200,[],b.encode(rows)
        sending=any(q['method']=='sendTransaction' for q in calls);before=self.submit_count;self.building_send=sending
        try:status,headers,body=Harness.transport(self,c,raw)
        finally:
            self.building_send=False
            if self.submit_count>before:self.chain_spent+=5000
        rows=b.parse(body)
        if self.mode!='healthy' and self.store.phase==1 and any(q['method']=='getFeeForMessage' for q in calls) and hasattr(self,'intent_name'):
            for q,row in zip(calls,rows):
                if q['method']=='getBlockHeight':row['result']=970
        return status,headers,b.encode(rows)
    def _verify_many_public(self,packets):
        out=[]
        for packet in packets:
            wire=base64.b64decode(packet['signedWireBase64']);out.append({'wireSha256':b.sha(wire),'messageSha256':b.sha(wire[65:]),'signatureHex':wire[1:65].hex()})
        return out
    def attest(self,outcome,unsafe=None):
        pid=10000+outcome['attemptId'];result=self.fixture_root/f'worker-result-{pid}.public.json';result.write_bytes(b.encode(outcome));stdout=self.fixture_root/f'worker-{pid}.stdout';stderr=self.fixture_root/f'worker-{pid}.stderr';stdout.write_bytes(result.read_bytes());stderr.write_bytes(b'')
        receipt={'schema':'piv1-d077-normal-unsent-exit-v1','packageSha256':self.package_sha,'authorizationSha256':self.authorization_sha,'attestorSha256':self.p['exitAttestorSha256'],'attemptId':outcome['attemptId'],'logicalPhase':outcome['logicalPhase'],'candidateRecordSha256':outcome['candidateRecordSha256'],'journalSha256':outcome['journalSha256'],'anchorSha256':outcome['anchorSha256'],'journalTail':outcome['journalTail'],'workerResultSha256':b.sha(result.read_bytes()),'workerResultPath':str(result),'workerPid':pid,'exitCode':0,'normalWait':True,'childrenReaped':True,'timeout':False,'platformStop':False,'resourceStop':None,'nativeCalls':outcome['nativeCalls'],'nativeReservations':outcome['nativeReservations'],'sendCalls':outcome['sendCalls'],'reservedFeeLamports':outcome['reservedFeeLamports'],'startedUnix':self.store.rows[0]['data']['startedUnix'],'completedUnix':int(time.time()),'stdoutSha256':b.sha(stdout.read_bytes()),'stderrSha256':b.sha(stderr.read_bytes())}
        if unsafe:receipt.update(unsafe)
        path=Path(self.p['normalExitDirectory'])/f'attempt-{outcome["attemptId"]:04}.public.json';path.write_bytes(b.encode(receipt));return path
    def reopen(self):
        self.store=m.RetiringStore(self.store.path,self.package_sha,self.authorization_sha,self.store.anchor_path);self.store.boundary=self.guard;self.store_disposition=self.store.disposition
        def disposition(kind,data):
            self.store_disposition(kind,data)
            if kind=='FINALIZED' and self.submit_count>=self.stop_after:self.deadline=time.monotonic()+119
        self.store.disposition=disposition;self.start=time.monotonic();self.deadline=self.start+max(0,self.store.rows[0]['data']['startedUnix']+28800-int(time.time()));self.worker=None;self.sent=self.native=self.native_reserved=self.fees=self.observed_fees=self.reserved=self.retirements=self.rpc_count=0;self.last_slot=0;self.last_clock=None;self.normal_exit_pins={};self.index=72;self.refusal=None;self.guard_ready=False
        self._load_existing_exits();self.guard_ready=True;self.semantic_replay()


class RetirementTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix='piv1-retirement-test-',dir='/home/jerem/piv1-pilot-checkpoints');self.addCleanup(self.tmp.cleanup)
    execute=UploadTests.execute
    def first_stop(self,mode='retire_once'):
        h=RetirementHarness(self.tmp.name,mode);result=self.execute(h);self.assertEqual(result['status'],'UNSENT_LIFETIME_STOP_PENDING_EXIT');self.assertEqual((h.sign_count,h.submit_count,h.reserved),(1,0,5000));return h,result
    def test_normal_exit_expiry_retirement_then_fresh_same_offset_success(self):
        h,result=self.first_stop();h.attest(result);h.reopen();result=self.execute(h);self.assertEqual(result['status'],'PAUSED_CLEAN');self.assertEqual((h.sign_count,h.submit_count,h.retirements,h.reserved,h.fees,h.index),(2,1,1,10000,5000,73));self.assertEqual(h.store.retired[1]['nextOffset'],67876);self.assertEqual(h.store.finalized[2]['offset'],67876);self.assertEqual(h.store.bridge['oldPhase72ReserveLamports'],5000);self.assertEqual(h.store.bridge['oldAuReserveLamports'],5000)
        h.reopen();self.assertEqual((h.index,h.native,h.sent,h.retirements,h.reserved,h.fees),(73,2,1,1,10000,5000));h.store.close()
    def test_missing_or_unsafe_trusted_process_exit_never_retires(self):
        for unsafe in (None,{'exitCode':1},{'childrenReaped':False},{'timeout':True},{'resourceStop':'mock-resource'},{'platformStop':True}):
            with self.subTest(unsafe=unsafe),tempfile.TemporaryDirectory(dir=self.tmp.name) as root:
                h=RetirementHarness(root);result=self.execute(h)
                if unsafe is not None:h.attest(result,unsafe)
                with self.assertRaises((b.Stop,FileNotFoundError)):h.reopen()
                self.assertEqual(h.submit_count,0)
                if h.store.fd is not None:
                    try:h.store.close()
                    except OSError:pass
    def test_still_valid_malformed_changed_state_or_chain_status_hold(self):
        for mode in ('still_valid','malformed_expiry','changed_account','chain_status'):
            with self.subTest(mode=mode),tempfile.TemporaryDirectory(dir=self.tmp.name) as root:
                h=RetirementHarness(root,mode);result=self.execute(h);h.attest(result);h.reopen()
                with self.assertRaises(b.Stop):self.execute(h)
                self.assertEqual((h.sign_count,h.submit_count,h.retirements),(1,0,0))
    def test_retired_sidecar_stays_fresh_after_logical_success(self):
        h,result=self.first_stop();h.attest(result);h.reopen();self.execute(h);h.reopen();self.assertEqual(h.store.cursor,73);name=next(n for n in h.store.expected if n.startswith('phase-0001/') and n.endswith('-SEND_INTENT.json'));transport=h.transport_leaf.public_rpc_transport
        def corrupt_during_later_callback(c,raw):
            reply=transport(c,raw)
            if any(q['method']=='getLatestBlockhash' for q in b.parse(raw)):(h.store.path/name).write_bytes(b'corrupt retired original\n')
            return reply
        h.transport_leaf.public_rpc_transport=corrupt_during_later_callback
        with self.assertRaisesRegex(b.Stop,'evidence_changed'):self.execute(h)
        self.assertEqual((h.sign_count,h.submit_count),(2,1));self.assertTrue(any(n.startswith('phase-0003/') and n.endswith('.response.json') for n in h.store.expected))
    def test_exit_result_forgery_and_partial_retirement_fail_closed(self):
        h,result=self.first_stop();path=h.attest(result);raw=b.parse(path.read_bytes());raw['sendCalls']=1;path.write_bytes(b.encode(raw))
        with self.assertRaisesRegex(b.Stop,'normal_exit_counts'):h.reopen()
        h.store.close()
    def test_rehashed_stdout_or_stderr_cannot_forge_normal_exit_semantics(self):
        for suffix,key in (('stdout','stdoutSha256'),('stderr','stderrSha256')):
            with self.subTest(stream=suffix),tempfile.TemporaryDirectory(dir=self.tmp.name) as root:
                h=RetirementHarness(root);result=self.execute(h);path=h.attest(result);receipt=b.parse(path.read_bytes());stream=h.fixture_root/f'worker-{receipt["workerPid"]}.{suffix}';stream.write_bytes(b'hash-consistent forged exit output\n');receipt[key]=b.sha(stream.read_bytes());path.write_bytes(b.encode(receipt))
                with self.assertRaisesRegex(b.Stop,'normal_exit_stream_semantics'):h.reopen()
                h.store.close()
    def test_global_caps_and_final404_extent_remain_exact(self):
        h=RetirementHarness(self.tmp.name,'healthy');self.assertEqual((m.extent(72),m.extent(934)),((67876,956),(891948,404)));h.native_reserved=934
        with self.assertRaisesRegex(b.Stop,'global_retirement_caps'):h.authorized()
        # Last-phase fixture, not execution of the preceding 862 mock Writes.
        h.index=934;h.fees=862*5000;h.chain_spent=863*5000;self.assertEqual(h.template(934)[-404:],h.image[-404:]);h.snapshot({'context':{'slot':h.slot},'value':[h.accounts(935)[k] for k in sorted({b.SYSTEM}|{h.roles[n] for n in b.NAMES})]},935,0)
        h.store.close()
    def test_unsafe_refusal_causes_never_become_retirable(self):
        for fault in ('lifetime','fee','authority','ambiguous','source','native_incomplete'):
            with self.subTest(fault=fault),tempfile.TemporaryDirectory(dir=self.tmp.name) as root:
                h=RetirementHarness(root,'healthy');h.fault=fault
                if fault=='native_incomplete':
                    h.mode='retire_once'
                    def incomplete(j,payload,_raw_capture):return h.native_mock(j,payload,lambda raw,complete:_raw_capture(raw,False))
                    h.transport_leaf.invoke_signer=incomplete
                with self.assertRaises(b.Stop):self.execute(h)
                self.assertFalse(any(r['kind']=='UNSENT_LIFETIME_REFUSAL' for r in h.store.rows));self.assertEqual(h.retirements,0)
                with self.assertRaises(b.Stop):m.RetiringStore(h.store.path,h.package_sha,h.authorization_sha,h.store.anchor_path)
    def test_partial_record_and_semantically_forged_retirement_fail_closed(self):
        h,result=self.first_stop();h.attest(result);h.reopen();self.execute(h);journal=h.store.path/'journal.jsonl';anchor=h.store.anchor_path
        row=next(r for r in h.store.rows if r['kind']=='RETIRED_UNSENT');path=h.store.path/f'phase-{row["data"]["phase"]:04}'/row['data']['file'];value=b.parse(path.read_bytes());value['data']['expiry']['finalizedHeight']+=1;raw=b.encode(value);path.write_bytes(raw);rows=b.parse(b.encode(h.store.rows));tail=None;anchors=[]
        for r in rows:
            d=r['data']
            if r['kind']=='FILE' and d.get('phase')==row['data']['phase'] and d.get('name')==path.name:d.update(sha256=b.sha(raw),bytes=len(raw))
            if r['kind']=='RETIRED_UNSENT':d['sha256']=b.sha(raw)
            r['previous']=tail;tail=b.sha(b.encode(r));anchors.append(b.encode({'sequence':r['sequence'],'digest':tail,'previous':r['previous']}))
        journal.write_bytes(b''.join(b.encode(r) for r in rows));anchor.write_bytes(b''.join(anchors))
        with self.assertRaisesRegex(b.Stop,'replay_retired'):h.reopen()
        h.store.close();journal.write_bytes(journal.read_bytes()+b'{')
        with self.assertRaisesRegex(b.Stop,'partial_journal'):m.RetiringStore(h.store.path,h.package_sha,h.authorization_sha,h.store.anchor_path)
    def test_normal_exit_directory_replacement_fails_fresh_source_guard(self):
        h=RetirementHarness(self.tmp.name,'healthy');directory=Path(h.p['normalExitDirectory']);directory.rename(directory.with_name('displaced-exits'));directory.mkdir(mode=0o700)
        with mock.patch.object(m.UploadExecution,'source_audit',return_value=None):
            with self.assertRaisesRegex(b.Stop,'exit_directory_security'):m.RetiringUploadExecution.source_audit(h)
        h.store.close()
    def test_real_public_batch_checker_fixture_and_bad_signature(self):
        h=RetirementHarness(self.tmp.name,'healthy');fixture=b.parse(b.read(b.REPO/'clients/cli/startup-public-check.fixture.json'))['transactions'][0];wire=base64.b64decode(fixture['signedWireBase64']);message=wire[1+64*wire[0]:];at=4+32*message[3];worker=b.Sdk(h.sources['tools/startup_two_operation.cjs'],0,message)
        try:reviewed=worker.call({'action':'BIND','blockhash':h.c.b58encode(message[at:at+32])})['reviewed']
        finally:worker.close()
        checked=m.RetiringUploadExecution._verify_many_public(h,[{'signedWireBase64':fixture['signedWireBase64'],'reviewed':reviewed}]);self.assertEqual(checked[0]['wireSha256'],fixture['signedWireSha256']);bad=bytearray(wire);bad[1]^=1
        with self.assertRaisesRegex(b.Stop,'public_replay_crypto'):m.RetiringUploadExecution._verify_many_public(h,[{'signedWireBase64':b.b64(bad),'reviewed':reviewed}])
        h.store.close()

if __name__=='__main__':unittest.main()
