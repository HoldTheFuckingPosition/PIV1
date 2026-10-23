"""Selected PIV1 Testnet startup manifest and mandatory phase predicates.

The October 21 oracle is a conservative retained profile, never a freshness or
runtime-equivalence claim. Changes stop. No wallet, signer, RPC or journal lives
here. The existing operator collects actual raw accounts and owns persistence.
"""
import base64
import copy
import hashlib
import json
from functools import lru_cache
from pathlib import Path
import struct

SCHEMA = 'piv1-selected-startup-release-manifest-v1'
PROFILE_SCHEMA = 'piv1-selected-startup-profile-v1'
ORACLE_SHA = 'a1c17f54f8a5ccd953e8e86c792a784c7ce91884570c7f169618f462f43327e6'
COUNT = 949
ELF_SHA = 'e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99'
ELF_BYTES = 894552
SYSTEM = '11111111111111111111111111111111'
LOADER = 'BPFLoaderUpgradeab1e11111111111111111111111'
SQUADS = 'SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf'
NATIVE = 'NativeLoader1111111111111111111111111111111'
ABSENT = dict(exists=False,owner=None,executable=None,lamports=None,dataBytes=None,dataSha256=None)


class Stop(ValueError): pass


def need(ok, reason):
    if not ok: raise Stop(reason)


def sha(raw): return hashlib.sha256(raw).hexdigest()


def canonical(value):
    return (json.dumps(value,sort_keys=True,separators=(',', ':'),ensure_ascii=True,allow_nan=False)+'\n').encode()


def full(manifest): return manifest.get('schema') == SCHEMA


@lru_cache(maxsize=1)
def oracle():
    path=Path(__file__).with_name('startup_manifest.oracle.json')
    need(not path.is_symlink(),'startup_oracle_source_pin')
    raw=path.read_bytes(); need(sha(raw)==ORACLE_SHA,'startup_oracle_source_pin')
    value=json.loads(raw)
    need(value['schema']=='piv1-selected-startup-source-oracle-v1' and len(value['baseline'])==57,'startup_oracle_shape')
    return value


