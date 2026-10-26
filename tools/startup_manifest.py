"""Selected PIV1 Testnet startup manifest and mandatory phase predicates.

The October 21 oracle retains exact selected security policy. Five named external
Jito rules admit current protocol counters from coherent raw snapshots, retaining
whole-account digests. This is no freshness/runtime-equivalence claim. No wallet,
signer, RPC or journal lives here; the existing operator owns collection/storage.
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

# Exact old raw profiles are small, individually bound to the unchanged oracle.
# Provenance: SPL Stake Pool2.0.3 state.rs, legacy Token8.0.0 state.rs and
# solana-stake-interface StakeStateV2. Existing production identity/deposit
# semantics remain authoritative; no cargo-cache source is a runtime dependency.
JITO_SOURCE_PINS = {
    'programs/piv1/src/integrations/jito_identity.rs': '0dba585d0454af35ce9c087cfaa260b384e9d3f3396413282641d9ca087354d3',
    'programs/piv1/src/integrations/jito_deposit.rs': '3732c29d4fb493c0d91aa4827a31cf229706ddf73d18c0c49ed2a23edc0e6801',
}
JITO_PROFILE_BASE64 = {
    'stakePool': 'AdRy0PRIDCydCVtqIrCQwnYrD+1RjuY2roagNl8MFXGnngoPKlzeOLE6NGMz1mIDudWyTcyYV8Rg5LhMsIEsiilC2pZQOcb6NTzeMknM+FlLCpXxp1jOHK8qVJcDhNR2zP3f/Xm2+vjpe1HoUavXRjlVyDTPuCgcD1cX2ej2iOTZI7If9mkTuKB0C1bF1vVegscnyyQ+/Daw1pah7JXriHE4/NFB6YMsrxCtkXSVyg8nG1spPNRwJ+pzcAftQOs5oL12jyijo7L7UvNaWKfli1lcRMd47QEs9xqm+cI2cAxkJwbd9uHXZaGT2cvhRs7reawctIXtX1s3kTqM9YV+/wCpsaclJwkEAACnhi56AgMAACkEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAZAAAAAAAAAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADoAwAAAAAAAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA6AMAAAAAAAABAAAAAAAAAADgbEh4AgMAAKcHfucIBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=',
    'poolMint': 'AQAAAFTlniTAUaihSnsel5fw4Szfp0kCFmUxlaalEqbxZmArp4YuegIDAAAJAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==',
    'managerFeeAccount': '/NFB6YMsrxCtkXSVyg8nG1spPNRwJ+pzcAftQOs5oL3lhr80IOk9LXTA87UxTVFS+tSCtB8XcIhodTcrPJkngD/Y+xwRAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA',
    'reserveStake': 'AQAAAIDVIgAAAAAAVOWeJMBRqKFKex6Xl/DhLN+nSQIWZTGVpqUSpvFmYCtU5Z4kwFGooUp7HpeX8OEs36dJAhZlMZWmpRKm8WZgKwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=',
}
JITO_KINDS = {
    'stakePool': 'JITO_POOL',
    'poolMint': 'JITO_MINT',
    'managerFeeAccount': 'JITO_MANAGER_FEE',
    'reserveStake': 'JITO_RESERVE',
    'validatorList': 'JITO_VALIDATOR_LIST',
}


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


def jito_profiles():
    result={}
    for name,text in JITO_PROFILE_BASE64.items():
        raw=base64.b64decode(text,validate=True);key=oracle()['roles'][name]
        need(len(raw)==oracle()['baseline'][key]['dataBytes'] and sha(raw)==oracle()['baseline'][key]['dataSha256'],'startup_jito_original_profile')
        result[name]=raw
    return result


def jito_pool(raw):
    """Decode the complete SPL2.0.3 prefix, separating five named counters."""
    position=0
    def take(n):
        nonlocal position
        need(position+n<=len(raw),'startup_jito_pool_truncated')
        value=raw[position:position+n];position+=n;return value
    def byte():return take(1)[0]
    def u64():return struct.unpack('<Q',take(8))[0]
    def fee():
        value=struct.unpack('<QQ',take(16));need(value[1]<=value[0],'startup_jito_pool_fee');return value
    def future():
        tag=byte();need(tag in (0,1,2),'startup_jito_pool_future_fee');return (tag,None if tag==0 else fee())
    def optional():
        tag=byte();need(tag in (0,1),'startup_jito_pool_option');return (tag,None if tag==0 else take(32))
    def percentage():
        value=byte();need(value<=100,'startup_jito_pool_referral');return value
    need(byte()==1,'startup_jito_pool_type')
    security={name:take(32) for name in ('manager','staker','stakeDepositAuthority')}
    security['withdrawBump']=byte()
    security.update({name:take(32) for name in ('validatorList','reserveStake','poolMint','managerFeeAccount','tokenProgram')})
    counters={name:u64() for name in ('totalLamports','poolTokenSupply','lastUpdateEpoch')}
    security['lockup']=take(48)
    security['epochFee']=fee();security['nextEpochFee']=future()
    security['preferredDepositValidator']=optional();security['preferredWithdrawValidator']=optional()
    security['stakeDepositFee']=fee();security['stakeWithdrawalFee']=fee()
    security['nextStakeWithdrawalFee']=future();security['stakeReferralFee']=percentage()
    security['solDepositAuthority']=optional();security['solDepositFee']=fee();security['solReferralFee']=percentage()
    security['solWithdrawAuthority']=optional();security['solWithdrawalFee']=fee();security['nextSolWithdrawalFee']=future()
    counters.update({name:u64() for name in ('lastEpochPoolTokenSupply','lastEpochTotalLamports')})
    security['allocationTail']=take(len(raw)-position)
    return security,counters


def jito_mint(raw):
    need(len(raw)==82 and struct.unpack_from('<I',raw)[0]==1 and raw[44]==9 and raw[45]==1 and struct.unpack_from('<I',raw,46)[0]==0,'startup_jito_mint_layout')
    return (raw[:36],raw[44:]),struct.unpack_from('<Q',raw,36)[0]


def jito_manager_fee(raw):
    need(len(raw)==165 and raw[108]==1 and struct.unpack_from('<I',raw,72)[0] in (0,1) and struct.unpack_from('<I',raw,109)[0]==0 and struct.unpack_from('<I',raw,129)[0] in (0,1),'startup_jito_manager_fee_layout')
    return (raw[:64],raw[72:]),struct.unpack_from('<Q',raw,64)[0]


def jito_digests(c,phase,accounts,clock):
    """Five closed rules over actual complete bytes, never caller digests/masks.

    Validator entries follow the accepted identity-only boundary: external pool
    governance may change membership/order/stake/status. No withdrawal selection
    or price is authenticated here; complete raw bytes still bind coverage.
    """
    if phase['kind']=='UPLOAD_WRITE':return {}
    repo=Path(__file__).resolve().parent.parent
    for name,want in JITO_SOURCE_PINS.items():
        source=repo/name
        need(not source.is_symlink() and sha(source.read_bytes())==want,'startup_jito_semantic_source_pin')
    need(accounts is not None and clock is not None,'startup_jito_raw_snapshot_required')
    r=oracle()['roles'];baseline=oracle()['baseline'];profiles=jito_profiles();raw={};result={}
    for name in JITO_KINDS:
        key=r[name];value=accounts[key];ref=baseline[key]
        need(value['exists'] is True and value['owner']==ref['owner'] and value['executable'] is False,'startup_jito_account_identity')
        data=c.b64(value['dataBase64'],c.MAX_ACCOUNT,True)
        need(len(data)==ref['dataBytes'],'startup_jito_account_layout')
        lamports=c.u64(value['lamports'],True)
        need(name=='reserveStake' or str(lamports)==ref['lamports'],'startup_jito_account_lamports')
        raw[name]=data;result[key]=account(value['owner'],lamports,data)
    policy,counters=jito_pool(raw['stakePool']);selected,_=jito_pool(profiles['stakePool'])
    need(policy==selected,'startup_jito_pool_security_policy')
    need(all(policy[name]==c.b58decode(r[name],32) for name in ('validatorList','reserveStake','poolMint','managerFeeAccount','tokenProgram')),'startup_jito_pool_topology')
    need(policy['lockup']==bytes(48),'startup_jito_pool_lockup')
    # Clock is from the same complete getMultipleAccounts response. Startup does
    # not execute economics; a stale pool is not a fresh conversion quote.
    clock_raw=c.b64(clock['dataBase64'],40,False);need(len(clock_raw)==40,'startup_jito_clock_layout')
    epoch=struct.unpack_from('<Q',clock_raw,16)[0]
    need(counters['lastUpdateEpoch']<=epoch,'startup_jito_future_epoch')
    mint_policy,supply=jito_mint(raw['poolMint']);selected_mint,_=jito_mint(profiles['poolMint'])
    need(mint_policy==selected_mint and supply<=counters['poolTokenSupply'],'startup_jito_mint_security_or_supply')
    fee_policy,_=jito_manager_fee(raw['managerFeeAccount']);selected_fee,_=jito_manager_fee(profiles['managerFeeAccount'])
    need(fee_policy==selected_fee and raw['managerFeeAccount'][:32]==c.b58decode(r['poolMint'],32),'startup_jito_manager_fee_security')
    reserve=raw['reserveStake'];need(reserve==profiles['reserveStake'] and struct.unpack_from('<I',reserve)[0]==1,'startup_jito_reserve_security')
    rent=struct.unpack_from('<Q',reserve,4)[0]
    need(int(accounts[r['reserveStake']]['lamports'])>=rent,'startup_jito_reserve_rent')
    data=raw['validatorList'];maximum,count=struct.unpack_from('<II',data,1)
    need(data[0]==2 and maximum==10000 and (len(data)-9)//73==maximum and count<=maximum,'startup_jito_validator_list_geometry')
    return result


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
        for name,kind in JITO_KINDS.items():result[r[name]]={'jitoRule':kind}
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
    return _typed_materialize(operator,c,state,expected(operator,c,state,phase,stage),facts)


def _typed_materialize(operator,c,state,descriptors,facts):
    """Materialize this module's freshly source-derived descriptors."""
    ref=oracle();r=ref['roles'];rules=None;result={}
    for key,rule in descriptors.items():
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


