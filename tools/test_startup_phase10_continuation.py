"""Synthetic host controller tests; no native signatures, keys, RPC or processes.

Real successor Journal/replay, suffix Journal/operate, canonical949 manifest,
full Buffer predicates, raw RPC receipts, clocks, at-most-once and recovery run.
Explicit seams: synthetic ELF/account baseline, historical pin overrides, fake
signature checker and pure review decoder. These are not live/control proof.
"""
import base64
import copy
import importlib.util
import json
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest import mock

HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location('continuation_fixture',HERE/'test_startup_phase10_exception.py')
t=importlib.util.module_from_spec(spec);spec.loader.exec_module(t)
spec=importlib.util.spec_from_file_location('continuation',HERE/'startup_phase10_continuation.py')
s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s)
RAW_CORE=t.e.successor_core


class ContinuationTests(unittest.TestCase):
    result=t.Phase10ExceptionTests.result
    repin_chain=t.Phase10ExceptionTests.repin_chain
    make_authorization=t.Phase10ControllerTests.make_authorization
    signer=t.Phase10ControllerTests.signer
    run_action=t.Phase10ControllerTests.run_action
    tail=t.Phase10ControllerTests.tail

    def setUp(self):
        t.Phase10ControllerTests.setUp(self)
        self.patch=mock.patch.object(s,'e',t.e);self.patch.start();self.addCleanup(self.patch.stop)
        self.suffix_completed=11;self.suffix_mode=False;self.suffix_response_drift=False;self.suffix_balance_skew=0
        clock_key=self.oracle['roles']['clock'];self.oracle['baseline'][clock_key].update(dataBytes=40,lamports='1')
        self.parent=Path(self.tmp.name)/'original-public-parent';self.parent.mkdir(mode=0o700);(self.parent/'lock').write_bytes(self.core.helpers()[3].LOCK_BYTES);(self.parent/'lock').chmod(0o600)
        j=self.core.helpers()[3];chain=[];meta=(self.parent/'lock').stat();identity={'device':str(meta.st_dev),'inode':str(meta.st_ino),'ctimeNs':str(meta.st_ctime_ns)}
        for index in range(12):
            body={'schema':j.RECORD_SCHEMA,'sequence':str(index),'previousDigest':chain[-1]['digest'] if chain else None,'lockIdentity':identity,
                  'kind':'BEGIN' if index==0 else 'EVENT','data':{'id':'synthetic','input':{'trace':[],'cumulativeFeeCapLamports':'0'}} if index==0 else {'synthetic':index}}
            record={**body,'digest':t.e.sha(t.e.encode(body))};chain.append(record);(self.parent/f'{index:08}.json').write_bytes(t.e.encode(record));(self.parent/f'{index:08}.json').chmod(0o600)
        self.parent_tail=j.tail(chain)
        self.adapter=RAW_CORE().startup_manifest();self.adapter.ELF_SHA=t.e.ELF_SHA;self.adapter.oracle=lambda:self.oracle
        self.manifest=self.adapter.assemble(self.core.operator_api(),j,self.image,package_sha256='c'*64,parent_tail=self.parent_tail,historical_ledger={'actualFeeLamports':'0','reservedFeeLamports':'0','cumulativeFeeCapLamports':'0'})
        # Assemble's artifact remains the real source fixture contract; image
        # identity alone is synthetic, as in existing manifest host tests.
        self.records[0]['data'].update(manifest=self.manifest,baseParentPath=str(self.parent))
        self.auth['manifestSha256']=t.e.sha(t.e.encode(self.manifest));self.auth['baseParentTail']=self.parent_tail
        self.auth_raw=t.e.encode(self.auth);t.e.AUTH_SHA=t.e.sha(self.auth_raw)
        self.stop['originalAuthorizationSha256']=t.e.AUTH_SHA;self.stop_raw=t.e.encode(self.stop);t.e.STOP_SHA=t.e.sha(self.stop_raw)
        t.e.MANIFEST_SHA=t.e.sha(t.e.encode(self.manifest))
        self.records[52]['data']['intent']['authorizationSha256']=t.e.AUTH_SHA
        self.repin_chain()
        self.journal=str(Path(self.tmp.name)/('au-phase10-'+t.e.TAIL+'.exception'))
        self.p=self.result();self.p['preparerSourceSha256']=t.e.sha((HERE/'startup_phase10_exception.py').read_bytes())
        self.value={'schema':'piv1-au-phase10-package-v1','evidence':['synthetic']*3,'proposal':self.p,'bridgeSourceSha256':t.e.sha((HERE/t.e.BRIDGE).read_bytes())}
        self.make_authorization()
        patch=mock.patch.object(t.e,'records_at',side_effect=lambda path:copy.deepcopy(self.records));patch.start();self.addCleanup(patch.stop)
        def core_factory():
            core=RAW_CORE();adapter=core.startup_manifest();adapter.ELF_SHA=t.e.ELF_SHA;adapter.oracle=lambda:self.oracle
            core.startup_manifest=lambda **kw:adapter
            core.check_signed=lambda *args:{'reason':'SYNTHETIC_CRYPTO_SEAM_ONLY'}
            def describe(api,packets,**kw):
                reviews=[]
                for packet in packets:
                    wire=base64.b64decode(packet['wireBase64']);message=wire[1+64*wire[0]:]
                    # Pure fixture decoder for the source-selected Write11/12.
                    offset=int.from_bytes(message[-968:-964],'little')
                    self.assertEqual(offset%956,0)
                    review=adapter.write_review(self.image,offset//956+1)
                    reviews.append({**review,'messageBase64':base64.b64encode(message).decode(),'messageSha256':t.e.sha(message)})
                return reviews
            core.describe=describe
            return core
        patch=mock.patch.object(t.e,'successor_core',side_effect=core_factory);patch.start();self.addCleanup(patch.stop)
        # Production continuation loads a new actual790 module. Alter only the
        # explicit synthetic fixture globals after loading that genuine source.
        original_module=s.importlib.util.module_from_spec
        def module(spec):
            result=original_module(spec)
            if spec.name=='suffix_canonical_manifest':
                # exec later sets these globals, so patch via loader wrapper at
                # the module validation entry before its first image use.
                class FixtureModule(type(result)):
                    def __getattribute__(obj,name):
                        if name in ('validate','expected'):
                            obj.__dict__['ELF_SHA']=t.e.ELF_SHA;obj.__dict__['oracle']=lambda:self.oracle
                        return super().__getattribute__(name)
                result.__class__=FixtureModule
            return result
        patch=mock.patch.object(s.importlib.util,'module_from_spec',side_effect=module);patch.start();self.addCleanup(patch.stop)

    def snapshot(self):
        accounts=t.Phase10ControllerTests.snapshot(self)
        if self.suffix_mode:
            r=self.oracle['roles'];written=(self.suffix_completed-1)*956
            raw=t.e.struct.pack('<IB',1,1)+base64.b64decode(self.oracle['writeTemplate']['prefixBase64'])[4:36]+self.image[:written]+bytes(t.e.ELF_BYTES-written)
            accounts[r['loaderBuffer']]['data']=[base64.b64encode(raw).decode(),'base64']
            accounts[r['g0']]['lamports']-=5000*(self.suffix_completed-11)+self.suffix_balance_skew
        return accounts

    def transport(self,raw):
        result=t.Phase10ControllerTests.transport(self,raw)
        if self.suffix_mode and any(q['method']=='sendTransaction' for q in json.loads(raw)):self.suffix_completed=12
        return result

    def finalized(self,debit=55000):
        self.debit=debit
        created=self.run_action('CREATE');sent=self.run_action('CAPTURE_SIGN_SEND',created['tail']);self.recovered=True
        terminal=self.run_action('RECOVER',sent['tail']);self.assertEqual(terminal['status'],'FINALIZED')
        return terminal['tail']

    def suffix(self,tail):
        self.suffix_value=s.package(self.value,self.journal,tail)
        self.suffix_journal=str(Path(self.tmp.name)/('au-phase10-'+t.e.TAIL+'.continuation'))
        manifest,_=s.checked(self.suffix_value);expiry=str(int(t.e.time.time())+3600)
        inner={'schema':'piv1-launcher-release-authorization-v2','manifestSha256':s.sha(s.encode(manifest)),'baseParentTail':self.parent_tail,
               'operatorSourceSha256':t.e.PINS['startup_release_operator.py'],'expiresUnixTimestamp':expiry,'scope':'EXACT_BOUNDED_TESTNET_RELEASE',
               'feeBudgetNamespace':manifest['proposedFeeBudget']['namespace'],'signerBackendSha256':t.e.PINS['startup_signer_backend.cjs'],'signerProfileSha256':'b'*64}
        raw=s.encode(inner)
        self.outer={'schema':s.AUTH_SCHEMA,'scope':'EXACT_PHASE11_948_AFTER_NEW_PHASE10_FINALIZED','packageSha256':s.sha(s.encode(self.suffix_value)),
            'journalPath':self.suffix_journal,'baseParentPath':str(self.parent),'sourcePins':self.suffix_value['sourcePins'],'phase10Tail':tail,'originalTail':t.e.TAIL,
            'expiresUnixTimestamp':expiry,'transportMode':'OFFLINE_INJECTED_TEST','backendAuthorizationRawBase64':base64.b64encode(raw).decode(),'backendAuthorizationSha256':s.sha(raw),
            'manifestSha256':s.sha(s.encode(manifest)),'carriedPossibleFeeLamports':'5000','combinedFeeCapLamports':'4820000'}
        self.suffix_mode=True
        self.packet={'id':'startup_0011','wireBase64':base64.b64encode(bytes([1])+bytes(64)+base64.b64decode(manifest['phases'][11]['reviewed']['messageBase64'])).decode()}

    def suffix_signer(self,payload):
        self.assertIn('outerAuthorizationRawBase64',payload);value=payload['input'];request=value['request'];wire=bytearray(base64.b64decode(value['packets'][0]['wireBase64']));wire[1:65]=bytes([10])*64
        self.signed_wire=base64.b64encode(wire).decode();self.signature=t.e.b58(wire[1:65])
        if self.suffix_response_drift:
            patch=mock.patch.object(t.e.time,'time',return_value=int(self.outer['expiresUnixTimestamp'])+1);patch.start();self.addCleanup(patch.stop)
        return {'schema':'piv1-startup-signing-response-v1','requestId':request['requestId'],'packets':[{'id':'startup_0011','wireBase64':self.signed_wire}]}

    def call(self,action,tail=None,data=None,**kw):
        raw=s.encode(self.outer)
        return s.controller({'schema':s.SCHEMA,'action':action,'expectedTail':tail,'data':data},self.suffix_value,self.suffix_journal,
            authorization_raw=raw,authorization_sha256=s.sha(raw),live=True,live_sign=True,_transport=self.transport,_signer=self.suffix_signer,**kw)

    def test_real_controller_phase11_send_recover_and_exact_two_debit_branches(self):
        for debit in (55000,60000):
            with self.subTest(debit=debit):
                if debit==60000:self.doCleanups();self.setUp()
                tail=self.finalized(debit);self.suffix(tail);before=copy.deepcopy(self.records)
                created=self.call('CREATE');self.assertEqual(created['completedPhases'],11);self.assertEqual(created['combinedReservedFeeLamports'],'65000')
                sent=self.call('CAPTURE_SIGN_SEND',created['tail'],{'packet':self.packet,'signerProfilePath':'/synthetic-unread-profile.json'})
                self.assertEqual(sent['completedPhases'],11);self.assertEqual(sent['combinedReservedFeeLamports'],'70000')
                done=self.call('RECOVER',sent['tail'],{'id':'startup_0011'})
                self.assertEqual(done['completedPhases'],12);self.assertEqual(done['combinedReservedFeeLamports'],'70000');self.assertFalse(done['carriedOriginalUnknown']['reservationReleased'])
                self.assertEqual(self.records,before)
                review=self.adapter.write_review(self.image,12)
                next_packet={'id':'startup_0012','wireBase64':base64.b64encode(bytes([1])+bytes(64)+base64.b64decode(review['messageBase64'])).decode()}
                prepared=self.call('PREPARE',done['tail'],{'captureLifetime':True,'packets':[next_packet]})
                handoff=self.call('HANDOFF',prepared['tail'],['startup_0012'])
                self.assertEqual(handoff['combinedReservedFeeLamports'],'70000')
                with self.assertRaisesRegex(ValueError,'compound_current_phase_required'):
                    self.call('CAPTURE_SIGN_SEND',handoff['tail'],{'packet':self.packet,'signerProfilePath':'/synthetic-unread-profile.json'})

    def test_nonterminal_wrong_tail_and_parent_tamper_cannot_handoff(self):
        created=self.run_action('CREATE');sent=self.run_action('CAPTURE_SIGN_SEND',created['tail'])
        with self.assertRaisesRegex(ValueError,'phase10_finalized_required'):s.package(self.value,self.journal,sent['tail'])
        self.recovered=True;done=self.run_action('RECOVER',sent['tail'])
        with self.assertRaisesRegex(ValueError,'phase10_tail_changed'):s.package(self.value,self.journal,'0'*64)
        path=self.parent/'00000011.json';value=json.loads(path.read_bytes());value['data']['synthetic']=99;path.write_bytes(s.encode(value))
        with self.assertRaisesRegex(ValueError,'record_digest'):s.package(self.value,self.journal,done['tail'])

    def test_fresh_scope_and_fake_mode_refuse_before_new_namespace(self):
        self.suffix(self.finalized())
        bad=copy.deepcopy(self.outer);self.outer['carriedPossibleFeeLamports']='0'
        with self.assertRaisesRegex(ValueError,'authorization_binding'):self.call('CREATE')
        self.assertFalse(Path(self.suffix_journal).exists());self.outer=bad
        raw=s.encode(self.outer)
        with self.assertRaisesRegex(ValueError,'transport_mode_changed'):
            s.controller({'schema':s.SCHEMA,'action':'CREATE','expectedTail':None,'data':None},self.suffix_value,self.suffix_journal,authorization_raw=raw,authorization_sha256=s.sha(raw))
        self.assertFalse(Path(self.suffix_journal).exists())

    def test_callback_expiry_retains_original_public_response_before_refusal(self):
        self.suffix(self.finalized());created=self.call('CREATE');self.suffix_response_drift=True
        with self.assertRaisesRegex(ValueError,'authorization_expired'):
            self.call('CAPTURE_SIGN_SEND',created['tail'],{'packet':self.packet,'signerProfilePath':'/synthetic-unread-profile.json'})
        records=sorted(Path(self.suffix_journal).glob('*.json'))
        self.assertEqual(json.loads(records[-1].read_bytes())['kind'],'PREPARE_SIGN_ATTEMPT')
        response=Path(self.suffix_journal+'.evidence')/'00000002-send'/'signer-response.json'
        self.assertEqual(json.loads(response.read_bytes())['packets'][0]['id'],'startup_0011')
        self.assertEqual(self.suffix_completed,11)

    def test_exact_g0_predicate_refuses_one_lamport_drift_without_send(self):
        self.suffix(self.finalized(60000));created=self.call('CREATE');self.suffix_balance_skew=1
        sends=self.sends
        with self.assertRaisesRegex(ValueError,'prestate'):
            self.call('CAPTURE_SIGN_SEND',created['tail'],{'packet':self.packet,'signerProfilePath':'/synthetic-unread-profile.json'})
        self.assertEqual(self.sends,sends);self.assertEqual(self.suffix_completed,11)


if __name__=='__main__':unittest.main()
