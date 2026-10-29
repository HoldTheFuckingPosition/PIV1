"""Exactly one Buffer-create and first-write attempt; never an upload controller.

The trusted, separately reviewed launcher supplies exact package/authorization hashes.
No restart API: even a pre-send failure consumes the fresh evidence directory. Static
pins/history are audited before EACH original and before terminal result exposure.
Only the immutable operation copy is used inside that interval (founder timing policy).
"""
import base64
import fcntl
import hashlib
import json
import os
from pathlib import Path
import resource
import selectors
import stat
import struct
import subprocess
import time
import types

REPO = Path(__file__).resolve().parent.parent
GENESIS = '4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY'
ELF_SHA = '4ccc59ab4f37d2af98cf2dc3f2b4b1aaf701cb4061cd0315e378c0bae2b69cf2'
ORACLE_SHA = 'e3ca98bbc901dce6da6d1894c25a34b2649cc0d9bd19c226a11cb1ed10c3b868'
NODE = Path('/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node')
NODE_SHA = 'bc17c508ffeed0ec622934f9b7fa72f8e78da65350e63c3eceb56fa688aa5e12'
SYSTEM = '11111111111111111111111111111111'
SYSVAR = 'Sysvar1111111111111111111111111111111111111'
LOADER = 'BPFLoaderUpgradeab1e11111111111111111111111'
RENT = 4533986360
FEES = (10000, 5000)
NAMES = ('g0','h2','pivProgram','programData','loaderBuffer','loaderProgram','computeBudget','rent','clock','instructions')
REQUIRED_SOURCES = ('tools/startup_two_operation.py','tools/startup_two_operation.cjs',
    'tools/startup_signer_backend.cjs','tools/startup_release_operator.py',
    'tools/collect_startup_public_evidence.py','clients/cli/startup-public-check.cjs',
    'tools/startup_manifest.candidate-oracle.json','tools/startup_manifest.oracle.json',
    'spikes/task-0.4-jito/package-lock.json',
    'spikes/task-0.4-jito/node_modules/@solana/web3.js/lib/index.cjs.js',
    'spikes/task-0.4-jito/node_modules/@solana/web3.js/package.json')

class Stop(ValueError): pass

def need(v, reason):
    if not v: raise Stop(reason)

def encode(v): return (json.dumps(v,sort_keys=True,separators=(',',':'))+'\n').encode()
def sha(v): return hashlib.sha256(v).hexdigest()
def b64(v): return base64.b64encode(v).decode()
def parse(raw):
    def pairs(items):
        out={}
        for k,v in items:
            need(k not in out,'duplicate_json_key');out[k]=v
        return out
    return json.loads(raw,object_pairs_hook=pairs,parse_constant=lambda _: (_ for _ in ()).throw(Stop('json_constant')))

def identity(st): return (st.st_dev,st.st_ino,st.st_mode,st.st_uid,st.st_gid,st.st_nlink,st.st_size,st.st_mtime_ns,st.st_ctime_ns)

def read(path, limit=128*1024*1024):
    path=Path(path);fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
    try:
        before=os.fstat(fd);need(stat.S_ISREG(before.st_mode) and before.st_size<=limit,'file_bound')
        chunks=[];size=0
        while True:
            part=os.read(fd,min(1024*1024,limit+1-size))
            if not part:break
            chunks.append(part);size+=len(part);need(size<=limit,'file_bound')
        need(identity(os.fstat(fd))==identity(before) and identity(os.stat(path,follow_symlinks=False))==identity(before),'file_changed')
        return b''.join(chunks)
    finally:os.close(fd)

def module(name, raw):
    m=types.ModuleType(name);m.__file__=str(REPO/'tools'/f'{name}.py');exec(compile(raw,m.__file__,'exec'),m.__dict__);return m

def no_core(): resource.setrlimit(resource.RLIMIT_CORE,(0,0))