def coverage(operator,c,state,phase,stage,review,facts,clock,accounts=None,full_snapshot=None,preflight_receipt=None):
    """Deterministic replay-bound coverage of actual observed complete accounts."""
    descriptors=expected(operator,c,state,phase,stage)
    literal={key:rule for key,rule in descriptors.items() if 'startupRule' not in rule and 'jitoRule' not in rule}
    literal.update(jito_digests(c,phase,accounts,clock))
    literal.update(_typed_materialize(operator,c,state,descriptors,facts))
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
      **({'preflightReceipt':copy.deepcopy(preflight_receipt)} if phase['kind']!='UPLOAD_WRITE' and stage=='PRE' else {}),
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


# Separate preparation scope. None of these APIs select the legacy oracle,
# construct an operational manifest, or authorize controller dispatch.
CANDIDATE_SCOPE = 'piv1-testnet-4ccc-in02-unselected-v2'
CANDIDATE_PHASE_SCHEMA = 'piv1-unselected-startup-phase-v2'
CANDIDATE_PROFILE_SCHEMA = 'piv1-unselected-startup-candidate-profile-v2'


def candidate_profile():
    """Return the fixed reviewed candidate identity, never caller policy."""
    return {
        'schema': CANDIDATE_PROFILE_SCHEMA, 'candidateScope': CANDIDATE_SCOPE,
        'artifact': {'bytes': 892352, 'sha256': '4ccc59ab4f37d2af98cf2dc3f2b4b1aaf701cb4061cd0315e378c0bae2b69cf2'},
        'roles': {
            'program': 'GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux',
            'programData': '6HV5GQesbZpTyBVTiMu64UU4JgCX2D2s489sKt26xpd8',
            'vault0': 'A2UHpGyiCBdUZDMrofGswNiyS4ATD1ytBSiAsWNdxUTw',
            'feePayer': 'BUuuNvicRKF4ZFEUH4iS8xGzhAKQca3ou4U1s4NaBYoT',
            'rentPayer': 'H2AaoG8HZKfNyZzMhDGMB8n9edviyHtJGKpPL3uYpxt4',
            'htfpRecipient': '33S4fefxDJ9Bz6KmHns37uvwcWSPQnsEyopAZ6YtJpdo',
            'teamRecipient': '2qJ3htw96VVBbx3TaLp8zy96AVDM9SMK3pNosrG8bNg9',
            'externalFundingSource': '4WKQg3Sm8bvHS8DBmxoiMMi4Fev4mJU6ZLwGSTg6jDna'},
        'loaderBuffer': {'address': '2hhJ8ZgoL8Xk4uXfH9uQ97wsDxfmT4qSHRGj67c3zrUo',
                         'seed': 'piv1-t-4ccc59ab4f37d2af98cf2dc3', 'dataBytes': 892389},
        'initializer': {'selector': 'PIV1IN02', 'instructionBytes': 313, 'compactBytes': 1445},
        'startupPacketCount': 947, 'separateSourceFundingPacketCount': 1,
        'historicalReviewedMessageCatalogSha256': '943847425bd801c3c99e7f6bbe3b4f7b1ff6ad874b6431e48d6409e600eac888',
        'selectedForDeployment': False, 'operationalManifest': False}


