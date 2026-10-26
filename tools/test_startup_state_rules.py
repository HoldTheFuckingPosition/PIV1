"""Focused public-byte tests; no new keys/signatures or actual transport.

The ProgramData operator integration deliberately uses a synthetic ELF hash seam
and an existing historical signed wire with retrospective injected state. It
tests consumed rules, not an actual bootstrap or verification of the selected ELF.
Root separately compares the unchanged real ELF/runtime evidence.
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
spec=importlib.util.spec_from_file_location("operator_fixture_helpers",HERE/"test_startup_release_operator.py")
h=importlib.util.module_from_spec(spec); spec.loader.exec_module(h)
o=h.o
RAW=(HERE/"startup_state_rules.fixture.json").read_bytes()
assert o.sha(RAW)=="666d0c12b8b272bfd4051f1a9612697db8e8ac9133ff7e3f204ae9d928795da7"
F=json.loads(RAW)


class StartupStateTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        h.OperatorTests.setUpClass()
        cls.r=o.state_rules(); cls.c=h.OperatorTests.c; cls.j=h.OperatorTests.j

    def setUp(self):
        self.state={"seal":{"anchorUnixTimestamp":"0","stateBinding":copy.deepcopy(F["binding"])},"runtimeFacts":{}}
        self.phase={"kind":"INITIALIZER","initializerIndex":8,"expectedPrestate":{},"expectedPoststate":{}}

    def account(self,owner,raw,lamports=None):
        return {"exists":True,"owner":owner,"executable":False,"lamports":lamports or self.r.RENT_FLOORS[len(raw)],"dataBase64":base64.b64encode(raw).decode()}

    def rule(self,kind,size): return {"startupRule":kind,"lamports":self.r.RENT_FLOORS[size]}

    def clock(self,slot,timestamp,context=None):
        return self.r.clock(self.c,self.account(self.r.SYSVAR,struct.pack("<QqQQq",slot,0,0,0,timestamp),"1"),str(context or slot),0)

    def proposal(self,count,status,timestamp):
        old=bytes.fromhex(F["goldenProposal"]["data_hex"])
        keys=sorted(F["binding"]["postManifest"]["initializer"]["governance"]["approvers"][:count],key=lambda k:self.c.b58decode(k,32))
        raw=old[:48]+bytes([status])+struct.pack("<q",timestamp)+old[57:58]+struct.pack("<I",count)+b"".join(self.c.b58decode(k,32) for k in keys)+bytes(8)
        return self.account(self.r.SQUADS,raw+bytes(646-len(raw)))

    def check_proposal(self,index,count,status,timestamp,state=None):
        phase={"kind":"INITIALIZER","initializerIndex":index,"expectedPrestate":{},"expectedPoststate":{}}
        key=F["binding"]["plan"]["proposal"]; phase["expectedPoststate"]={key:self.rule("PROPOSAL",646)}
        facts=self.r.verify(self.c,state or self.state,phase,"POST",phase["expectedPoststate"],{key:self.proposal(count,status,timestamp)},inclusion_slot="11",pre_clock=self.clock(10,100),post_clock=self.clock(12,200))
        self.r.transition(state or self.state,phase,facts,11,self.clock(10,100),self.clock(12,200))
        return facts

    def test_all16_targets_equal_retained_literal_full_bytes(self):
        actual=self.r.target_bytes(self.c,self.state)
        self.assertEqual(len(actual),16)
        for row in F["goldenTargets"]:
            self.assertEqual(actual[row["key"]],(row["owner"],bytes.fromhex(row["data_hex"])))
            self.assertEqual(str(row["lamports"]),self.r.RENT_FLOORS[len(bytes.fromhex(row["data_hex"]))])

    def test_real_sealed_anchor_and_exact_principal_authorities_and_padding(self):
        self.state["seal"]["anchorUnixTimestamp"]="1234567890"
        self.state["seal"]["stateBinding"]["postManifest"]["initializer"]["initialization"]["anchorUnixTimestamp"]="1234567890"
        target=F["targets"][0]; row=next(x for x in F["goldenTargets"] if x["key"]==target["address"])
        raw=bytearray.fromhex(row["data_hex"]); raw[910:918]=struct.pack("<q",1234567890)
        rule={target["address"]:self.rule("INITIALIZED_TARGET",1014)}
        self.r.verify(self.c,self.state,self.phase,"POST",rule,{target["address"]:self.account(row["owner"],bytes(raw))})
        for offset in (342,766,998):
            altered=bytearray(raw); altered[offset]^=1
            with self.assertRaisesRegex(ValueError,"startup_target_bytes"):
                self.r.verify(self.c,self.state,self.phase,"POST",rule,{target["address"]:self.account(row["owner"],bytes(altered))})
        with self.assertRaisesRegex(ValueError,"startup_frozen_rent_floor"):
            self.r.verify(self.c,self.state,self.phase,"POST",rule,{target["address"]:self.account(row["owner"],bytes(raw)+b"\0",rule[target["address"]]["lamports"])})

    def test_program_data_exact_receipt_slot_authority_image_and_persisted_fact(self):
        image=bytes(894552); phase={"kind":"BOOTSTRAP"}; expected={self.r.PROGRAM_DATA:self.rule("PROGRAM_DATA",894597)}
        raw=struct.pack("<IQB",3,91,1)+self.c.b58decode(self.r.VAULT,32)+image
        with mock.patch.object(self.r,"ELF_SHA",o.sha(image)):
            facts=self.r.verify(self.c,self.state,phase,"POST",expected,{self.r.PROGRAM_DATA:self.account(self.r.LOADER,raw)},inclusion_slot="91")
            self.assertEqual(facts,{"deploymentSlot":"91"})
            later={**self.state,"runtimeFacts":facts}
            self.r.verify(self.c,later,{"kind":"UPLOAD_WRITE"},"PRE",expected,{self.r.PROGRAM_DATA:self.account(self.r.LOADER,raw)})
            for offset in (4,13,45):
                bad=bytearray(raw); bad[offset]^=1
                with self.assertRaisesRegex(ValueError,"startup_program_data_bytes"):
                    self.r.verify(self.c,self.state,phase,"POST",expected,{self.r.PROGRAM_DATA:self.account(self.r.LOADER,bytes(bad))},inclusion_slot="91")

    def test_proposal_causal_create_intermediate_threshold_execute_timestamps(self):
        self.state["runtimeFacts"]=self.check_proposal(3,0,1,110)
        for index in (4,5,6):
            with self.assertRaisesRegex(ValueError,"startup_proposal_timestamp_changed"):
                self.check_proposal(index,index-3,1,111)
            self.state["runtimeFacts"]=self.check_proposal(index,index-3,1,110)
        self.state["runtimeFacts"]=self.check_proposal(7,4,3,160)
        self.state["runtimeFacts"]=self.check_proposal(8,4,5,180)
        self.assertEqual(self.state["runtimeFacts"]["proposal"],{"timestamp":"180","status":5,"approvedCount":4})
        with self.assertRaisesRegex(ValueError,"startup_proposal_prior_fact"):
            self.check_proposal(7,4,3,160,{**self.state,"runtimeFacts":{}})
        for invalid in ({"timestamp":None,"status":5,"approvedCount":4},{"timestamp":"180","status":True,"approvedCount":4}):
            with self.assertRaises(ValueError): self.r.facts_shape({"proposal":invalid})

    def test_heartbeat_one_period_only_and_proposal_preserved_on_both_sides(self):
        key=F["binding"]["plan"]["alt"]["addresses"][3]; proposal=F["binding"]["plan"]["proposal"]
        self.state["runtimeFacts"]={"proposal":{"timestamp":"180","status":5,"approvedCount":4}}
        phase={"kind":"HEARTBEAT","initializerIndex":None}
        expected={key:self.rule("HEARTBEAT_REWARD_0",84),proposal:self.rule("PROPOSAL",646)}
        phase["expectedPrestate"]=phase["expectedPoststate"]=expected
        baseline=self.r.target_bytes(self.c,self.state)[key][1]
        accounts={key:self.account(self.r.PROGRAM,baseline),proposal:self.proposal(4,5,180)}
        self.r.verify(self.c,self.state,phase,"PRE",expected,accounts)
        active=baseline[:51]+b"\1"+struct.pack("<Q",3)+bytes(24); accounts[key]=self.account(self.r.PROGRAM,active)
        pre=self.clock(10,3*self.r.PERIOD+20); post=self.clock(12,3*self.r.PERIOD+30)
        facts=self.r.verify(self.c,self.state,phase,"POST",expected,accounts,inclusion_slot="11",pre_clock=pre,post_clock=post)
        self.assertEqual(facts["heartbeatPeriod"],"3")
        self.r.transition(self.state,phase,facts,11,pre,post)
        with self.assertRaisesRegex(ValueError,"startup_heartbeat_ambiguous_period"):
            self.r.verify(self.c,self.state,phase,"POST",expected,accounts,inclusion_slot="11",pre_clock=self.clock(10,4*self.r.PERIOD-1),post_clock=self.clock(12,4*self.r.PERIOD+1))
        bad=bytearray(active); bad[60]=1; accounts[key]=self.account(self.r.PROGRAM,bytes(bad))
        with self.assertRaisesRegex(ValueError,"startup_heartbeat_bytes"):
            self.r.verify(self.c,self.state,phase,"POST",expected,accounts,inclusion_slot="11",pre_clock=pre,post_clock=post)

    def test_clock_identity_causality_window_and_checked_period_overflow(self):
        with self.assertRaisesRegex(ValueError,"startup_clock_window_regression"):
            self.r.window(self.clock(10,100,12),self.clock(13,200),11)
        with self.assertRaisesRegex(ValueError,"startup_clock_window_regression"):
            self.r.window(self.clock(10,200),self.clock(12,100),11)
        account=self.account(self.r.SYSTEM,bytes(40),"1")
        with self.assertRaisesRegex(ValueError,"startup_clock_identity"): self.r.clock(self.c,account,"1",0)
        with self.assertRaisesRegex(ValueError,"startup_period_overflow"): self.r.period(self.r.I64_MAX-1,self.r.I64_MAX)

    def test_operator_send_and_recover_consume_typed_rule_and_persist_facts(self):
        host=h.OperatorTests(methodName="test_nullable_imports_no_helpers"); host.setUp(); self.addCleanup(host.doCleanups)
        host.phase["kind"]="BOOTSTRAP"
        absent={"exists":False,"owner":None,"executable":None,"lamports":None,"dataBytes":None,"dataSha256":None}
        host.phase["expectedPrestate"]={self.r.PROGRAM_DATA:absent}
        host.phase["expectedPoststate"]={self.r.PROGRAM_DATA:self.rule("PROGRAM_DATA",894597)}
        host.manifest["sourcePins"]["stateRules"]=o.STATE_RULES_SHA
        slot=int(host.tx["slot"]); image=bytes(894552)
        data=struct.pack("<IQB",3,slot,1)+self.c.b58decode(self.r.VAULT,32)+image
        bad_post=[False]; bad_rent=[True]
        def transport(raw):
            replies=[]
            for query in json.loads(raw):
                method=query["method"]
                if method=="getMinimumBalanceForRentExemption":
                    host.calls.append(method); result=int(self.r.RENT_FLOORS[query["params"][0]])+(1 if bad_rent[0] else 0)
                elif method=="getMultipleAccounts":
                    host.calls.append(method); sent="sendTransaction" in host.calls
                    actual=bytearray(data)
                    if bad_post[0]: actual[13]^=1
                    account=self.account(self.r.LOADER,bytes(actual))
                    result={"context":{"slot":slot+1 if sent else slot-1},"value":[host.wire_account(account) if sent else None]}
                else:
                    _,_,body=host.transport(o.encode([query])); replies.extend(json.loads(body)); continue
                replies.append({"jsonrpc":"2.0","id":query["id"],"result":result})
            return 200,[],o.encode(replies)
        with mock.patch.object(o,"state_rules",return_value=self.r),mock.patch.object(self.r,"ELF_SHA",o.sha(image)):
            host.ready()
            with self.assertRaisesRegex(ValueError,"startup_fresh_rent_profile_changed"):
                host.call("SEND",{"id":host.phase["id"]},_transport=transport,**host.auth())
            host.refresh_tail(); self.assertNotIn("sendTransaction",host.calls); bad_rent[0]=False
            host.call("SEND",{"id":host.phase["id"]},_transport=transport,**host.auth()); bad_post[0]=True
            with self.assertRaisesRegex(ValueError,"startup_program_data_bytes"):
                host.call("RECOVER",{"id":host.phase["id"]},_transport=transport,**host.auth())
            host.refresh_tail(); self.assertEqual(host.call("INSPECT")["completedPhases"],0); bad_post[0]=False
            self.assertEqual(host.call("RECOVER",{"id":host.phase["id"]},_transport=transport,**host.auth())["status"],"ALL_PHASES_RECONCILED")
            records=[json.loads(p.read_bytes()) for p in sorted(o.journal_path(host.base).glob("*.json"))]
            self.assertEqual(o.replay(self.j,records)["runtimeFacts"],{"deploymentSlot":str(slot)})
            self.assertEqual(host.calls.count("sendTransaction"),1)

    def test_sdk_seal_binding_produces_actual_bumps_and_nonzero_anchor(self):
        pre=copy.deepcopy(F["binding"]["postManifest"]); pre["initializer"]["initialization"]["anchorUnixTimestamp"]=None
        host=h.OperatorTests(methodName="test_nullable_imports_no_helpers")
        with mock.patch.object(h,"CLOCK_FIXTURE",{"preManifest":pre}): host.setUp()
        self.addCleanup(host.doCleanups)
        host.manifest["phases"]=[{**copy.deepcopy(host.phase),"id":f"init_{i}","kind":"INITIALIZER","initializerIndex":i,"reviewed":None} for i in range(9)]
        config=F["targets"][0]["address"]
        host.manifest["phases"][8]["expectedPoststate"]={config:self.rule("INITIALIZED_TARGET",1014)}
        host.manifest["sourcePins"]["stateRules"]=o.STATE_RULES_SHA
        host.manifest["initializerPreManifestSha256"]=o.sha(self.j.encode(pre)); host.create()
        def boundary(action,tail,value): return self.j.apply({"schema":self.j.SCHEMA,"action":action,"expectedTail":tail,"segment":value,"event":None},host.base)
        prof=boundary("FREEZE_INITIALIZER_PROFILE",host.manifest["parentTail"],pre)
        floor=max(int(pre["initializer"]["transport"]["referenceSlot"]),int(pre["existingAlt"]["contextSlot"]),int(host.tx["slot"]))+1
        clock={"address":self.r.CLOCK,"owner":self.r.SYSVAR,"executable":False,"contextSlot":str(floor),"dataBase64":base64.b64encode(struct.pack("<QqQQq",floor,0,0,0,1234567890)).decode()}
        sealed=boundary("SEAL_INITIALIZER_CLOCK",prof["tail"],clock)
        host.call("BIND_SEAL",{"parentPath":str(host.base),"parentTail":sealed["tail"]},_transport=host.transport)
        records=[json.loads(p.read_bytes()) for p in sorted(o.journal_path(host.base).glob("*.json"))]
        bound=o.replay(self.j,records)
        for name in ("authorityBump","proposalBump","targetBumps"):
            self.assertEqual(bound["seal"]["stateBinding"][name],F["binding"][name])
        self.assertEqual(struct.unpack_from("<q",self.r.target_bytes(self.c,bound)[config][1],910)[0],1234567890)
        self.assertTrue(all((host.base/name).read_bytes()==raw for name,raw in host.base_bytes.items()))


class CandidateProgramDataTests(unittest.TestCase):
    """Synthetic image hash seam only; real ELF confirmation is separate evidence.

    Source loaders/pins remain real. No parent controller, journal or signing
    initialization is needed for these pure byte/scope checks.
    """

    @classmethod
    def setUpClass(cls):
        cls.r = o.state_rules()
        cls.a = o.startup_manifest()
        spec = importlib.util.spec_from_file_location('candidate_public_codec', HERE/'collect_startup_public_evidence.py')
        cls.c = importlib.util.module_from_spec(spec); spec.loader.exec_module(cls.c)

    def setUp(self):
        self.image = b'\x7f' * 892352
        actual_sha = self.r.sha
        self.patch = mock.patch.object(self.r, 'sha', side_effect=lambda raw:
            self.r.CANDIDATE_ELF_SHA if raw == self.image else actual_sha(raw))
        self.patch.start(); self.addCleanup(self.patch.stop)
        self.raw = struct.pack('<IQB', 3, 91, 1) + self.c.b58decode(self.r.VAULT, 32) + self.image
        self.account = {'exists': True, 'owner': self.r.LOADER, 'executable': False,
                        'lamports': '4534027000', 'dataBase64': base64.b64encode(self.raw).decode()}
        self.expected = {self.r.PROGRAM_DATA: self.r.candidate_program_data_rule()}

    def verify(self, *, index=935, stage='POST', account=None, prior_fact=None, slot='91', expected=None):
        return self.r.verify_candidate_program_data(
            self.c, self.a.candidate_phase(index), stage,
            self.expected if expected is None else expected,
            {self.r.PROGRAM_DATA: self.account if account is None else account},
            prior_fact=prior_fact, inclusion_slot=slot)

    def test_scoped_causal_fact_is_created_once_then_required_without_legacy_adoption(self):
        self.assertEqual(self.r.CANDIDATE_SCOPE, self.a.CANDIDATE_SCOPE)
        fact = self.verify()
        self.assertEqual(fact, {'candidateScope': self.r.CANDIDATE_SCOPE,
                               'elfSha256': self.r.CANDIDATE_ELF_SHA, 'deploymentSlot': '91'})
        self.assertEqual(self.verify(index=936, stage='PRE', prior_fact=fact, slot=None), fact)
        self.assertEqual(self.verify(index=946, prior_fact=fact, slot=None), fact)
        with self.assertRaisesRegex(ValueError, 'candidate_deployment_replay'):
            self.verify(prior_fact=fact)
        with self.assertRaisesRegex(ValueError, 'candidate_program_data_phase'):
            self.verify(stage='PRE')
        for prior in (None, {'deploymentSlot': '91'}, {**fact, 'candidateScope': self.a.PROFILE_SCHEMA},
                      {**fact, 'elfSha256': self.r.ELF_SHA}):
            with self.assertRaisesRegex(ValueError, 'candidate_deployment_fact'):
                self.verify(index=936, prior_fact=prior)
        changed = {**fact, 'deploymentSlot': '92'}
        with self.assertRaisesRegex(ValueError, 'candidate_program_data_bytes'):
            self.verify(index=936, prior_fact=changed)

    def test_default_legacy_rule_shape_and_verifier_refuse_candidate_bytes(self):
        self.assertNotIn(self.r.CANDIDATE_PROGRAM_DATA_RULE, self.r.KINDS)
        self.assertNotIn(892397, self.r.RENT_FLOORS)
        with self.assertRaisesRegex(ValueError, 'startup_rule_shape'):
            self.r.shape(self.c, None, self.r.candidate_program_data_rule())
        with self.assertRaisesRegex(ValueError, 'startup_frozen_rent_floor'):
            self.r.verify(self.c, {'runtimeFacts': {}}, {'kind': 'BOOTSTRAP'}, 'POST',
                          self.expected, {self.r.PROGRAM_DATA: self.account}, inclusion_slot='91')
        self.assertEqual(self.r.ELF_SHA, 'e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99')

    def test_full_image_header_authority_owner_flags_size_and_frozen_rent_are_exact(self):
        for offset in (0, 4, 12, 13, 45, len(self.raw)-1):
            raw = bytearray(self.raw); raw[offset] ^= 1
            changed = {**self.account, 'dataBase64': base64.b64encode(raw).decode()}
            with self.assertRaisesRegex(ValueError, 'candidate_program_data_bytes'):
                self.verify(account=changed)
        for field, value in (('owner', self.r.SYSTEM), ('executable', True), ('exists', False),
                             ('lamports', '4534027001')):
            with self.assertRaises(ValueError): self.verify(account={**self.account, field: value})
        for raw in (self.raw[:-1], self.raw+b'\0'):
            with self.assertRaisesRegex(ValueError, 'candidate_program_data_identity'):
                self.verify(account={**self.account, 'dataBase64': base64.b64encode(raw).decode()})
        for slot in (None, '0', '-1', '18446744073709551616'):
            with self.assertRaises(ValueError): self.verify(slot=slot)
        with self.assertRaisesRegex(ValueError, 'candidate_program_data_rule'):
            self.verify(expected={self.r.PROGRAM_DATA: {'startupRule': 'PROGRAM_DATA', 'lamports': '4545203000'}})

    def test_no_extra_account_mask_or_legacy_phase_descriptor_is_accepted(self):
        phase = self.a.candidate_phase(935)
        for changed in ({**phase, 'id': 'startup_0935'}, {**phase, 'kind': 'UPLOAD_WRITE'},
                        {**phase, 'candidateScope': self.a.PROFILE_SCHEMA}, {**phase, 'extra': None}):
            with self.assertRaisesRegex(ValueError, 'candidate_phase_shape'):
                self.r.verify_candidate_program_data(self.c, changed, 'POST', self.expected,
                    {self.r.PROGRAM_DATA: self.account}, inclusion_slot='91')
        for accounts in ({}, {self.r.PROGRAM_DATA: self.account, self.r.PROGRAM: self.account}):
            with self.assertRaisesRegex(ValueError, 'candidate_program_data_coverage'):
                self.r.verify_candidate_program_data(self.c, phase, 'POST', self.expected, accounts,
                                                    inclusion_slot='91')
        with self.assertRaisesRegex(ValueError, 'candidate_program_data_rule'):
            self.verify(expected={**self.expected, self.r.PROGRAM: self.expected[self.r.PROGRAM_DATA]})
        with self.assertRaisesRegex(ValueError, 'candidate_program_data_phase'):
            self.verify(index=934)


if __name__=="__main__": unittest.main()


class CandidateRuntimeHandoffTests(unittest.TestCase):
    """One host-only12-phase suffix: actual ELF, synthetic Clock/account bytes, no journals."""
    @classmethod
    def setUpClass(cls):
        cls.a=o.startup_manifest();cls.ref=cls.a.candidate_oracle();cls.j,cls.r,cls.c=cls.a._candidate_helpers()
        cls.image=Path(cls.ref['provenance']['elfPublicPath']).read_bytes();cls.manifest=cls.a.candidate_assemble(cls.image)
        cls.base=int(cls.ref['baselineContextSlot'])+100;cls.timestamp=1793016100
        cls.cases={};cls.results={};cls.binding=None;prior=None
        for i in range(935,947):
            for stage in ('PRE','POST'):
                offset=3*(i-935)+(2 if stage=='POST' else 0);slot=cls.base+offset;ts=cls.timestamp+2*(i-935)+(stage=='POST')
                clock=cls.account(cls.r.SYSVAR,struct.pack('<QqQQq',slot,0,0,0,ts),'1')
                if i==937 and stage=='PRE':
                    pre=copy.deepcopy(cls.ref['preManifest']);pre['initializer']['transport']['referenceSlot']=str(slot);pre['existingAlt']['contextSlot']=str(slot)
                    selected={k:clock[k]for k in ('owner','executable','dataBase64')};selected.update(address=cls.r.CLOCK,contextSlot=str(slot))
                    cls.binding=cls.j.candidate_clock_binding(pre,selected)
                accounts={cls.r.CLOCK:clock,cls.r.PROGRAM_DATA:cls.absent() if i==935 and stage=='PRE' else cls.account(cls.r.LOADER,struct.pack('<IQB',3,cls.base+1,1)+cls.c.b58decode(cls.r.VAULT,32)+cls.image,'4534027000')}
                if i>=940:
                    if i==940 and stage=='PRE':proposal=cls.absent()
                    else:
                        count=min(i-940-(stage=='PRE'),4);count=4 if i>=945 else count
                        status=5 if i>945 or i==945 and stage=='POST' else 3 if count==4 else 1
                        timestamp=cls.timestamp+(21 if status==5 else 19 if status==3 else 11)
                        g=cls.ref['preManifest']['initializer']['governance'];approved=sorted(g['approvers'][:count],key=lambda k:cls.c.b58decode(k,32))
                        raw=__import__('hashlib').sha256(b'account:Proposal').digest()[:8]+cls.c.b58decode(g['multisigAddress'],32)+struct.pack('<QBqB',1,status,timestamp,cls.ref['bumps']['proposalBump'])+struct.pack('<I',count)+b''.join(cls.c.b58decode(k,32)for k in approved)+bytes(8)
                        proposal=cls.account(cls.r.SQUADS,raw+bytes(646-len(raw)),cls.ref['rents']['646'])
                    accounts[cls.ref['roles']['proposal1']]=proposal
                if i>=945:
                    view={'seal':{'stateBinding':{'plan':cls.binding['unsignedInitializerPlan'],'postManifest':cls.binding['postManifest'],**cls.ref['bumps']},'anchorUnixTimestamp':cls.binding['clockDecoded']['unixTimestamp']}}
                    for address,(owner,raw) in cls.r.target_bytes(cls.c,view).items():
                        if i==945 and stage=='PRE':accounts[address]=cls.absent()
                        else:
                            if i==946 and stage=='POST' and address==cls.ref['targetAddresses'][3]:raw=raw[:51]+b'\1'+struct.pack('<Q',0)+bytes(24)
                            accounts[address]=cls.account(owner,raw,cls.ref['rents'][str(len(raw))])
                case={'prior_runtime':copy.deepcopy(prior),'clock_binding':cls.binding if i>=937 else None,'context_slot':str(slot),'inclusion_slot':str(cls.base+3*(i-935)+1)if stage=='POST'else None}
                cls.cases[i,stage]=(accounts,case)
                result=o.candidate_verify_state_handoff(cls.manifest,i,stage,accounts,**case)
                cls.results[i,stage]=result;prior=result['runtimeFacts']

    @staticmethod
    def account(owner,raw,lamports):return {'exists':True,'owner':owner,'executable':False,'lamports':lamports,'dataBase64':base64.b64encode(raw).decode()}
    @staticmethod
    def absent():return {'exists':False,'owner':None,'executable':None,'lamports':None,'dataBase64':None}

    def check_case(self,i,stage,mutate=None):
        accounts,kw=copy.deepcopy(self.cases[i,stage])
        if mutate:mutate(accounts,kw)
        return o.candidate_verify_state_handoff(self.manifest,i,stage,accounts,**kw)

    def test_actual_elf_four_runtime_families_and_forward_bootstrap_seal_order(self):
        for i in (935,936):
            facts=self.results[i,'POST']['runtimeFacts']
            self.assertEqual(facts['initializerBindingState'],'UNBOUND_TO_INITIALIZER')
            self.assertIsNone(facts['initializerBindingSha256'])
        self.assertEqual(self.results[936,'POST']['runtimeFacts']['runtimePrerequisiteFloor'],str(self.base+5))
        first=self.results[937,'PRE']['runtimeFacts']['initializerBindingSha256']
        self.assertTrue(all(self.results[i,s]['runtimeFacts']['initializerBindingSha256']==first for i in range(937,947)for s in ('PRE','POST')))
        final=self.results[946,'POST'];facts=final['runtimeFacts']
        self.assertEqual(facts['deploymentFact']['deploymentSlot'],str(self.base+1))
        self.assertEqual(facts['proposal'],{'timestamp':str(self.timestamp+21),'status':5,'approvedCount':4})
        self.assertEqual(facts['heartbeatPeriod'],'0');self.assertEqual(facts['lastRuntimePhase'],946)
        self.assertTrue(final['runtimePredicateVerification'])
        self.assertTrue(all(final[k]is False for k in self.r.CANDIDATE_RUNTIME_FLAGS))

    def test_scope_masks_actual_programdata_bytes_rent_authority_and_clock_owner_reject(self):
        for name,value in (('owner',self.r.SYSTEM),('lamports','4534026999'),('executable',True)):
            with self.assertRaises(ValueError):self.check_case(935,'POST',lambda a,k:a[self.r.PROGRAM_DATA].__setitem__(name,value))
        for at in (0,12,13,44,45,892396):
            def corrupt(a,k):
                raw=bytearray(base64.b64decode(a[self.r.PROGRAM_DATA]['dataBase64']));raw[at]^=1;a[self.r.PROGRAM_DATA]['dataBase64']=base64.b64encode(raw).decode()
            with self.assertRaises(ValueError):self.check_case(935,'POST',corrupt)
        with self.assertRaises(ValueError):self.check_case(935,'PRE',lambda a,k:a[self.r.CLOCK].__setitem__('owner',self.r.SYSTEM))
        with self.assertRaises(ValueError):self.check_case(935,'PRE',lambda a,k:a.__setitem__(self.r.VAULT,self.absent()))
        with self.assertRaises(ValueError):self.check_case(936,'PRE',lambda a,k:k.__setitem__('prior_runtime',{'deploymentSlot':'1'}))

    def test_early_anchor_skipped_receiver_post_replay_and_preclock_replacement_refuse(self):
        with self.assertRaises(ValueError):self.check_case(935,'PRE',lambda a,k:k.__setitem__('clock_binding',self.binding))
        with self.assertRaises(ValueError):self.check_case(937,'PRE',lambda a,k:k.__setitem__('prior_runtime',self.results[935,'POST']['runtimeFacts']))
        with self.assertRaises(ValueError):self.check_case(936,'POST',lambda a,k:k.__setitem__('prior_runtime',self.results[936,'POST']['runtimeFacts']))
        with self.assertRaises(ValueError):self.check_case(936,'POST',lambda a,k:k['prior_runtime']['pendingPreClock'].__setitem__('dataSha256','0'*64))
        with self.assertRaises(ValueError):self.check_case(936,'POST',lambda a,k:k.__setitem__('inclusion_slot',str(self.base+2)))

    def test_prior_proposal_time_and_pending_clock_cannot_widen_causal_window(self):
        def forged_proposal(a,k,timestamp):
            k['prior_runtime']['proposal']['timestamp']=str(timestamp)
            address=self.ref['roles']['proposal1'];raw=bytearray(base64.b64decode(a[address]['dataBase64']))
            raw[49:57]=struct.pack('<q',timestamp);a[address]['dataBase64']=base64.b64encode(raw).decode()
        def future(a,k):forged_proposal(a,k,self.timestamp+1000)
        with self.assertRaisesRegex(ValueError,'candidate_runtime_proposal_future'):self.check_case(941,'PRE',future)
        def before_anchor(a,k):forged_proposal(a,k,self.timestamp+3)
        with self.assertRaisesRegex(ValueError,'candidate_runtime_proposal_before_anchor'):self.check_case(941,'PRE',before_anchor)
        def pending(a,k):
            clock=k['prior_runtime']['pendingPreClock'];raw=struct.pack('<QqQQq',self.base+1,0,0,0,self.timestamp)
            clock.update(slot=str(self.base+1),contextSlot=str(self.base+1),unixTimestamp=str(self.timestamp),dataBase64=base64.b64encode(raw).decode(),dataSha256=self.r.sha(raw))
        with self.assertRaisesRegex(ValueError,'candidate_runtime_pending_clock_regression'):self.check_case(936,'POST',pending)

    def test_binding_floor_and_whole_immutable_commitment_reject(self):
        earlier=self.j.candidate_clock_binding(self.ref['preManifest'],self.ref['clock'])
        with self.assertRaisesRegex(ValueError,'candidate_runtime_binding_before_prerequisite'):
            self.check_case(937,'PRE',lambda a,k:k.__setitem__('clock_binding',earlier))
        changed=copy.deepcopy(self.binding['preManifest']);changed['initializer']['transport']['lastValidBlockHeight']=str(int(changed['initializer']['transport']['lastValidBlockHeight'])+1)
        rebound=self.j.candidate_clock_binding(changed,self.binding['clock'])
        with self.assertRaisesRegex(ValueError,'candidate_runtime_binding_changed'):
            self.check_case(938,'PRE',lambda a,k:k.__setitem__('clock_binding',rebound))

    def test_proposal_target_bytes_timestamp_and_ambiguous_heartbeat_window_refuse(self):
        proposal=self.ref['roles']['proposal1']
        def timestamp(a,k):
            raw=bytearray(base64.b64decode(a[proposal]['dataBase64']));raw[49:57]=struct.pack('<q',self.timestamp-1);a[proposal]['dataBase64']=base64.b64encode(raw).decode()
        with self.assertRaisesRegex(ValueError,'startup_proposal_timestamp_window'):self.check_case(940,'POST',timestamp)
        def target(a,k):
            address=self.ref['targetAddresses'][0];raw=bytearray(base64.b64decode(a[address]['dataBase64']));raw[-1]^=1;a[address]['dataBase64']=base64.b64encode(raw).decode()
        with self.assertRaisesRegex(ValueError,'startup_target_bytes'):self.check_case(945,'POST',target)
        def crossing(a,k):
            raw=bytearray(base64.b64decode(a[self.r.CLOCK]['dataBase64']));raw[32:40]=struct.pack('<q',int(self.binding['clockDecoded']['unixTimestamp'])+self.r.PERIOD);a[self.r.CLOCK]['dataBase64']=base64.b64encode(raw).decode()
        with self.assertRaisesRegex(ValueError,'startup_heartbeat_ambiguous_period'):self.check_case(946,'POST',crossing)
