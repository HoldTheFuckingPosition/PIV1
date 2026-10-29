"""Focused public SDK/static-plan tests; no native keys or network.

The lifecycle retains actual SDK preparation, binding and CHECK dispatch; fake
crypto admission/native/TLS and the SDK signed VERIFY call are mocked. Its
elapsed time excludes that VERIFY IPC/source/SDK work and cannot qualify a live
hot path. Separate component tests exercise the real public Ed25519 checker.
"""
import base64
import copy
import json
from pathlib import Path
import time
import unittest
from unittest import mock

from tools import startup_minimal_buffer as m
from tools import test_startup_release_operator as fixtures
o=fixtures.o


class MinimalBufferSdkTests(unittest.TestCase):
    setUpClass=classmethod(fixtures.CandidateGovernedFundingHandoffTests.setUpClass.__func__)

    def client(self):
        a,j,_,domain,unchanged=o._candidate_preparation_sources()
        client=o._candidate_buffer_sdk(j,self.fixture_type.manifest,unchanged,domain)
        self.addCleanup(client.close)
        return client,a,j

    def test_actual_sdk_exact_zero_wires_and_real_historical_signature_checks(self):
        client,a,j=self.client();manifest=self.fixture_type.manifest
        for index in (0,1):
            expected=o._candidate_prefix_review(a,self.c,manifest,index,self.a.candidate_oracle()['roles']['vault0'])
            wire=client.bind(index,self.a.candidate_oracle()['roles']['vault0'],expected)
            raw=base64.b64decode(wire);count=len(expected['requiredSigners'])
            self.assertEqual(raw,bytes([count])+bytes(64*count)+base64.b64decode(expected['messageBase64']))
        reviewed,life,_=fixtures.phase_from_fixture(self.c,0);tx=fixtures.FIXTURE['transactions'][0]
        phase={'id':'historical_public_fixture','packet':{'signedWireBase64':tx['signedWireBase64'],'reviewed':reviewed},
            'lifetime':life,'feeQuoteLamports':tx['feeLamports'],'feeCapLamports':tx['feeLamports'],'expectedPoststate':{}}
        value={'schema':'piv1-startup-public-check-v1','frozen':{'packageSha256':'a'*64,'payloadSha256':'b'*64,
            'anchorUnixTimestamp':None,'transactionIndex':'0'},'cumulativeFeeCapLamports':tx['feeLamports'],'phases':[phase],'trace':[]}
        good=client.request({'action':'CHECK','inputs':[value]})[0]
        self.assertEqual(good['reason'],'PENDING_OR_UNKNOWN_NO_REPLACEMENT')
        damaged=copy.deepcopy(value);wire=bytearray(base64.b64decode(tx['signedWireBase64']));wire[1]^=1
        damaged['phases'][0]['packet']['signedWireBase64']=base64.b64encode(wire).decode()
        bad=client.request({'action':'CHECK','inputs':[damaged]})[0]
        self.assertEqual(bad['reason'],'signature_invalid')
        self.assertEqual(client.request({'action':'CHECK','inputs':[value]})[0],good)  # No stale result.

    def test_static_rebuild_is_private_and_checks_full_input_source_and_artifact(self):
        client,a,j=self.client();client.install_checks(a,j)
        rebuilt=a.candidate_assemble
        with mock.patch.object(a,'candidate_assemble',wraps=rebuilt)as assemble:
            client.prepare_manifest(a,self.fixture_type.manifest)
            original=a.candidate_validate(self.fixture_type.manifest)
            original['phases'][0]['index']=935
            self.assertEqual(a.candidate_validate(self.fixture_type.manifest)['phases'][0]['index'],0)
            self.assertEqual(assemble.call_count,1)
            altered=copy.deepcopy(self.fixture_type.manifest);altered['phases'][0]['index']=935
            with self.assertRaisesRegex(ValueError,'static_manifest_changed'):a.candidate_validate(altered)
            with mock.patch.object(a,'_candidate_image',side_effect=ValueError('artifact_guard')):
                with self.assertRaisesRegex(ValueError,'artifact_guard'):a.candidate_validate(self.fixture_type.manifest)
            with mock.patch.object(client,'unchanged',side_effect=ValueError('fresh_source_guard')):
                with self.assertRaisesRegex(ValueError,'fresh_source_guard'):a.candidate_validate(self.fixture_type.manifest)

    def test_exact_python_load_pin_refuses_changed_bytes_before_exec(self):
        _,j,_,domain,_=o._candidate_preparation_sources()
        with mock.patch.object(Path,'read_bytes',return_value=b'raise RuntimeError("unverified_code_executed")\n'):
            with self.assertRaisesRegex(ValueError,'minimal_sdk_source_pin'):
                o._candidate_buffer_sdk(j,self.fixture_type.manifest,lambda _:None,domain)

    def test_worker_original_once_invalid_signature_and_source_drift_fail_closed(self):
        client,a,j=self.client();expected=o._candidate_prefix_review(a,self.c,self.fixture_type.manifest,0,self.a.candidate_oracle()['roles']['vault0'])
        unsigned=client.bind(0,self.a.candidate_oracle()['roles']['vault0'],expected)
        with self.assertRaisesRegex(ValueError,'worker_unresolved|checker_stderr'):
            client.request({'action':'VERIFY','index':0,'wireBase64':unsigned})
        self.assertIsNotNone(client.process.poll());self.assertTrue(client.closed)
        other,a,j=self.client();other.bind(0,self.a.candidate_oracle()['roles']['vault0'],expected)
        with self.assertRaisesRegex(ValueError,'worker_unresolved|checker_stderr'):
            other.bind(0,self.a.candidate_oracle()['roles']['vault0'],expected)
        drift,a,j=self.client()
        read=Path.read_bytes
        def changed(path):
            raw=read(path)
            return raw+b' ' if path.name=='startup_minimal_buffer.cjs' else raw
        with mock.patch.object(Path,'read_bytes',changed):
            with self.assertRaisesRegex(ValueError,'source_changed'):drift.guard('source_drift_fixture')

    def test_worker_stderr_and_extra_frame_are_rejected_and_reaped(self):
        real=m.subprocess.Popen
        for fault in ('stderr','extra_frame'):
            script=("process.stdin.once('data',()=>{require('node:fs').writeSync(2,'fixture');"
                "require('node:fs').writeSync(1,JSON.stringify({result:{sdk:'1.98.4',preparedPhases:[0,1]}})+'\\n');});"
                if fault=='stderr' else
                "process.stdin.once('data',()=>{require('node:fs').writeSync(1,'{\\\"result\\\":{}}\\n{\\\"result\\\":{}}\\n');});")
            processes=[]
            def launch(argv,**kwargs):
                process=real([str(self.j.NODE),'-e',script],**kwargs);processes.append(process);return process
            _,j,_,_,unchanged=o._candidate_preparation_sources()
            with mock.patch.object(m.subprocess,'Popen',side_effect=launch):
                with self.assertRaisesRegex(ValueError,'checker_stderr|unsolicited_reply|worker_unresolved'):
                    m.BufferSdkClient(o,j,self.fixture_type.manifest,unchanged)
            self.assertIsNotNone(processes[0].poll())