def validate_candidate_profile(profile):
    """Reject cross-candidate identity, role, domain and numeric-type changes."""
    try:
        matches = type(profile) is dict and canonical(profile) == canonical(candidate_profile())
    except (TypeError, ValueError):
        matches = False
    need(matches, 'candidate_profile_identity')


def candidate_phase(index):
    """Closed 947-phase identity/Write geometry; Source funding is separate."""
    need(type(index) is int and 0 <= index < 947, 'candidate_phase_index')
    kind = ('BUFFER_CREATE' if index == 0 else 'UPLOAD_WRITE' if index <= 934
            else 'BOOTSTRAP' if index == 935 else 'RECIPIENTS' if index == 936
            else 'INITIALIZER' if index <= 945 else 'HEARTBEAT')
    return {'schema': CANDIDATE_PHASE_SCHEMA, 'candidateScope': CANDIDATE_SCOPE,
            'id': f'candidate_{index:04}', 'index': index, 'kind': kind,
            'initializerIndex': index - 937 if 937 <= index <= 945 else None,
            'writeOffset': (index - 1) * 956 if kind == 'UPLOAD_WRITE' else None,
            'writeBytes': (404 if index == 934 else 956) if kind == 'UPLOAD_WRITE' else None}


def candidate_phase_index(phase):
    need(type(phase) is dict and 'index' in phase, 'candidate_phase_shape')
    expected = candidate_phase(phase['index'])
    try:
        matches = canonical(phase) == canonical(expected)
    except (TypeError, ValueError):
        matches = False
    need(matches, 'candidate_phase_shape')
    return phase['index']