@lru_cache(maxsize=1)
def image_bytes(text,expected_sha):
    need(type(text) is str and len(text)==4*((ELF_BYTES+2)//3),'startup_elf_bound')
    try: raw=base64.b64decode(text,validate=True)
    except Exception as exc: raise Stop('startup_elf_base64') from exc
    need(len(raw)==ELF_BYTES and base64.b64encode(raw).decode()==text and sha(raw)==expected_sha,'startup_selected_elf')
    return raw


def selected_image(profile):
    need(type(profile) is dict and set(profile)=={'schema','oracleSha256','elfBase64'},'startup_profile_fields')
    need(profile['schema']==PROFILE_SCHEMA and profile['oracleSha256']==ORACLE_SHA,'startup_profile_identity')
    return image_bytes(profile['elfBase64'],ELF_SHA)


def phase_index(phase):
    name=phase['id']; need(type(name) is str and len(name)==12 and name.startswith('startup_'),'startup_phase_id')
    try: index=int(name[8:])
    except ValueError as exc: raise Stop('startup_phase_id') from exc
    need(0<=index<COUNT and name==f'startup_{index:04}','startup_phase_id')
    return index


def phase_kind(index):
    return 'BUFFER_CREATE' if index==0 else 'UPLOAD_WRITE' if index<=936 else 'BOOTSTRAP' if index==937 else 'RECIPIENTS' if index==938 else 'INITIALIZER' if index<=947 else 'HEARTBEAT'


def shortvec(value):
    result=bytearray()
    while True:
        part=value&127; value>>=7; result.append(part|(128 if value else 0))
        if not value:return bytes(result)


def write_review(image, index):
    need(1<=index<=936,'startup_write_index'); ref=oracle()['writeTemplate']
    offset=(index-1)*956; chunk=image[offset:offset+956]
    need(len(chunk)==(692 if index==936 else 956),'startup_write_chunk')
    data=struct.pack('<IIQ',1,offset,len(chunk))+chunk
    message=base64.b64decode(ref['prefixBase64'])+shortvec(len(data))+data
    return {**{k:copy.deepcopy(v) for k,v in ref.items() if k!='prefixBase64'},'messageBase64':base64.b64encode(message).decode(),'messageSha256':sha(message)}


def reviewed(image,index):
    if 939<=index<=947:return None
    if 1<=index<=936:return write_review(image,index)
    name={0:'create',937:'bootstrap',938:'recipients',948:'heartbeat'}[index]
    return copy.deepcopy(oracle()['staticMessages'][name])


def assemble(operator,j,image,*,package_sha256,parent_tail,historical_ledger,minimum_remaining_block_heights='40'):
    """Build complete descriptors from selected source facts, with nine deferred wires.

    The ELF is public and stored once. No account maps or949 generated wires are
    committed as fixtures. Actual blockhashes/signatures remain operator inputs.
    """
    need(type(image) is bytes and len(image)==ELF_BYTES and sha(image)==ELF_SHA,'startup_selected_elf')
    j.digest(package_sha256); need(j.u64(minimum_remaining_block_heights)>0,'startup_lifetime_margin')
    historical=copy.deepcopy(historical_ledger)
    pins={'readback':operator.READBACK_SHA,'checker':j.CHECKER_SHA,'sdk':j.SDK_SHA,
          'journal':'bfea079a5740fa66e437930d50793a18289beeae1760af4164e986295c8a6d37',
          'stateRules':operator.STATE_RULES_SHA,'startupManifest':operator.MANIFEST_SHA,'startupOracle':ORACLE_SHA}
    phases=[]
    for index in range(COUNT):
        phases.append({'id':f'startup_{index:04}','kind':phase_kind(index),'initializerIndex':index-939 if 939<=index<=947 else None,
          'reviewed':reviewed(image,index),'feeCapLamports':oracle()['fees'][index],
          'minimumRemainingBlockHeights':minimum_remaining_block_heights,'expectedPrestate':{},'expectedPoststate':{}})
    return {'schema':SCHEMA,'packageSha256':package_sha256,'artifact':copy.deepcopy(operator.ARTIFACT),'sourcePins':pins,
      'initializerPreManifestSha256':sha(canonical(oracle()['preManifest'])),'parentTail':parent_tail,'historicalLedger':historical,
      'proposedFeeBudget':{'namespace':'piv1-testnet-startup-release-'+package_sha256,'status':'PROPOSAL_ONLY_NOT_AUTHORIZED','feeCapLamports':'4815000','historicalLedgerReference':{'parentTail':parent_tail,'ledger':copy.deepcopy(historical)}},
      'startupProfile':{'schema':PROFILE_SCHEMA,'oracleSha256':ORACLE_SHA,'elfBase64':base64.b64encode(image).decode()},'phases':phases}


def validate(operator,j,manifest):
    """Mandatory on every full-mode reopen, before any handoff/import/transport."""
    need(full(manifest),'startup_full_manifest_required')
    need(set(manifest)=={'schema','packageSha256','artifact','sourcePins','initializerPreManifestSha256','parentTail','historicalLedger','proposedFeeBudget','startupProfile','phases'},'startup_manifest_fields')
    need(manifest['artifact']==operator.ARTIFACT,'startup_selected_artifact')
    j.digest(manifest['packageSha256']); j.digest(manifest['initializerPreManifestSha256'])
    budget=manifest['proposedFeeBudget'];ledger=manifest['historicalLedger']
    need(set(ledger)=={'cumulativeFeeCapLamports','reservedFeeLamports','actualFeeLamports'},'startup_historical_ledger_fields')
    for value in ledger.values():j.u64(value)
    need(budget=={'namespace':'piv1-testnet-startup-release-'+manifest['packageSha256'],'status':'PROPOSAL_ONLY_NOT_AUTHORIZED','feeCapLamports':'4815000','historicalLedgerReference':{'parentTail':manifest['parentTail'],'ledger':ledger}},'startup_budget_binding')
    expected_pins={'readback':operator.READBACK_SHA,'checker':j.CHECKER_SHA,'sdk':j.SDK_SHA,'journal':'bfea079a5740fa66e437930d50793a18289beeae1760af4164e986295c8a6d37','stateRules':operator.STATE_RULES_SHA,'startupManifest':operator.MANIFEST_SHA,'startupOracle':ORACLE_SHA}
    need(manifest['sourcePins']==expected_pins,'startup_manifest_source_pins')
    image=selected_image(manifest['startupProfile']); ref=oracle()
    need(len(manifest['phases'])==COUNT,'startup_exact_949_required')
    need(manifest['initializerPreManifestSha256']==sha(canonical(ref['preManifest'])),'startup_initializer_profile')
    need(manifest['sourcePins'].get('startupManifest')==operator.MANIFEST_SHA and manifest['sourcePins'].get('startupOracle')==ORACLE_SHA and manifest['sourcePins'].get('stateRules')==operator.STATE_RULES_SHA,'startup_manifest_source_pins')
    for index,phase in enumerate(manifest['phases']):
        need(set(phase)=={'id','kind','initializerIndex','reviewed','feeCapLamports','minimumRemainingBlockHeights','expectedPrestate','expectedPoststate'},'startup_phase_fields')
        need(j.u64(phase['minimumRemainingBlockHeights'])>0,'startup_lifetime_margin')
        need(phase_index(phase)==index and phase['kind']==phase_kind(index),'startup_exact_phase_order')
        need(phase['initializerIndex']==(index-939 if 939<=index<=947 else None),'startup_initializer_index')
        need(phase['expectedPrestate']=={} and phase['expectedPoststate']=={},'startup_generated_states_only')
        need(phase['feeCapLamports']==ref['fees'][index],'startup_phase_fee_profile')
        target=reviewed(image,index)
        need(phase['reviewed'] is None if target is None else operator.semantic(phase['reviewed'])==operator.semantic(target),'startup_source_message_semantics')
    need(sum(map(int,ref['fees']))==4815000,'startup_source_fee_total')
    # One immutable CREATE record must fit the inherited16MiB record input bound.
    need(len(canonical(manifest))<j.MAX_INPUT-65536,'startup_manifest_record_bound')


def required_keys(phase):
    ref=oracle()
    if phase['kind']!='UPLOAD_WRITE':return sorted(ref['baseline'])
    # P2 relies on the loader/Buffer and current actor debits. The57-account
    # profile is checked before create and again at the atomic bootstrap boundary.
    names=('g0','h2','pivProgram','programData','loaderBuffer','loaderProgram','computeBudget','rent','clock','instructions')
    return sorted(ref['roles'][name] for name in names)


def large_key():return oracle()['roles']['stakePoolProgramData']


def normalize_account(c,key,value):
    # One selected complete ProgramData image exceeds the unchanged1MiB helper.
    if key!=large_key() or value is None:return c.normalize_account(value)
    ref=oracle()['baseline'][key]
    need(type(value) is dict and value.get('owner')==ref['owner'] and value.get('executable') is False,'startup_large_account_identity')
    data=value.get('data');need(type(data) is list and len(data)==2 and data[1]=='base64','startup_large_account_encoding')
    raw=c.b64(data[0],1106125,True)
    need(len(raw)==1106125 and sha(raw)==ref['dataSha256'],'startup_large_account_complete_image')
    need(str(c.u64(value.get('lamports')))==ref['lamports'],'startup_large_account_lamports')
    return {'exists':True,'owner':value['owner'],'executable':False,'lamports':ref['lamports'],'dataBase64':data[0]}


def digest(operator,c,key,value):
    if key!=large_key():return operator.state_digest(c,value)
    need(value['exists'] is True,'startup_large_account_missing')
    raw=c.b64(value['dataBase64'],1106125,True)
    need(len(raw)==1106125,'startup_large_account_complete_image')
    result=account(value['owner'],value['lamports'],raw,value['executable'])
    need(result==oracle()['baseline'][key],'startup_large_account_complete_image')
    return result


def proof_keys(phase):return [key for key in required_keys(phase) if key!=large_key()]


def account(owner,lamports,data,executable=False):
    return dict(exists=True,owner=owner,executable=executable,lamports=str(lamports),dataBytes=len(data),dataSha256=sha(data))


def complete_count(phase,stage):return phase_index(phase)+(stage=='POST')


def actor_balances(completed):
    ref=oracle();r=ref['roles'];rent=ref['rents']
    g=int(ref['baseline'][r['g0']]['lamports'])-sum(map(int,ref['fees'][:completed]))
    h=int(ref['baseline'][r['h2']]['lamports'])
    if completed>=1:h-=int(rent['894589'])
    if completed>=938:h+=int(rent['894589'])-int(rent['36'])-int(rent['894597'])
    if completed>=939:h-=2*int(rent['0'])
    if completed>=940:h-=int(rent['1559'])
    if completed>=942:h-=int(rent['1548']);g+=int(rent['1559'])
    if completed>=943:h-=int(rent['646'])
    if completed>=948:h-=sum(int(rent[str(size)]) for size in ref['targetSizes'])
    need(min(g,h)>=0,'startup_actor_insufficient')
    return g,h


def storage_bytes(c,state,completed):
    """Exact selected Squads Borsh storage; zero tails match pinned allocation."""
    ref=oracle();r=ref['roles'];key=lambda value:c.b58decode(value,32)
    if completed<940:return {}
    seal=state.get('seal');need(seal is not None,'startup_storage_seal_required')
    plan=seal['stateBinding']['plan'];compact=bytes.fromhex(plan['compact']['hex']);native=bytes.fromhex(plan['native']['hex'])
    need(len(compact)==1447 and len(native)==315,'startup_storage_lengths')
    disc=lambda name:hashlib.sha256(('account:'+name).encode()).digest()[:8]
    if completed<942:
        uploaded=800 if completed==940 else 1447
        raw=disc('TransactionBuffer')+key(r['multisig'])+key(r['g0'])+bytes([1,0])+hashlib.sha256(compact).digest()+struct.pack('<HI',1447,uploaded)+compact[:uploaded]
        return {r['governanceBuffer1']:raw+bytes(1559-len(raw))}
    count=compact[3];keys=compact[4:4+32*count];at=4+32*count
    need(count==34 and compact[at]==1,'startup_compact_shape')
    program=compact[at+1];roles_count=compact[at+2];indices=compact[at+3:at+3+roles_count]
    need(roles_count==34,'startup_compact_roles')
    raw=disc('VaultTransaction')+key(r['multisig'])+key(r['g0'])+struct.pack('<Q',1)+bytes([ref['transactionBump'],0,ref['vaultBump']])+bytes(4)
    raw+=compact[:3]+struct.pack('<I',count)+keys+struct.pack('<I',1)+bytes([program])+struct.pack('<I',roles_count)+indices+struct.pack('<I',len(native))+native+bytes(4)
    need(len(raw)==1548,'startup_transaction_size')
    return {r['transaction1']:raw}


@lru_cache(maxsize=937)
def loader_buffer_account(completed,image,authority,rent):
    written=min(max(completed-1,0)*956,ELF_BYTES)
    data=struct.pack('<IB',1,1)+authority+image[:written]+bytes(ELF_BYTES-written)
    return account(LOADER,rent,data)


def expected(operator,c,state,phase,stage):
    """Derive complete active-phase predicates; caller hashes cannot select them."""
    ref=oracle();r=ref['roles'];rent=ref['rents'];completed=complete_count(phase,stage)
    result={key:copy.deepcopy(ref['baseline'][key]) for key in required_keys(phase) if key!=r['clock']}
    g,h=actor_balances(completed)
    result[r['g0']]=account(SYSTEM,g,b'');result[r['h2']]=account(SYSTEM,h,b'')
    if 1<=completed<938:
        image=selected_image(state['manifest']['startupProfile'])
        result[r['loaderBuffer']]=loader_buffer_account(completed,image,c.b58decode(r['g0'],32),rent['894589'])
    else:result[r['loaderBuffer']]=copy.deepcopy(ABSENT)
    if completed>=938:
        result[r['pivProgram']]=account(LOADER,rent['36'],struct.pack('<I',2)+c.b58decode(r['programData'],32),True)
        result[r['programData']]={'startupRule':'PROGRAM_DATA','lamports':rent['894597']}
    if phase['kind']!='UPLOAD_WRITE':
        if completed>=939:
            for name in ('htfpRecipient','teamRecipient'):result[r[name]]=account(SYSTEM,rent['0'],b'')
        ms=base64.b64decode(ref['multisigBase64'])
        if completed>=942:ms=ms[:78]+struct.pack('<Q',1)+ms[86:]
        result[r['multisig']]=account(SQUADS,ref['baseline'][r['multisig']]['lamports'],ms)
        for key,data in storage_bytes(c,state,completed).items():result[key]=account(SQUADS,rent[str(len(data))],data)
        if completed>=942:result[r['governanceBuffer1']]=copy.deepcopy(ABSENT)
        if completed>=943:result[r['proposal1']]={'startupRule':'PROPOSAL','lamports':rent['646']}
        if completed>=948:
            for index,key in enumerate(ref['targetAddresses']):
                kind='HEARTBEAT_REWARD_0' if index==3 and phase['kind']=='HEARTBEAT' else 'INITIALIZED_TARGET'
                result[key]={'startupRule':kind,'lamports':rent[str(ref['targetSizes'][index])]}
    return result


def effective(operator,c,state,phase):
    if not full(state['manifest']):return phase
    return {**phase,'expectedPrestate':expected(operator,c,state,phase,'PRE'),'expectedPoststate':expected(operator,c,state,phase,'POST'),'completeStartup':True}


def exact_typed(operator,c,state,phase,stage,facts):
    """Materialize compact replay digests from the existing four typed facts.

    SEND/RECOVER still call the unchanged rules against actual complete bytes.
    This stores no extra nullable security oracle or arbitrary field mask.
    """
    ref=oracle();r=ref['roles'];rules=None;result={}
    for key,rule in expected(operator,c,state,phase,stage).items():
        kind=rule.get('startupRule')
        if kind is None:continue
        if rules is None:rules=operator.state_rules()
        if kind=='PROGRAM_DATA':
            need('deploymentSlot' in facts,'startup_deployment_fact_missing')
            data=struct.pack('<IQB',3,int(facts['deploymentSlot']),1)+c.b58decode(r['vault0'],32)+selected_image(state['manifest']['startupProfile'])
            owner=LOADER
        elif kind in ('INITIALIZED_TARGET','HEARTBEAT_REWARD_0'):
            owner,data=rules.target_bytes(c,state)[key]
            if kind=='HEARTBEAT_REWARD_0' and 'heartbeatPeriod' in facts:data=data[:51]+b'\1'+struct.pack('<Q',int(facts['heartbeatPeriod']))+bytes(24)
        else:
            need(kind=='PROPOSAL' and 'proposal' in facts,'startup_proposal_fact_missing');value=facts['proposal'];count=value['approvedCount']
            approved=sorted(ref['preManifest']['initializer']['governance']['approvers'][:count],key=lambda x:c.b58decode(x,32))
            data=hashlib.sha256(b'account:Proposal').digest()[:8]+c.b58decode(r['multisig'],32)+struct.pack('<QBqB',1,value['status'],int(value['timestamp']),ref['proposalBump'])
            data+=struct.pack('<I',count)+b''.join(c.b58decode(x,32) for x in approved)+bytes(8);data+=bytes(646-len(data));owner=SQUADS
        result[key]=account(owner,rule['lamports'],data)
    return result


def message_instructions(review,c):
    """Read the already pinned decoder's resolved message, for virtual sysvar data."""
    raw=base64.b64decode(review['messageBase64'],validate=True);position=0
    def take(n):
        nonlocal position
        need(n>=0 and position+n<=len(raw),'startup_virtual_message_truncated')
        value=raw[position:position+n];position+=n;return value
    def number():
        out=0;before=position
        for index in range(3):
            byte=take(1)[0];out|=(byte&127)<<(7*index)
            if not byte&128:
                need(raw[before:position]==shortvec(out),'startup_virtual_shortvec');return out
        raise Stop('startup_virtual_shortvec')
    version=raw[0]==128
    if version:take(1)
    take(3);take(number()*32);take(32);instructions=[]
    roles=review['accountRoles']
    for _ in range(number()):
        program=take(1)[0];indices=take(number());data=take(number())
        need(program<len(roles) and all(index<len(roles) for index in indices),'startup_virtual_role_index')
        body=struct.pack('<H',len(indices))
        for index in indices:
            role=roles[index];body+=bytes([int(role['signer'])|2*int(role['writable'])])+c.b58decode(role['address'],32)
        body+=c.b58decode(roles[program]['address'],32)+struct.pack('<H',len(data))+data;instructions.append(body)
    if version:
        for _ in range(number()):take(32);take(number());take(number())
    need(position==len(raw) and 1<=len(instructions)<=16,'startup_virtual_message_tail')
    at=2+2*len(instructions);offsets=[]
    for body in instructions:offsets.append(at);at+=len(body)
    need(at<65536,'startup_virtual_instruction_bound')
    return struct.pack('<H',len(instructions))+b''.join(struct.pack('<H',n) for n in offsets)+b''.join(instructions)+struct.pack('<H',len(instructions)-1)


def coverage(operator,c,state,phase,stage,review,facts,clock,accounts=None,full_snapshot=None):
    """Deterministic replay-bound coverage of actual observed complete accounts."""
    descriptors=expected(operator,c,state,phase,stage)
    literal={key:rule for key,rule in descriptors.items() if 'startupRule' not in rule}
    literal.update(exact_typed(operator,c,state,phase,stage,facts))
    need(clock is not None,'startup_coverage_clock_required');operator.state_rules().saved_clock(clock)
    clock_key=oracle()['roles']['clock'];clock_digest=copy.deepcopy(oracle()['baseline'][clock_key])
    clock_digest['dataSha256']=clock['dataSha256'];literal[clock_key]=clock_digest
    need(set(literal)==set(required_keys(phase)),'startup_coverage_set')
    if accounts is not None:
        need(set(accounts)==set(required_keys(phase)),'startup_snapshot_exact_coverage')
        need({key:digest(operator,c,key,accounts[key]) for key in literal}==literal,'startup_mandatory_account_predicate')
    wire=state['signed'].get(phase['id'])
    referenced=oracle()['roles']['instructions'] in {role['address'] for role in review['accountRoles']}
    return {'schema':'piv1-startup-mandatory-coverage-v1','oracleSha256':ORACLE_SHA,'manifestSha256':state['manifestSha256'],'phase':phase['id'],'stage':stage,
      'parentTail':state['manifest']['parentTail'],'sealDataSha256':state['seal']['sealDataSha256'] if state['seal'] else None,
      'messageSha256':review['messageSha256'],'wireSha256':wire['wireSha256'] if wire else None,
      'requiredKeys':required_keys(phase),'accountDigests':literal,'runtimeFacts':copy.deepcopy(facts),'clock':copy.deepcopy(clock),
      'fullSnapshot':copy.deepcopy(full_snapshot),
      'virtualInstructionsSha256':sha(message_instructions(review,c)) if referenced else None,
      'virtualInstructionsContract':'RPC_ABSENT_RUNTIME_SYNTHESIZED_FROM_EXACT_COMPILED_OUTER_MESSAGE' if referenced else 'NOT_REFERENCED_NOT_INSTANTIATED_RPC_ABSENT'}


def validate_seal(operator,j,state,data):
    ref=oracle();post=copy.deepcopy(ref['preManifest']);post['initializer']['initialization']['anchorUnixTimestamp']=data['anchorUnixTimestamp']
    need(data['stateBinding'] is not None and data['stateBinding']['postManifest']==post,'startup_selected_seal_profile')
    derived=j._node_request({'preManifest':ref['preManifest'],'anchor':data['anchorUnixTimestamp']},planner=True)
    need(derived['postManifest']==post and derived['plan']==data['stateBinding']['plan'],'startup_selected_seal_plan')
    plan=derived['plan'];need(plan['transaction']==ref['roles']['transaction1'] and plan['buffer']==ref['roles']['governanceBuffer1'] and plan['proposal']==ref['roles']['proposal1'],'startup_selected_seal_accounts')
    packets=[]
    for packet in plan['packets']:
        raw=base64.b64decode(packet['wireBase64']);offset=1+raw[0]*64
        tables=[{'address':plan['alt']['address'],'addresses':plan['alt']['addresses']}] if raw[offset]==128 else []
        packets.append({'wireBase64':packet['wireBase64'],'lookupTables':tables})
    description=operator.describe(j,packets,state_plan=plan,governance=post['initializer']['governance'])
    reviews=description.pop('packets')
    need(all(data['stateBinding'][name]==description[name] for name in ('targetBumps','authorityBump','proposalBump')),'startup_sealed_derived_bumps')
    need(len(data['reviewedPackets'])==9,'startup_sealed_nine_messages')
    need(len(reviews)==9 and all(operator.semantic(a)==operator.semantic(b) for a,b in zip(reviews,data['reviewedPackets'])),'startup_sealed_nine_messages')
    need(data['stateBinding']['proposalBump']==ref['proposalBump'],'startup_selected_proposal_bump')


def rent_sizes(phase):
    return [894589] if phase['kind']=='UPLOAD_WRITE' else sorted(map(int,oracle()['rents']))