class Store:
    """New-only evidence with exact fsync/readback. No reopening/adoption code."""
    def __init__(self,path):
        self.path=Path(path);need(self.path.is_absolute() and REPO not in self.path.parents,'evidence_location')
        for parent in reversed(self.path.parent.parents):
            st=parent.lstat();need(stat.S_ISDIR(st.st_mode) and st.st_uid in (0,os.getuid()) and not st.st_mode&0o022,'evidence_parent')
        st=self.path.parent.lstat();need(stat.S_ISDIR(st.st_mode) and st.st_uid==os.getuid() and not st.st_mode&0o022,'evidence_parent')
        self.path.mkdir(mode=0o700);self.fd=os.open(self.path,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
        fcntl.flock(self.fd,fcntl.LOCK_EX|fcntl.LOCK_NB);self.expected={};self.sequence=0;self.tail=None;self.directory_identity=(os.fstat(self.fd).st_dev,os.fstat(self.fd).st_ino)
        parent=os.open(self.path.parent,os.O_RDONLY|os.O_DIRECTORY)
        try:os.fsync(parent)
        finally:os.close(parent)
    def stable(self):
        st=os.stat(self.path,follow_symlinks=False)
        need(stat.S_ISDIR(st.st_mode) and st.st_uid==os.getuid() and stat.S_IMODE(st.st_mode)==0o700
            and (st.st_dev,st.st_ino)==self.directory_identity and (os.fstat(self.fd).st_dev,os.fstat(self.fd).st_ino)==self.directory_identity,'evidence_directory_changed')
    def write(self,name,raw):
        self.stable()
        need('/' not in name and name not in self.expected,'evidence_once')
        fd=os.open(name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600,dir_fd=self.fd)
        try:
            p=0
            while p<len(raw):p+=os.write(fd,raw[p:])
            os.fsync(fd)
        finally:os.close(fd)
        os.fsync(self.fd);self.stable();need(read(self.path/name)==raw,'durability_uncertain')
        self.expected[name]=sha(raw)
    def record(self,kind,data):
        row={'sequence':self.sequence,'previous':self.tail,'kind':kind,'data':data}
        raw=encode(row);self.last_record_name=f'{self.sequence:04}-{kind}.json';self.write(self.last_record_name,raw)
        self.sequence+=1;self.tail=sha(raw);return self.tail
    def current_record(self,name):
        self.stable();need(name in self.expected and sha(read(self.path/name,16384))==self.expected[name],'current_durable_record_changed')
    def audit(self):
        self.stable()
        need(set(os.listdir(self.fd))==set(self.expected),'evidence_inventory_changed')
        for name,digest in self.expected.items():need(sha(read(self.path/name))==digest,'evidence_changed')
    def close(self):os.close(self.fd)

class Sdk:
    def __init__(self,source,index,template):
        # Compile exact pre-audited worker bytes. Its dependencies are loaded now,
        # before ORIGINAL and audited again at terminal, never a helper reload.
        bootstrap="const Module=require('node:module');const m=new Module(process.argv[1]);m.filename=process.argv[1];m.paths=Module._nodeModulePaths(require('node:path').dirname(m.filename));m._compile(Buffer.from(process.argv[2],'base64').toString(),m.filename);m.exports.main().catch(()=>process.exitCode=1);"
        self.p=subprocess.Popen([str(NODE),'-e',bootstrap,str(REPO/'tools/startup_two_operation.cjs'),b64(source)],
            stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,cwd=REPO,
            env={'PIV1_PUBLIC_CHECK_REPO':str(REPO)},close_fds=True,preexec_fn=no_core)
        for s in (self.p.stdin,self.p.stdout,self.p.stderr):os.set_blocking(s.fileno(),False)
        try:need(self.call({'action':'PREPARE','index':index,'templateBase64':b64(template)})=={'sdk':'1.98.4','index':index},'sdk_prepare')
        except BaseException:self.close();raise
    def call(self,v):
        raw=encode(v);need(len(raw)<=16384,'sdk_input');out=bytearray();pos=0;deadline=time.monotonic()+10
        with selectors.DefaultSelector() as sel:
            sel.register(self.p.stdin,selectors.EVENT_WRITE);sel.register(self.p.stdout,selectors.EVENT_READ);sel.register(self.p.stderr,selectors.EVENT_READ)
            while b'\n' not in out:
                need(time.monotonic()<deadline and self.p.poll() is None,'sdk_timeout')
                for ev,_ in sel.select(.05):
                    if ev.fileobj is self.p.stdin:
                        pos+=os.write(self.p.stdin.fileno(),raw[pos:])
                        if pos==len(raw):sel.unregister(self.p.stdin)
                    else:
                        chunk=os.read(ev.fileobj.fileno(),16385)
                        need(ev.fileobj is not self.p.stderr and bool(chunk),'sdk_failure');out.extend(chunk);need(len(out)<=16384,'sdk_output')
        need(pos==len(raw) and out.count(b'\n')==1 and out.endswith(b'\n'),'sdk_framing')
        v=parse(out);need(set(v)=={'result'},'sdk_response');return v['result']
    def close(self):
        if self.p.poll() is None:self.p.kill()
        self.p.wait()
        for s in (self.p.stdin,self.p.stdout,self.p.stderr):s.close()

class Execution:
    def __init__(self,package_path,package_sha,authorization_path,authorization_sha,profile_path,evidence_path):
        self.package_path=Path(package_path);self.raw=read(self.package_path,2*1024*1024)
        need(sha(self.raw)==package_sha,'package_pin');self.p=parse(self.raw);self.package_sha=package_sha
        self.authorization_path=Path(authorization_path);self.authorization_sha=authorization_sha;self.profile_path=profile_path
        self.store=Store(evidence_path);self.start=time.monotonic();self.deadline=self.start+300;self.timings=[]
        self.sent=0;self.native=0;self.native_reserved=0;self.fees=0;self.observed_fees=0;self.observed_rent=0;self.reserved=0;self.last_slot=0;self.last_clock=None;self.rpc_count=0;self.current=None
        self.auth_raw=read(self.authorization_path,32768);self.auth=parse(self.auth_raw)
        self.sources={};self.worker=None;self.index=0
    def mark(self,stage,started,cpu):
        self.timings.append({'operation':self.index,'stage':stage,'startSeconds':started-self.start,'endSeconds':time.monotonic()-self.start,
            'processCpuSeconds':time.process_time()-cpu})
    def local(self,stage,fn):
        t=time.monotonic();cpu=time.process_time()
        try:return fn()
        finally:self.mark(stage,t,cpu)
    def resources(self):
        need(time.monotonic()<self.deadline,'execution_deadline')
        need(os.statvfs(self.store.path).f_bavail*os.statvfs(self.store.path).f_frsize>=2*1024**3,'disk_reserve')
    def authorized(self):
        self.resources();self.store.stable();now=int(time.time())
        need(read(self.authorization_path,32768)==self.auth_raw and sha(self.auth_raw)==self.authorization_sha,'authorization_changed')
        need(self.auth['schema']=='piv1-launcher-release-authorization-v2' and self.auth['scope']=='EXACT_BOUNDED_TESTNET_RELEASE'
            and self.auth['manifestSha256']==self.package_sha and self.p['notBeforeUnix']<=now<min(self.p['expiresUnix'],int(self.auth['expiresUnixTimestamp'])),'authorization_expired_or_scope')
        need(self.auth['operatorSourceSha256']==self.p['sourcePins']['tools/startup_release_operator.py']
            and self.auth['signerBackendSha256']==self.p['sourcePins']['tools/startup_signer_backend.cjs'],'authorization_source')
    def audit_static(self):
        need(read(self.package_path,2*1024*1024)==self.raw,'package_changed');p=self.p
        need(p['schema']=='piv1-dedicated-two-operation-v1' and p['cluster']=='testnet' and p['genesisHash']==GENESIS
            and p['operations']==['CREATE_BUFFER','WRITE_FIRST_CHUNK'] and p['authority']=='D-074'
            and p['feeCapLamports']==20000 and p['perOperationFeeCapLamports']==10000
            and p['rentLamports']==RENT and p['historicalUnknownReserveLamports']==5000,'package_scope')
        need(set(REQUIRED_SOURCES)<=set(p['sourcePins']),'incomplete_source_pins')
        sources={}
        for name,want in p['sourcePins'].items():
            need(not Path(name).is_absolute() and '..' not in Path(name).parts,'source_path')
            raw=read(REPO/name);need(sha(raw)==want,'source_pin');sources[name]=raw
        need(sha(read(NODE))==NODE_SHA,'runtime_pin')
        image=read(p['artifactPath'],892352);need(len(image)==892352 and sha(image)==ELF_SHA,'artifact_pin')
        oracle=sources['tools/startup_manifest.candidate-oracle.json'];need(sha(oracle)==ORACLE_SHA,'oracle_pin')
        ref=parse(oracle);roles=ref['roles'];need(p['roles']=={k:roles[k] for k in NAMES},'role_binding')
        need(p['historyPins'] and p['authorityBindingPath'] in p['historyPins'],'historical_authority_required')
        for path,want in p['historyPins'].items():need(sha(read(path))==want,'historical_pin')
        for path,want in p['historyDirectories'].items():
            st=Path(path).lstat();actual={'names':sorted(os.listdir(path)),'dev':st.st_dev,'ino':st.st_ino,'uid':st.st_uid,'mode':stat.S_IMODE(st.st_mode)}
            need(stat.S_ISDIR(st.st_mode) and actual==want,'history_directory_changed')
        # Exact historical UNKNOWN subtree remains separately reserved; no recovery.
        operator=parse(read(REPO/'docs/PIV1_TESTNET_RELEASE_OPERATOR.json'))
        old=operator['exactConditionalStartupApproval']['nextBoundedUpload']['originalRecovery']
        need(sha(json.dumps(old,sort_keys=True,separators=(',',':')).encode())==p['historicalUnknownSha256'],'historical_unknown_changed')
        for name in ('initialG0Lamports','initialH2Lamports'):need(type(p[name]) is int and 0<=p[name]<2**64,'payer_u64')
        need(p['initialG0Lamports']>=20000+5000 and p['initialH2Lamports']>=RENT,'budget_sufficiency')
        self.authorized();self.store.audit()
        if self.sources:need(sources==self.sources,'loaded_source_changed')
        else:
            self.sources=sources;self.ref=ref;self.roles=roles;self.image=image
            self.c=module('collect_startup_public_evidence',sources['tools/collect_startup_public_evidence.py'])
            # Only public_rpc_transport is called. No helpers/manifests/controllers.
            self.transport_leaf=module('startup_release_operator',sources['tools/startup_release_operator.py'])
        return image
    def template(self,index):
        if index==0:return base64.b64decode(self.ref['staticMessages']['create']['messageBase64'],validate=True)
        data=struct.pack('<IIQ',1,0,956)+self.image[:956]
        length=[];n=len(data)
        while True:
            length.append((n&127)|(128 if n>>7 else 0));n>>=7
            if not n:break
        return base64.b64decode(self.ref['writeTemplate']['prefixBase64'],validate=True)+bytes(length)+data
    def rpc(self,label,calls):
        self.resources();self.rpc_count+=1;need(self.rpc_count<=26,'rpc_budget')
        raw=encode([{'jsonrpc':'2.0','id':i+1,'method':m,'params':p} for i,(m,p) in enumerate(calls)])
        prefix=f'rpc-{self.rpc_count:02}-{label}'
        # SEND_INTENT already durably contains exact original wire/signature.
        # Preserve the latest fresh minContextSlot without a post-check fsync.
        # Literal send request is persisted on callback return/failure; a crash
        # before that remains the durable original's UNKNOWN/HOLD, never retry.
        if label!='send':self.local('rpc_request_persistence',lambda:self.store.write(prefix+'.request.json',raw))
        t=time.monotonic();cpu=time.process_time()
        try:
            if label=='send':self.store.current_record(self.intent_name);self.fresh_age();self.sent+=1
            status,headers,body=self.transport_leaf.public_rpc_transport(self.c,raw)
            self.last_rpc_response_at=time.monotonic()
        except Exception as exc:
            partial=getattr(exc,'partial',None)
            if partial:self.store.write(prefix+'.partial.bin',partial)
            raise
        finally:
            self.mark('rpc_wait_'+label,t,cpu)
            if label=='send':self.local('rpc_send_request_persistence',lambda:self.store.write(prefix+'.request.json',raw))
        self.local('rpc_response_persistence',lambda:(self.store.write(prefix+'.response.json',body),self.store.write(prefix+'.http.json',encode({'status':status,'headers':headers}))))
        need(status==200 and len(body)<=16*1024*1024,'rpc_status_bound');rows=parse(body)
        need(type(rows) is list and len(rows)==len(calls),'rpc_batch')
        by={}
        for row in rows:
            need(type(row) is dict and set(row)=={'jsonrpc','id','result'} and row['jsonrpc']=='2.0' and type(row['id']) is int
                and 1<=row['id']<=len(calls) and row['id'] not in by,'rpc_reply');by[row['id']]=row['result']
        return [by[i+1] for i in range(len(calls))]
    def snapshot(self,response,completed,floor):
        context=self.c.u64(response['context']['slot']);need(context>=floor,'snapshot_context')
        keys=sorted({SYSTEM}|{self.roles[k] for k in NAMES});values=response['value'];need(type(values) is list and len(values)==len(keys),'snapshot_coverage')
        accounts={k:self.c.normalize_account(v) for k,v in zip(keys,values)};r=self.roles
        for name in ('pivProgram','programData'):need(not accounts[r[name]]['exists'],'program_already_exists')
        fees=sum(FEES[:completed]);g=self.p['initialG0Lamports']-fees;h=self.p['initialH2Lamports']-(RENT if completed else 0)
        need(g>=5000+sum(FEES[completed:]) and h>=0,'payer_sufficiency')
        for name,amount in (('g0',g),('h2',h)):
            a=accounts[r[name]];need(a=={'exists':True,'owner':SYSTEM,'executable':False,'lamports':str(amount),'dataBase64':''},'payer_amount_or_authority')
        a=accounts[r['loaderBuffer']]
        if completed==0:need(not a['exists'],'buffer_exists')
        else:
            data=struct.pack('<IB',1,1)+self.c.b58decode(r['g0'],32)+(self.image[:956] if completed==2 else b'')+bytes(892352-(956 if completed==2 else 0))
            need(a=={'exists':True,'owner':LOADER,'executable':False,'lamports':str(RENT),'dataBase64':b64(data)},'buffer_exact_readback')
        for key in [SYSTEM]+[r[name] for name in ('loaderProgram','computeBudget','rent','instructions')]:
            a=accounts[key];want=self.ref['baseline'][key]
            digest={'exists':a['exists'],'owner':a['owner'],'executable':a['executable'],'lamports':a['lamports'],
                'dataBytes':len(base64.b64decode(a['dataBase64'])),'dataSha256':sha(base64.b64decode(a['dataBase64']))} if a['exists'] else {'exists':False,'owner':None,'executable':None,'lamports':None,'dataBytes':None,'dataSha256':None}
            need(digest==want,'runtime_account_changed')
        clock=accounts[r['clock']];need(clock['exists'] and clock['owner']==SYSVAR and not clock['executable'] and int(clock['lamports'])>0,'clock_owner')
        raw=base64.b64decode(clock['dataBase64'],validate=True);need(len(raw)==40,'clock_size');slot,_,_,_,unix=struct.unpack('<QqQQq',raw)
        need(floor<=slot<=context and (self.last_clock is None or slot>=self.last_clock[0] and unix>=self.last_clock[1]),'clock_regression')
        self.last_clock=(slot,unix);self.last_slot=context
        return accounts,context
    def fresh_age(self):
        need(self.current is not None and time.monotonic()-self.current['responseAtMonotonic']<=1,'fresh_sample_aged_hold')
    def fresh(self,label,bound,life):
        self.authorized();floor=self.last_slot;config={'commitment':'finalized','minContextSlot':floor};keys=sorted({SYSTEM}|{self.roles[k] for k in NAMES})
        genesis,quote,accounts,valid,height,rent=self.rpc(label,[('getGenesisHash',[]),('getFeeForMessage',[bound['reviewed']['messageBase64'],config]),
            ('getMultipleAccounts',[keys,{'encoding':'base64',**config}]),('isBlockhashValid',[life['recentBlockhash'],config]),('getBlockHeight',[config]),
            ('getMinimumBalanceForRentExemption',[892389,{'commitment':'finalized'}])])
        need(genesis==GENESIS,'wrong_cluster');need(rent==RENT,'rent_changed')
        height=self.c.u64(height);need(valid['value'] is True and self.c.u64(valid['context']['slot'])>=floor and height>=int(life['currentBlockHeight'])
            and height+40<=int(life['lastValidBlockHeight']),'original_lifetime')
        need(self.c.u64(quote['context']['slot'])>=floor and quote['value']==FEES[self.index] and self.fees+quote['value']<=self.p['feeCapLamports'],'fee_changed')
        self.pre_accounts,_=self.local('dynamic_account_checks',lambda:self.snapshot(accounts,self.index,floor));self.authorized()
        self.last_slot=max(self.last_slot,quote['context']['slot'],valid['context']['slot'])
        self.current={'height':height,'remaining':int(life['lastValidBlockHeight'])-height,'contextFloor':self.last_slot,'responseAtMonotonic':self.last_rpc_response_at}
        self.timings.append({'operation':self.index,'stage':label+'_margin',**self.current,'atSeconds':time.monotonic()-self.start})
        self.fresh_age()
        return self.current
    def sign(self,packet):
        binding={'id':packet['id'],'preparedRecordDigest':self.store.tail,'preparedSha256':sha(encode(packet)),
            'unsignedWireSha256':sha(base64.b64decode(packet['wireBase64'])),'messageSha256':packet['reviewed']['messageSha256'],'requiredSigners':packet['reviewed']['requiredSigners']}
        body={'schema':'piv1-startup-signing-request-v1','cluster':'testnet','genesisHash':GENESIS,'operatorTail':self.store.tail,
            'operatorSourceSha256':self.auth['operatorSourceSha256'],'authorizationSha256':self.authorization_sha,'baseParentPath':self.p['baseParentPath'],
            'baseParentTail':self.auth['baseParentTail'],'manifestSha256':self.package_sha,'sealSha256':None,'packets':[binding]}
        request={**body,'requestId':sha(encode(body))}
        self.local('durable_sign_attempt',lambda:self.store.record('SIGN_ATTEMPT',{'request':request}));self.native_reserved+=1;reservation_name=self.store.last_record_name
        # The unchanged backend independently verifies public bindings before private access.
        payload={'authorizationRawBase64':b64(self.auth_raw),'authorizationSha256':self.authorization_sha,'request':request,
            'packets':[packet],'signerProfilePath':self.profile_path}
        # A minimal adapter supplies only the pre-verified path and strict parser;
        # invoke_signer is a bounded subprocess leaf, never a controller callback.
        j=types.SimpleNamespace(pins=lambda:REPO,NODE=NODE,parse=parse)
        def capture(raw,complete):self.store.write(f'native-{self.index}.json',raw);self.store.write(f'native-{self.index}-complete.json',encode({'complete':complete}))
        self.fresh('before_sign',self.bound,packet['lifetime'])
        self.store.current_record(reservation_name);self.fresh_age();self.native+=1
        response=self.local('native_signing',lambda:self.transport_leaf.invoke_signer(j,payload,_raw_capture=capture))
        need(response['schema']=='piv1-startup-signing-response-v1' and response['requestId']==request['requestId'] and len(response['packets'])==1
            and response['packets'][0]['id']==packet['id'],'native_response_binding')
        return response['packets'][0]['wireBase64']
    def confirm(self,wire,identity):
        signature=self.c.b58encode(bytes.fromhex(identity['signatureHex']));deadline=time.monotonic()+30;floor=self.last_slot
        for n in range(8):
            self.authorized();need(time.monotonic()<deadline,'confirmation_bound_hold')
            genesis,statuses,tx=self.rpc('confirm',[('getGenesisHash',[]),('getSignatureStatuses',[[signature],{'searchTransactionHistory':True}]),
                ('getTransaction',[signature,{'encoding':'base64','commitment':'finalized','maxSupportedTransactionVersion':0}])])
            need(time.monotonic()<deadline and genesis==GENESIS,'confirmation_bound_or_cluster')
            rows=statuses.get('value');need(type(rows) is list and len(rows)==1 and type(rows[0]) is dict,'unknown_hold')
            status=rows[0];slot=self.c.u64(status['slot']);need(slot>=floor and statuses['context']['slot']>=slot and 'err' in status,'status_hold')
            if tx is None and status.get('confirmationStatus') in ('processed','confirmed') and status['err'] is None:
                need(n<7 and time.monotonic()+1<deadline,'confirmation_bound_hold');time.sleep(1);continue
            need(status.get('confirmationStatus')=='finalized' and type(tx) is dict and tx.get('slot')==slot and tx.get('version')=='legacy'
                and tx.get('transaction')==[wire,'base64'],'transaction_identity_hold')
            meta=tx.get('meta');need(type(meta) is dict and 'err' in meta and meta['err']==status['err'] and meta.get('fee')==FEES[self.index],'transaction_failure_hold')
            self.observed_fees+=FEES[self.index]
            # Exact per-message account balances; no token/CPI/loaded-address effects.
            addresses=[a['address'] for a in self.bound['reviewed']['accountRoles']]
            before=self.pre_accounts
            expected_pre=[int(before[k]['lamports']) if before[k]['exists'] else 0 for k in addresses]
            expected_post=expected_pre.copy();expected_post[addresses.index(self.roles['g0'])]-=FEES[self.index]
            if self.index==0 and meta['err'] is None:expected_post[addresses.index(self.roles['h2'])]-=RENT;expected_post[addresses.index(self.roles['loaderBuffer'])]+=RENT
            need(meta.get('preBalances')==expected_pre and meta.get('postBalances')==expected_post
                and meta.get('preTokenBalances',[])==[] and meta.get('postTokenBalances',[])==[]
                and meta.get('loadedAddresses',{'readonly':[],'writable':[]})=={'readonly':[],'writable':[]},'transaction_balance_hold')
            need(meta['err'] is None,'finalized_failed_transaction_hold')
            if self.index==0:self.observed_rent=RENT
            post,=self.rpc('readback',[('getMultipleAccounts',[sorted({SYSTEM}|{self.roles[k] for k in NAMES}),{'encoding':'base64','commitment':'finalized','minContextSlot':slot}])])
            self.local('exact_poststate',lambda:self.snapshot(post,self.index+1,max(slot,floor)))
            self.authorized();return {'signature':signature,'slot':slot,'feeLamports':FEES[self.index],'wireSha256':identity['wireSha256'],'poststateSha256':sha(encode(post))}
        raise Stop('confirmation_bound_hold')
    def run(self):
        results=[]
        try:
            self.store.record('CONSUMED_SCOPE',{'packageSha256':self.package_sha,'authorizationSha256':self.authorization_sha})
            for index in (0,1):
                self.index=index;self.local('static_audit_before_original',self.audit_static)
                template=self.template(index);self.worker=self.local('sdk_prepare_before_original',lambda:Sdk(self.sources['tools/startup_two_operation.cjs'],index,template))
                self.authorized()
                genesis,original,initial_height=self.rpc('original',[('getGenesisHash',[]),('getLatestBlockhash',[{'commitment':'finalized','minContextSlot':self.last_slot}]),('getBlockHeight',[{'commitment':'finalized','minContextSlot':self.last_slot}])])
                need(genesis==GENESIS and original['context']['slot']>=self.last_slot,'original_cluster_context');self.last_slot=original['context']['slot']
                self.bound=self.local('message_bind',lambda:self.worker.call({'action':'BIND','blockhash':original['value']['blockhash']}))
                # Independent byte reconstruction: only the 32-byte legacy blockhash changes.
                count=template[3];offset=4+32*count;expected=template[:offset]+self.c.b58decode(original['value']['blockhash'],32)+template[offset+32:]
                need(base64.b64decode(self.bound['reviewed']['messageBase64'])==expected,'bound_message_mismatch')
                life={'recentBlockhash':original['value']['blockhash'],'lastValidBlockHeight':str(original['value']['lastValidBlockHeight']),'currentBlockHeight':str(self.c.u64(initial_height)),'minimumRemainingBlockHeights':'40'}
                packet={'id':('UPLOAD_CREATE_BUFFER','UPLOAD_WRITE_0000')[index],'wireBase64':self.bound['wireBase64'],'reviewed':self.bound['reviewed'],
                    'lifetime':life,'feeQuoteLamports':str(FEES[index])}
                self.store.record('PREPARED',packet)
                wire=self.sign(packet)
                identity=self.local('exact_signature_verification',lambda:self.worker.call({'action':'VERIFY','wireBase64':wire}))
                self.authorized();self.reserved+=FEES[index]
                self.local('durable_send_intent',lambda:self.store.record('SEND_INTENT',{'operation':index,'wireBase64':wire,'identity':identity,'lifetime':life,'feeReservationLamports':FEES[index],
                    'sendPolicy':{'encoding':'base64','skipPreflight':False,'maxRetries':0,'preflightCommitment':'finalized','minContextSlotRule':'MAX_FINAL_FRESH_CONTEXTS'}}))
                self.intent_name=self.store.last_record_name
                # No submission until durable intent succeeds; refresh dynamics after fsync.
                self.fresh('before_send',self.bound,life);self.authorized();need(self.sent==index,'one_send')
                acknowledged,=self.rpc('send',[('sendTransaction',[wire,{'encoding':'base64','skipPreflight':False,'maxRetries':0,
                    'preflightCommitment':'finalized','minContextSlot':self.last_slot}])])
                need(acknowledged==self.c.b58encode(bytes.fromhex(identity['signatureHex'])),'send_acknowledgement_hold')
                result=self.confirm(wire,identity)
                self.local('static_history_audit_terminal',self.audit_static)
                self.store.record('FINALIZED',result);self.store.audit();results.append(result);self.fees+=FEES[index]
                self.worker.close();self.worker=None
            result={'status':'FINALIZED_EXACT_TWO_OPERATIONS','results':results,'sendCalls':self.sent,'nativeCalls':self.native,'feeLamports':self.fees,'rentLamports':RENT,'timings':self.timings}
            self.store.write('result.json',encode(result));return result
        except BaseException as exc:
            result={'status':'HOLD_NO_RETRY','reason':str(exc) if isinstance(exc,Stop) else type(exc).__name__,'operation':self.index,
                'sendCalls':self.sent,'nativeCalls':self.native,'nativeReservations':self.native_reserved,'confirmedFeeLamports':self.observed_fees,'confirmedRentLamports':self.observed_rent,'admittedFeeLamports':self.fees,'reservedFeeLamports':self.reserved,
                'completedOperations':results,'timings':self.timings}
            try:self.store.write('hold.json',encode(result))
            except Exception:pass
            raise Stop(result['reason']) from None
        finally:
            if self.worker is not None:self.worker.close()
            self.store.close()