# Complete candidate PREVIEW descriptors. This domain has no controller or live dispatch.
CANDIDATE_MANIFEST_SCHEMA = 'piv1-unselected-startup-preview-manifest-v2'
CANDIDATE_ORACLE_SHA = 'e3ca98bbc901dce6da6d1894c25a34b2649cc0d9bd19c226a11cb1ed10c3b868'
CANDIDATE_HELPER_PINS = {
    'tools/startup_public_journal.py': 'c492c1fae58caeb88f8fbe137251440626aeb051514b3f6157fefce1a0155b48',
    'tools/startup_state_rules.py': 'c85e56752aa2cb56d7e909bc61cdabcc844a34f1dc6623015d612c4dbd071395',
    'tools/collect_startup_public_evidence.py': '96309ac61a07dc23f204cd7c799b25ec67624af4d1103b9057b25728db85b7f6'}
CANDIDATE_PREVIEW_FLAGS = {'liveReady': False, 'authorized': False, 'controllerReady': False,
    'operationalManifest': False, 'runtimePredicateVerification': False, 'completeStateOracle': False,
    'freshnessQualified': False, 'fundingObserved': False, 'signerControlProved': False}


def candidate_oracle():
    path = Path(__file__).with_name('startup_manifest.candidate-oracle.json')
    need(not path.is_symlink(), 'candidate_oracle_source_pin')
    raw = path.read_bytes()
    need(len(raw) <= 65536 and sha(raw) == CANDIDATE_ORACLE_SHA, 'candidate_oracle_source_pin')
    ref = json.loads(raw)
    need(ref['schema'] == 'piv1-unselected-candidate-startup-preview-oracle-v2'
         and ref['scope'] == CANDIDATE_SCOPE and len(ref['baseline']) == 60, 'candidate_oracle_identity')
    return ref