class MinimalBufferLifecycleTests(unittest.TestCase):
    setUpClass=classmethod(fixtures.CandidateGovernedFundingHandoffTests.setUpClass.__func__)
    setUp=fixtures.CandidateGovernedFundingHandoffTests.setUp
    build_parent=fixtures.CandidateGovernedFundingHandoffTests.build_parent
    positive_checker=fixtures.CandidateGovernedFundingHandoffTests.positive_checker
    call=fixtures.CandidateGovernedFundingHandoffTests.call
    buffer_request=fixtures.CandidateGovernedBufferPhaseTests.buffer_request
    boundaries=fixtures.CandidateGovernedBufferRangeTests.boundaries

    def test_dedicated_two_phase_mock_lifecycle_static_before_hash_and_full_reopen(self):
        real=o._candidate_buffer_sdk;events=[];clients=[]
        def factory(j,manifest,unchanged,domain):
            client=real(j,manifest,unchanged,domain);clients.append(client);events.append(('SDK_PREPARED',len(calls)))
            request=client.request
            def public_crypto(value):
                events.append((value['action'],len(calls)))
                if value['action']=='CHECK':
                    reports=request(value)  # Retain real IPC/source/crypto-negative work.
                    self.assertTrue(all(report.get('reason') in ('zero_signature','signature_invalid')for report in reports))
                    return [{'reason':'ALL_DECLARED_PHASES_FINALIZED_WITH_EXPECTED_POSTSTATE' if row['trace'][-1]['state']=='FINALIZED'
                        else 'PENDING_OR_UNKNOWN_NO_REPLACEMENT'}for row in value['inputs']]
                if value['action']=='VERIFY':
                    wire=base64.b64decode(value['wireBase64']);message=wire[1+64*wire[0]:]
                    return {'wireSha256':o.sha(wire),'messageSha256':o.sha(message),'signatureHex':wire[1:65].hex()}
                return request(value)
            client.request=public_crypto
            return client
        with self.positive_checker(),self.boundaries()as(calls,native,sent,active,_),mock.patch.object(o,'_candidate_buffer_sdk',side_effect=factory):
            self.call('CREATE_HANDOFF',self.manifest);request=self.buffer_request();started=time.monotonic()
            iterator=o._candidate_governed_minimal_buffer(request,self.path,expected_handoff=self.anchor,
                _native_profile_path='/not-read/public-fixture')
            rows=list(iterator);elapsed=time.monotonic()-started
            self.assertEqual([row['startupCursor']for row in rows],[1,2])
            self.assertEqual((len(native),len(sent)),(2,2));self.assertEqual(events[0],('SDK_PREPARED',0))
            self.assertEqual(len([event for event in events if event[0]=='BIND']),2)
            self.assertEqual(len([event for event in events if event[0]=='VERIFY']),2)
            self.assertTrue(clients[0].closed);self.assertIsNotNone(clients[0].process.poll())
            self.assertEqual([len(row['request']['packets'][0]['requiredSigners'])for row in native],[2,1])
            self.tail=rows[-1]['tail'];self.assertEqual(self.call('INSPECT')['startupCursor'],2)
            before=len(calls)
            with self.assertRaisesRegex(ValueError,'minimal_buffer_exact_range'):
                list(o._candidate_governed_minimal_buffer({**request,'expectedTail':self.tail},self.path,
                    expected_handoff=self.anchor,_native_profile_path='/not-read/public-fixture'))
            self.assertEqual(len(calls),before)
            print('MINIMAL_BUFFER_ORCHESTRATION_METRICS '+json.dumps({'elapsedSeconds':elapsed,'nativeMocks':len(native),
                'sendMocks':len(sent),'rpcMockBatches':len(calls),'cursor':2,'realWorkerCheckDispatch':True,
                'workerSignedVerifyMocked':True,'scope':'OFFLINE_ORCHESTRATION_NOT_REPRESENTATIVE_LIFETIME'}))