def _candidate_helpers():
    import importlib.util
    repo = Path(__file__).resolve().parent.parent
    modules = []
    for relative, expected in CANDIDATE_HELPER_PINS.items():
        path = repo / relative
        need(not path.is_symlink(), 'candidate_helper_source_pin')
        raw = path.read_bytes()
        need(sha(raw) == expected, 'candidate_helper_source_pin')
        spec = importlib.util.spec_from_file_location('preview_'+path.stem, path)
        module = importlib.util.module_from_spec(spec)
        exec(compile(raw,str(path),'exec',dont_inherit=True),module.__dict__); modules.append(module)
    j, rules, c = modules
    need(j.candidate_pins().resolve() == repo, 'candidate_helper_repo')
    rules.verify_sources(repo)
    return j, rules, c


def _candidate_image(text):
    need(type(text) is str and len(text) == 4*((892352+2)//3), 'candidate_image_bound')
    try: image = base64.b64decode(text, validate=True)
    except Exception as exc: raise Stop('candidate_image_base64') from exc
    need(len(image) == 892352 and base64.b64encode(image).decode() == text
         and sha(image) == candidate_profile()['artifact']['sha256'], 'candidate_image_identity')
    return image


def _candidate_fees(ref):
    f = ref['historicalFeesByName']
    initial = ['initializer_buffer_create', 'initializer_buffer_extend_1', 'initializer_transaction_from_buffer',
        'initializer_proposal_create', *['initializer_approve_'+str(i) for i in range(1,5)], 'initializer_execute']
    return ([f['loader_buffer_create_and_initialize']] + [f['loader_write_0']]*933
        + [f['loader_write_891948'], f['native_atomic_bootstrap'], f['two_native_receiver_prefund']]
        + [f[name] for name in initial] + [f['paused_G0_heartbeat']])


def _candidate_review(image, index, ref):
    phase = candidate_phase(index)
    if phase['kind'] == 'INITIALIZER': return None
    if phase['kind'] != 'UPLOAD_WRITE':
        name = {0:'create', 935:'bootstrap', 936:'recipients', 946:'heartbeat'}[index]
        return copy.deepcopy(ref['staticMessages'][name])
    template = ref['writeTemplate']; chunk = image[phase['writeOffset']:phase['writeOffset']+phase['writeBytes']]
    need(len(chunk) == phase['writeBytes'], 'candidate_write_image_span')
    data = struct.pack('<IIQ',1,phase['writeOffset'],len(chunk))+chunk
    message = base64.b64decode(template['prefixBase64'],validate=True)+shortvec(len(data))+data
    return {**{k:copy.deepcopy(v) for k,v in template.items() if k != 'prefixBase64'},
        'messageBase64':base64.b64encode(message).decode(),'messageSha256':sha(message)}


def _candidate_funding(ref):
    r = ref['roles']; rent = lambda n:int(ref['rents'][str(n)])
    # Same-H2 Loader credit is before ProgramData allocation; governance refund is G0 only.
    costs = [rent(892389), rent(36), -rent(892389), rent(892397), 2*rent(0),
        rent(1557), rent(1546), rent(646), sum(rent(n) for n in ref['accountSizes']['pivTargets'])]
    used = peak = 0
    for cost in costs: used += cost; peak = max(peak,used)
    required = peak+rent(0); observed = int(ref['baseline'][r['h2']]['lamports']); gap = max(0,required-observed)
    return {'observedH2Lamports':str(observed),'requiredH2InitialLamports':str(required),
        'conditionalH2CreditLamports':str(gap),'oneH2ResidualLamports':str(rent(0)),
        'conditionalSourceDebitOutsideStartupLamports':str(gap),
        'historicalSourceFeeOutsideStartupLamports':ref['historicalFeesByName']['external_source_to_H2_conditional_deficit'],
        'observedSourceLamports':ref['baseline'][r['source']]['lamports'],
        'sourceFundingSeparateFrom947Startup':True,'fundingObserved':False,
        'wholeBudgetProved':False,'spendingAuthorityProved':False}


def candidate_assemble(image):
    """Complete947-phase preview with9 deferred initializer messages; never an admitted manifest."""
    need(type(image) is bytes and len(image) == 892352 and sha(image) == candidate_profile()['artifact']['sha256'], 'candidate_image_identity')
    _candidate_helpers(); ref = candidate_oracle(); fees = _candidate_fees(ref)
    phases = [{**candidate_phase(i), 'reviewed':_candidate_review(image,i,ref),
        'historicalFeeProjectionLamports':fees[i], 'minimumRemainingBlockHeights':'40'} for i in range(947)]
    return {'schema':CANDIDATE_MANIFEST_SCHEMA,'candidateScope':CANDIDATE_SCOPE,
        'status':'PREVIEW_CONDITIONAL_FUNDING_REQUIRED','oracleSha256':CANDIDATE_ORACLE_SHA,
        'artifact':copy.deepcopy(candidate_profile()['artifact']),'sourcePins':dict(CANDIDATE_HELPER_PINS),
        'elfBase64':base64.b64encode(image).decode(),'phases':phases,
        'initializerPreManifestSha256':sha(canonical(ref['preManifest'])),
        'historicalStartupFeeProjectionLamports':str(sum(map(int,fees))),
        'feeProjectionScope':'HISTORICAL_REPRESENTATIVE_QUOTES_WITH_932_WRITE_EXTRAPOLATIONS_NOT_FRESH_CAP',
        'fundingPrecondition':_candidate_funding(ref),
        'historicalUncertainty':{'status':'UNKNOWN','possibleFeeAvailabilityHoldLamports':'5000','resolved':False,'adoptedAsCompletedParent':False},
        'baselineContextSlot':ref['baselineContextSlot'],'requiredFullSnapshotAccountCount':60,
        'runtimeIntegration':'OPEN_DEPLOYMENT_SLOT_PROPOSAL_CLOCK_WINDOWS_AND_HEARTBEAT_PERIOD',
        **CANDIDATE_PREVIEW_FLAGS}


def candidate_validate(manifest):
    need(type(manifest) is dict and manifest.get('schema') == CANDIDATE_MANIFEST_SCHEMA
         and manifest.get('candidateScope') == CANDIDATE_SCOPE, 'candidate_manifest_scope')
    need('elfBase64' in manifest and len(canonical(manifest)) <= 8*1024*1024, 'candidate_manifest_bound')
    rebuilt = candidate_assemble(_candidate_image(manifest['elfBase64']))
    need(canonical(manifest) == canonical(rebuilt), 'candidate_manifest_mismatch')
    return rebuilt


def _candidate_balances(ref, completed, mode):
    need(type(mode) is str and mode in ('OBSERVED_ONLY','CONDITIONAL_H2_CREDIT'), 'candidate_funding_mode')
    r = ref['roles']; rent = lambda n:int(ref['rents'][str(n)])
    g = int(ref['baseline'][r['g0']]['lamports'])-sum(map(int,_candidate_fees(ref)[:completed]))
    h = int(ref['baseline'][r['h2']]['lamports'])
    if mode == 'CONDITIONAL_H2_CREDIT': h += int(_candidate_funding(ref)['conditionalH2CreditLamports'])
    if completed >= 1: h -= rent(892389)
    if completed >= 936: h += rent(892389)-rent(36)-rent(892397)
    if completed >= 937: h -= 2*rent(0)
    if completed >= 938: h -= rent(1557)
    if completed >= 940: h -= rent(1546); g += rent(1557)
    if completed >= 941: h -= rent(646)
    if completed >= 946: h -= sum(rent(n) for n in ref['accountSizes']['pivTargets'])
    need(h >= 0 and g >= 5000, 'candidate_observed_funding_insufficient')
    return g,h


def _candidate_binding(binding, j, ref):
    need(binding is not None, 'candidate_clock_binding_required')
    rebuilt = j.validate_candidate_clock_binding(binding)
    need(canonical(rebuilt['preManifest']) == canonical(ref['preManifest']), 'candidate_preview_pre_manifest')
    plan = rebuilt['unsignedInitializerPlan']
    need(plan['alt']['addresses'] == ref['targetAddresses'] and plan['accountSizes'] == ref['accountSizes'], 'candidate_preview_plan')
    return rebuilt, {'plan':plan,'postManifest':rebuilt['postManifest'],**ref['bumps']}


def _candidate_storage(c, ref, binding, completed):
    if completed < 938: return {}
    r = ref['roles']; plan = binding['plan']; compact = bytes.fromhex(plan['compact']['hex']); native = bytes.fromhex(plan['native']['hex'])
    need(len(compact) == 1445 and len(native) == 313, 'candidate_storage_payload')
    key = lambda text:c.b58decode(text,32); disc = lambda name:hashlib.sha256(('account:'+name).encode()).digest()[:8]
    if completed < 940:
        uploaded = 800 if completed == 938 else 1445
        data = disc('TransactionBuffer')+key(r['multisig'])+key(r['g0'])+bytes([1,0])
        data += hashlib.sha256(compact).digest()+struct.pack('<HI',1445,uploaded)+compact[:uploaded]
        return {r['governanceBuffer1']:data+bytes(1557-len(data))}
    count = compact[3]; at = 4+32*count; program,n = compact[at+1:at+3]; indices = compact[at+3:at+3+n]
    data = disc('VaultTransaction')+key(r['multisig'])+key(r['g0'])+struct.pack('<Q',1)+bytes([ref['squadsBumps']['transaction'],0,ref['squadsBumps']['vault']])+bytes(4)
    data += compact[:3]+struct.pack('<I',count)+compact[4:at]+struct.pack('<I',1)+bytes([program])+struct.pack('<I',n)+indices+struct.pack('<I',len(native))+native+bytes(4)
    need(len(data) == 1546, 'candidate_stored_transaction_size')
    return {r['transaction1']:data}


def _candidate_unresolved(kind, index, stage, **policy):
    return {'candidateRule':kind,'candidateScope':CANDIDATE_SCOPE,'phase':candidate_phase(index)['id'],
        'stage':stage,'runtimePredicateVerification':False,'unresolved':True,**policy}


def candidate_expected(manifest, index, stage, *, clock_binding=None, funding_mode='OBSERVED_ONLY'):
    """Source-derived PREVIEW descriptors, including explicit unresolved runtime predicates.

    No caller facts/digests, new seal, observation authenticity or controller admission.
    Only the closed conditional mode models the fixed hypothetical initial H2 credit.
    """
    candidate_validate(manifest); phase = candidate_phase(index)
    need(type(stage) is str and stage in ('PRE','POST'), 'candidate_stage')
    completed = index+(stage == 'POST'); ref = candidate_oracle(); r = ref['roles']
    j,rules,c = _candidate_helpers(); g,h = _candidate_balances(ref,completed,funding_mode)
    names = ('g0','h2','pivProgram','programData','loaderBuffer','loaderProgram','computeBudget','rent','clock','instructions')
    keys = sorted(r[name] for name in names) if phase['kind'] == 'UPLOAD_WRITE' else sorted(ref['baseline'])
    result = {key:copy.deepcopy(ref['baseline'][key]) for key in keys}
    result[r['g0']] = account(SYSTEM,g,b''); result[r['h2']] = account(SYSTEM,h,b'')
    result[r['clock']] = _candidate_unresolved('CLOCK_WINDOW',index,stage,contextFloor=ref['baselineContextSlot'],owner='Sysvar1111111111111111111111111111111111111',dataBytes=40)
    if r['source'] in result and funding_mode == 'CONDITIONAL_H2_CREDIT':
        result[r['source']] = _candidate_unresolved('SEPARATE_SOURCE_FUNDING',index,stage,
            observedBaseline=copy.deepcopy(ref['baseline'][r['source']]),
            conditionalH2CreditLamports=_candidate_funding(ref)['conditionalH2CreditLamports'],
            currentSourceFeeLamports=None, outside947Startup=True, sourceFundingNotObserved=True)
    if 1 <= completed < 936:
        image = _candidate_image(manifest['elfBase64']); written = min((completed-1)*956,892352)
        data = struct.pack('<IB',1,1)+c.b58decode(r['g0'],32)+image[:written]+bytes(892352-written)
        result[r['loaderBuffer']] = account(LOADER,ref['rents']['892389'],data)
    else: result[r['loaderBuffer']] = copy.deepcopy(ABSENT)
    if completed >= 936:
        result[r['pivProgram']] = account(LOADER,ref['rents']['36'],struct.pack('<I',2)+c.b58decode(r['programData'],32),True)
        result[r['programData']] = _candidate_unresolved('PROGRAM_DATA',index,stage,
            sourceRule=rules.candidate_program_data_rule(),owner=LOADER,dataBytes=892397,
            elfSha256=candidate_profile()['artifact']['sha256'],authority=r['vault0'],
            deploymentFactScope=CANDIDATE_SCOPE,bootstrapPostOnlyFactCreation=True)
    if phase['kind'] != 'UPLOAD_WRITE':
        if completed >= 937:
            for name in ('htfpRecipient','teamRecipient'): result[r[name]] = account(SYSTEM,ref['rents']['0'],b'')
        ms = base64.b64decode(ref['multisigBase64'],validate=True)
        if completed >= 940: ms = ms[:78]+struct.pack('<Q',1)+ms[86:]
        result[r['multisig']] = account(SQUADS,ref['baseline'][r['multisig']]['lamports'],ms)
        if completed >= 938:
            rebuilt,binding = _candidate_binding(clock_binding,j,ref)
            for address,data in _candidate_storage(c,ref,binding,completed).items(): result[address] = account(SQUADS,ref['rents'][str(len(data))],data)
            if completed >= 940: result[r['governanceBuffer1']] = copy.deepcopy(ABSENT)
            if completed >= 941:
                count = min(completed-941,4); status = 5 if completed >= 946 else 3 if count == 4 else 1
                approved = sorted(ref['preManifest']['initializer']['governance']['approvers'][:count],key=lambda text:c.b58decode(text,32))
                result[r['proposal1']] = _candidate_unresolved('PROPOSAL',index,stage,owner=SQUADS,lamports=ref['rents']['646'],dataBytes=646,approved=approved,status=status,proposalBump=ref['bumps']['proposalBump'],timestamp=None,causalClockWindowRequired=True)
            if completed >= 946:
                view = {'seal':{'stateBinding':binding,'anchorUnixTimestamp':rebuilt['clockDecoded']['unixTimestamp']}}
                targets = rules.target_bytes(c,view)
                for address,(owner,data) in targets.items(): result[address] = account(owner,ref['rents'][str(len(data))],data)
                if completed == 947:
                    address = ref['targetAddresses'][3]
                    result[address] = _candidate_unresolved('HEARTBEAT_REWARD_0',index,stage,owner=ref['roles']['pivProgram'],lamports=ref['rents']['84'],dataBytes=84,initializedBaseline=result[address],anchorUnixTimestamp=rebuilt['clockDecoded']['unixTimestamp'],period=None,unambiguousCausalClockWindowRequired=True)
    need(set(result) == set(keys), 'candidate_descriptor_coverage')
    return {'schema':'piv1-candidate-startup-preview-state-v2','candidateScope':CANDIDATE_SCOPE,
        'phase':phase['id'],'stage':stage,'requiredSnapshotKeys':keys,'accounts':result,
        'fundingMode':funding_mode,'conditionalCreditLamports':_candidate_funding(ref)['conditionalH2CreditLamports'] if funding_mode == 'CONDITIONAL_H2_CREDIT' else '0',
        **CANDIDATE_PREVIEW_FLAGS}


def candidate_runtime_handoff(manifest, index, stage, accounts, *, prior_runtime=None,
                              clock_binding=None, context_slot=None, inclusion_slot=None):
    """Verify supplied runtime components through fixed candidate policy, never preview masks."""
    candidate_validate(manifest)
    phase = candidate_phase(index); j,rules,c = _candidate_helpers()
    result = rules.verify_candidate_runtime(c,phase,stage,accounts,prior_runtime=prior_runtime,
        clock_binding=clock_binding,context_slot=context_slot,inclusion_slot=inclusion_slot)
    return {**result,'manifestSha256':sha(canonical(manifest)),
        'oracleSha256':CANDIDATE_ORACLE_SHA,'controllerReady':False}
