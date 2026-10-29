"""Dedicated D-074/D-075 continuation: existing Buffer offsets 956..892352.

D-076 is deliberately not used. Sources, runtime, authorization, complete compact
journal and current phase sidecars are checked at callback/durability boundaries.
Only old finalized phase sidecars use D-075's private operation-owned timing proof.
Reopen accepts clean finalized boundaries only; uncertainty never recreates an
original or clears HOLD from matching account bytes.
"""
import base64
import fcntl
import os
from pathlib import Path
import stat
import struct
import time
import types

from tools import startup_two_operation as base

Stop=base.Stop
need=base.need
read=base.read
sha=base.sha
encode=base.encode
parse=base.parse
b64=base.b64
START=956
END=892352
CHUNK=956
COUNT=933
FEE=5000
CAP=COUNT*FEE


def extent(phase):
    need(type(phase) is int and 2<=phase<=934,'upload_phase')
    offset=(phase-1)*CHUNK
    return offset,min(CHUNK,END-offset)


class UploadStore:
    """One locked namespace, independent journal/anchor and immutable sidecars.

    Both journals are fsynced, then cross-checked before a durable action returns.
    During a write, a private expected-byte map covers the precise transient
    boundary. No transient proof survives process loss or is accepted on reopen.
    """
    def __init__(self,path,package_sha,authorization_sha,anchor_path):
        self.path=Path(path);self.anchor_path=Path(anchor_path);need(self.path.is_absolute() and base.REPO not in self.path.parents and self.anchor_path.is_absolute() and self.anchor_path.parent==self.path.parent and self.anchor_path.parent!=self.path,'evidence_location')
        for p in [*reversed(self.path.parent.parents),self.path.parent]:
            st=p.lstat();need(stat.S_ISDIR(st.st_mode) and st.st_uid in (0,os.getuid()) and not st.st_mode&0o022,'evidence_parent')
        fresh=not self.path.exists()
        if fresh:self.path.mkdir(mode=0o700)
        self.fd=os.open(self.path,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
        fcntl.flock(self.fd,fcntl.LOCK_EX|fcntl.LOCK_NB)
        st=os.fstat(self.fd);self.directory_identity=(st.st_dev,st.st_ino)
        self.expected={};self.closed={};self.rows=[];self.journal_raw=b'';self.anchor_raw=b'';self.sequence=0;self.tail=None
        self.phase=None;self.phase_names={};self.phase_dirs={};self.last_record_name=None;self.boundary=lambda:None;self.busy=False;self.transaction_active=False;self.total_bytes=0;self._closed_prefix=None
        self.package_sha=package_sha;self.authorization_sha=authorization_sha
        self.jfd=None;self.afd=None
        if fresh:
            self.jfd=os.open('journal.jsonl',os.O_RDWR|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600,dir_fd=self.fd)
            self.afd=os.open(self.anchor_path,os.O_RDWR|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600);fcntl.flock(self.afd,fcntl.LOCK_EX|fcntl.LOCK_NB)
            os.fsync(self.jfd);os.fsync(self.afd);os.fsync(self.fd)
            parent=os.open(self.path.parent,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
            try:os.fsync(parent)
            finally:os.close(parent)
            self.append('SCOPE',{'packageSha256':package_sha,'authorizationSha256':authorization_sha,'startedUnix':int(time.time())})
        else:
            self.jfd=os.open('journal.jsonl',os.O_RDWR|os.O_APPEND|os.O_NOFOLLOW,dir_fd=self.fd)
            self.afd=os.open(self.anchor_path,os.O_RDWR|os.O_APPEND|os.O_NOFOLLOW);fcntl.flock(self.afd,fcntl.LOCK_EX|fcntl.LOCK_NB)
            try:self.reopen()
            except BaseException:self.close();raise
        fcntl.flock(self.afd,fcntl.LOCK_EX|fcntl.LOCK_NB);self.jidentity=self.file_identity(self.jfd);self.aidentity=self.file_identity(self.afd)
        self.stable()
    @staticmethod
    def file_identity(fd):
        s=os.fstat(fd);need(stat.S_ISREG(s.st_mode) and s.st_uid==os.getuid() and stat.S_IMODE(s.st_mode)==0o600 and s.st_nlink==1,'journal_file_security')
        return (s.st_dev,s.st_ino)
    def stable(self):
        st=self.path.lstat();need(stat.S_ISDIR(st.st_mode) and st.st_uid==os.getuid() and stat.S_IMODE(st.st_mode)==0o700 and (st.st_dev,st.st_ino)==self.directory_identity,'evidence_directory_changed')
        for path,fd in [(self.path/'journal.jsonl',self.jfd),(self.anchor_path,self.afd)]:
            need(fd is not None,'journal_not_open');s=path.lstat()
            need((s.st_dev,s.st_ino)==self.file_identity(fd),'journal_identity_changed')
        for phase,want in self.phase_dirs.items():
            s=(self.path/f'phase-{phase:04}').lstat();need(stat.S_ISDIR(s.st_mode) and s.st_uid==os.getuid() and stat.S_IMODE(s.st_mode)==0o700 and (s.st_dev,s.st_ino)==want,'phase_directory_changed')
    def check_boundary(self):
        if not self.busy and not self.transaction_active:
            self.busy=True
            try:self.boundary()
            finally:self.busy=False
    def sync(self,fd):
        os.fsync(fd)
    def transaction(self,action):
        if self.transaction_active:return action()
        self.check_boundary();self.transaction_active=True
        try:return action()
        finally:self.transaction_active=False;self.check_boundary()
    def capture(self,action):
        # The caller already checked every guard before the external callback.
        # Retain returned untrusted bytes even when that callback altered a pin;
        # the full post-capture guard still precedes any semantic admission.
        need(not self.transaction_active,'capture_nesting');self.transaction_active=True
        try:return action()
        finally:self.transaction_active=False;self.check_boundary()
    def append(self,kind,data):
        return self.transaction(lambda:self._append(kind,data))
    def _append(self,kind,data):
        row={'sequence':self.sequence,'previous':self.tail,'kind':kind,'data':data};raw=encode(row)
        self.check_boundary();self.journal_raw+=raw
        need(os.write(self.jfd,raw)==len(raw),'journal_short_write');self.sync(self.jfd)
        anchor=encode({'sequence':self.sequence,'digest':sha(raw),'previous':self.tail});self.anchor_raw+=anchor
        need(os.write(self.afd,anchor)==len(anchor),'anchor_short_write');self.sync(self.afd)
        self.rows.append(row);self.sequence+=1;self.tail=sha(raw);self.check_boundary()
    def begin(self,phase):
        need(self.phase is None,'phase_already_active');self.phase=phase
        p=self.path/f'phase-{phase:04}';self.check_boundary();p.mkdir(mode=0o700)
        s=p.lstat();self.phase_dirs[phase]=(s.st_dev,s.st_ino);self.phase_names[phase]=set();self.sync(self.fd)
        self.append('BEGIN',{'phase':phase,'offset':extent(phase)[0]})
    def write(self,name,raw):
        return self.transaction(lambda:self._write(name,raw))
    def _write(self,name,raw):
        need(self.phase is not None and '/' not in name and name not in self.phase_names[self.phase],'evidence_once')
        rel=f'phase-{self.phase:04}/{name}';self.check_boundary()
        fd=os.open(self.path/rel,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
        self.expected[rel]=sha(raw);self.phase_names[self.phase].add(name);self.total_bytes+=len(raw)
        try:
            pos=0
            while pos<len(raw):pos+=os.write(fd,raw[pos:])
            self.sync(fd)
        finally:os.close(fd)
        dfd=os.open(self.path/f'phase-{self.phase:04}',os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
        try:self.sync(dfd)
        finally:os.close(dfd)
        need(read(self.path/rel)==raw,'durability_uncertain')
        self.append('FILE',{'phase':self.phase,'name':name,'sha256':sha(raw),'bytes':len(raw)})
    def record(self,kind,data):
        def action():
            self.last_record_name=f'{self.sequence:06}-{kind}.json';self.write(self.last_record_name,encode({'kind':kind,'data':data}))
            self.append(kind,{'phase':self.phase,'file':self.last_record_name,'sha256':self.expected[f'phase-{self.phase:04}/{self.last_record_name}']})
            return self.tail
        return self.transaction(action)
    def current_record(self,name):
        rel=f'phase-{self.phase:04}/{name}';need(rel in self.expected and sha(read(self.path/rel))==self.expected[rel],'current_durable_record_changed')
    def audit(self,full=False):
        self.stable();need(read(self.path/'journal.jsonl')==self.journal_raw and read(self.anchor_path)==self.anchor_raw,'raw_journal_changed')
        # Parse every complete raw record at every gate, including hash links.
        # D-075: freshly compare ALL raw journal/anchor bytes above. Only parse
        # the closed immutable prefix once per full audit; no disk-validity
        # proof, caller token or persisted shortcut is accepted.
        if not full and self._closed_prefix is not None:
            joff,aoff,tail,cursor,first=self._closed_prefix
        else:joff=aoff=0;tail=None;cursor=2;first=0
        active=None;stage=None;closed_proof=self._closed_prefix
        for n,(raw,anchor) in enumerate(zip(self.journal_raw[joff:].splitlines(keepends=True),self.anchor_raw[aoff:].splitlines(keepends=True)),first):
            row=parse(raw);a=parse(anchor)
            need(raw.endswith(b'\n') and row['sequence']==n and row['previous']==tail and a=={'sequence':n,'digest':sha(raw),'previous':tail},'journal_chain')
            tail=sha(raw);kind=row['kind'];data=row['data'];joff+=len(raw);aoff+=len(anchor)
            if n==0:
                need(kind=='SCOPE' and data['packageSha256']==self.package_sha and data['authorizationSha256']==self.authorization_sha,'journal_scope');closed_proof=(joff,aoff,tail,cursor,n+1)
            elif kind=='BEGIN':need(active is None and data=={'phase':cursor,'offset':extent(cursor)[0]},'journal_phase');active=cursor;stage='BEGIN'
            elif kind=='FILE':need(active==data['phase'] and '/' not in data['name'] and data['name'] not in ('','.','..') and type(data['bytes']) is int and data['bytes']>=0,'journal_file')
            elif kind=='HALT':need(n==len(self.journal_raw.splitlines())-1,'journal_halt_terminal')
            else:
                need(kind in ('PREPARED','SIGN_ATTEMPT','SEND_INTENT','FINALIZED') and active==data['phase'],'journal_kind')
                need(stage=={'PREPARED':'BEGIN','SIGN_ATTEMPT':'PREPARED','SEND_INTENT':'SIGN_ATTEMPT','FINALIZED':'SEND_INTENT'}[kind],'journal_semantic_order');stage=kind
                if kind=='FINALIZED':active=None;cursor+=1;closed_proof=(joff,aoff,tail,cursor,n+1)
        # A private transient write can briefly precede its anchor fsync. Exact
        # expected bytes above protect it; clean audits require complete pairing.
        if not self.busy:need(len(self.journal_raw.splitlines())==len(self.anchor_raw.splitlines()),'journal_anchor_gap')
        roots={'journal.jsonl'}|{f'phase-{p:04}' for p in self.phase_dirs}
        need(set(os.listdir(self.fd))==roots,'evidence_inventory_changed')
        for phase,names in self.phase_names.items():need(set(os.listdir(self.path/f'phase-{phase:04}'))==names,'phase_inventory_changed')
        for name,digest in self.expected.items():
            phase=int(name[6:10])
            if full or phase==self.phase:
                s=(self.path/name).lstat();need(stat.S_ISREG(s.st_mode) and s.st_uid==os.getuid() and stat.S_IMODE(s.st_mode)==0o600 and s.st_nlink==1 and sha(read(self.path/name))==digest,'evidence_changed')
        if full:self._closed_prefix=closed_proof
    def finish(self,result):
        self.audit(full=True);self.record('FINALIZED',result);self.audit(full=True)
        self.closed[self.phase]=result;self.phase=None;self.check_boundary()
    def pause(self):
        need(self.phase is None,'unclean_pause');self.audit(full=True)
    def reopen(self):
        jr=read(self.path/'journal.jsonl');ar=read(self.anchor_path);need(jr.endswith(b'\n') and ar.endswith(b'\n'),'partial_journal')
        jlines=jr.splitlines(keepends=True);alines=ar.splitlines(keepends=True);need(len(jlines)==len(alines),'partial_anchor')
        tail=None;active=None;final=[]
        for n,(raw,anchor) in enumerate(zip(jlines,alines)):
            row=parse(raw);a=parse(anchor);need(row['sequence']==n and row['previous']==tail and a=={'sequence':n,'digest':sha(raw),'previous':tail},'journal_chain');tail=sha(raw)
            kind=row['kind'];data=row['data'];self.rows.append(row)
            if n==0:need(kind=='SCOPE' and set(data)=={'packageSha256','authorizationSha256','startedUnix'} and data['packageSha256']==self.package_sha and data['authorizationSha256']==self.authorization_sha and type(data['startedUnix']) is int,'reopen_scope')
            elif kind=='BEGIN':
                phase=data['phase'];need(active is None and phase==2+len(final) and data['offset']==extent(phase)[0],'reopen_phase');active=phase
                p=self.path/f'phase-{phase:04}';s=p.lstat();self.phase_dirs[phase]=(s.st_dev,s.st_ino);self.phase_names[phase]=set()
            elif kind=='FILE':
                need(active==data['phase'] and '/' not in data['name'] and data['name'] not in self.phase_names[active],'reopen_file')
                rel=f'phase-{active:04}/{data["name"]}';need(len(read(self.path/rel))==data['bytes'] and sha(read(self.path/rel))==data['sha256'],'reopen_sidecar');self.expected[rel]=data['sha256'];self.phase_names[active].add(data['name']);self.total_bytes+=data['bytes']
            else:
                need(kind in ('PREPARED','SIGN_ATTEMPT','SEND_INTENT','FINALIZED') and active==data['phase'],'reopen_uncertainty')
                rel=f'phase-{active:04}/{data["file"]}';need(self.expected.get(rel)==data['sha256'],'reopen_record_pin')
                value=parse(read(self.path/rel));need(value['kind']==kind,'reopen_record_kind')
                if kind=='FINALIZED':self.closed[active]=value['data'];final.append(active);active=None
        need(active is None and (len(self.rows)==1 or self.rows[-1]['kind']=='FINALIZED'),'pending_or_hold_reopen_forbidden')
        self.journal_raw=jr;self.anchor_raw=ar;self.sequence=len(jlines);self.tail=tail;self.audit(full=True)
    def close(self):
        for fd in (self.jfd,self.afd,self.fd):
            if fd is not None:os.close(fd)


class UploadExecution(base.Execution):
    def __init__(self,package_path,package_sha,authorization_path,authorization_sha,profile_path,evidence_path):
        self.package_path=Path(package_path);self.raw=read(self.package_path,2*1024*1024);need(sha(self.raw)==package_sha,'package_pin');self.p=parse(self.raw);self.package_sha=package_sha
        self.authorization_path=Path(authorization_path);self.authorization_sha=authorization_sha;self.profile_path=profile_path
        self.auth_raw=read(self.authorization_path,32768);self.auth=parse(self.auth_raw)
        self.start=time.monotonic();self.deadline=self.start+min(self.p['wallSeconds'],28800);self.timings=[]
        self.sent=0;self.native=0;self.native_reserved=0;self.fees=0;self.observed_fees=0;self.observed_rent=0;self.reserved=0;self.last_slot=self.p['initialContextSlot'];self.last_clock=(self.p['initialClock']['slot'],self.p['initialClock']['unixTimestamp']);self.rpc_count=0;self.current=None
        self.sources={};self.worker=None;self.index=2;self.guard_busy=False;self.guard_ready=False
        self.store=UploadStore(evidence_path,package_sha,authorization_sha,self.p['progressAnchorPath']);self.store.boundary=self.guard
        self.deadline=min(self.deadline,self.start+max(0,self.store.rows[0]['data']['startedUnix']+self.p['wallSeconds']-int(time.time())))
        need(str(self.store.path)==self.p['evidencePath'],'evidence_package_binding')
        try:self.audit_static();self.guard_ready=True;self.semantic_replay()
        except BaseException as exc:
            self.retain_hold({'status':'HOLD_NO_RETRY','phase':self.index,'reason':'initialization_'+(str(exc) if isinstance(exc,Stop) else type(exc).__name__)})
            self.store.close();raise
    def resources(self):
        need(time.monotonic()<self.deadline,'execution_deadline');s=os.statvfs(self.store.path);need(s.f_bavail*s.f_frsize>=2*1024**3,'disk_reserve')
        need(sum(len(v) for v in (self.store.journal_raw,self.store.anchor_raw))+self.store.total_bytes<=8*1024**3,'evidence_budget')
    def authorized(self):
        self.resources();now=int(time.time())
        need(self.p['notBeforeUnix']<=self.store.rows[0]['data']['startedUnix']<=now,'scope_clock')
        if hasattr(self,'phase_deadline') and self.store.phase is not None:need(time.monotonic()<self.phase_deadline,'phase_deadline')
        need(read(self.authorization_path,32768)==self.auth_raw and sha(self.auth_raw)==self.authorization_sha,'authorization_changed')
        need(self.auth['schema']=='piv1-launcher-release-authorization-v2' and self.auth['scope']=='EXACT_BOUNDED_TESTNET_RELEASE' and self.auth['manifestSha256']==self.package_sha and self.p['notBeforeUnix']<=now<min(self.p['expiresUnix'],int(self.auth['expiresUnixTimestamp'])),'authorization_expired_or_scope')
        need(self.auth['operatorSourceSha256']==self.p['sourcePins']['tools/startup_release_operator.py'] and self.auth['signerBackendSha256']==self.p['sourcePins']['tools/startup_signer_backend.cjs'] and self.auth['baseParentTail']==self.p['baseParentTail'],'authorization_source_parent')
    def source_audit(self):
        need(read(self.package_path,2*1024*1024)==self.raw,'package_changed')
        current={}
        for name,want in self.p['sourcePins'].items():
            need(not Path(name).is_absolute() and '..' not in Path(name).parts,'source_path');raw=read(base.REPO/name);need(sha(raw)==want,'source_pin');current[name]=raw
        need(sha(read(base.NODE))==base.NODE_SHA,'runtime_pin')
        image=read(self.p['artifactPath'],END);need(len(image)==END and sha(image)==base.ELF_SHA,'artifact_pin')
        for path,want in self.p['historyPins'].items():need(sha(read(path))==want,'historical_pin')
        for path,want in self.p['historyDirectories'].items():
            s=Path(path).lstat();need(stat.S_ISDIR(s.st_mode) and {'names':sorted(os.listdir(path)),'dev':s.st_dev,'ino':s.st_ino,'uid':s.st_uid,'mode':stat.S_IMODE(s.st_mode)}==want,'history_directory_changed')
        op=parse(read(base.REPO/'docs/PIV1_TESTNET_RELEASE_OPERATOR.json'));old=op['exactConditionalStartupApproval']['nextBoundedUpload']['originalRecovery']
        need(sha(base.json.dumps(old,sort_keys=True,separators=(',',':')).encode())==self.p['historicalUnknownSha256'],'historical_unknown_changed')
        anchor=parse(read(self.p['fundingAnchorPath'],32768));need(self.p['fundingAnchorPath'] in self.p['historyPins'] and anchor['path']==self.p['baseParentPath'],'funding_anchor_binding')
        if self.sources:self.transport_leaf._candidate_governed_read_parent(self.j,anchor)
        if self.sources:need(current==self.sources and image==self.image,'loaded_source_changed')
        else:
            self.sources=current;self.image=image;need(sha(current['tools/startup_manifest.candidate-oracle.json'])==base.ORACLE_SHA,'oracle_pin');self.ref=parse(current['tools/startup_manifest.candidate-oracle.json']);self.roles=self.ref['roles']
            self.c=base.module('collect_startup_public_evidence',current['tools/collect_startup_public_evidence.py']);self.transport_leaf=base.module('startup_release_operator',current['tools/startup_release_operator.py']);self.j=base.module('startup_public_journal',current['tools/startup_public_journal.py'])
    def audit_static(self):
        p=self.p;need(p['schema']=='piv1-dedicated-buffer-upload-v1' and p['cluster']=='testnet' and p['genesisHash']==base.GENESIS and p['authority']=='D-074' and p['securityPolicy']=='D-075' and p['firstPhase']==2 and p['lastPhase']==934 and p['startOffset']==START and p['endOffset']==END and p['nativeCap']==COUNT and p['sendCap']==COUNT and p['feeCapLamports']==CAP and p['perOperationFeeCapLamports']==FEE and p['rentLamports']==0 and p['historicalUnknownReserveLamports']==5000 and 0<p['wallSeconds']<=28800,'package_scope')
        required=set(base.REQUIRED_SOURCES)|{'tools/startup_buffer_upload.py','tools/startup_public_journal.py'};need(required<=set(p['sourcePins']),'incomplete_source_pins')
        self.source_audit();need(p['roles']=={k:self.roles[k] for k in base.NAMES},'role_binding')
        need(p['initialG0Lamports']>=CAP+5000 and p['initialH2Lamports']>=0 and p['baseParentPath'] in p['historyDirectories'] and p['authorityBindingPath'] in p['historyPins'],'budget_or_parent')
        need(type(p['initialContextSlot']) is int and p['initialContextSlot']>=0 and set(p['initialClock'])=={'slot','unixTimestamp'} and all(type(v) is int for v in p['initialClock'].values()) and 0<=p['initialClock']['slot']<=p['initialContextSlot'],'initial_clock_floor')
        self.authorized();self.store.audit(full=True)
    def guard(self):
        if not self.guard_ready or self.guard_busy:return
        self.guard_busy=True
        try:self.source_audit();self.authorized();self.store.audit()
        finally:self.guard_busy=False
    def fresh_age(self):
        # Pure age/read checks share the immediately preceding full admission
        # guard; no external callback or durability action intervenes.
        super().fresh_age()
    def template(self,index):
        offset,length=extent(index);data=struct.pack('<IIQ',1,offset,length)+self.image[offset:offset+length];n=len(data);compact=[]
        while True:
            compact.append((n&127)|(128 if n>>7 else 0));n>>=7
            if not n:break
        return base64.b64decode(self.ref['writeTemplate']['prefixBase64'],validate=True)+bytes(compact)+data
    def rpc(self,label,calls):
        need(label!='send' and all(method!='sendTransaction' for method,_ in calls),'readonly_rpc_only')
        need(self.rpc_count<COUNT*15 and self.rpc_count-self.phase_rpc_start<15,'rpc_budget');self.rpc_count+=1
        ordinal=self.rpc_count-self.phase_rpc_start;raw=encode([{'jsonrpc':'2.0','id':i+1,'method':method,'params':params} for i,(method,params) in enumerate(calls)]);prefix=f'rpc-{ordinal:02}-{label}'
        # The coherent persistence postguard is also the immediate TLS admission.
        self.store.write(prefix+'.request.json',raw)
        try:
            status,headers,body=self.transport_leaf.public_rpc_transport(self.c,raw);self.last_rpc_response_at=time.monotonic()
        except BaseException as exc:
            partial=getattr(exc,'partial',None)
            if partial:self.store.capture(lambda:self.store.write(prefix+'.partial.bin',partial))
            else:self.guard()
            raise
        self.store.capture(lambda:(self.store.write(prefix+'.response.json',body),self.store.write(prefix+'.http.json',encode({'status':status,'headers':headers}))))
        return self.rpc_values(status,body,len(calls))
    @staticmethod
    def rpc_values(status,body,count):
        need(status==200 and len(body)<=16*1024*1024,'rpc_status_bound');rows=parse(body);need(type(rows) is list and len(rows)==count,'rpc_batch');by={}
        for row in rows:
            need(type(row) is dict and set(row)=={'jsonrpc','id','result'} and row['jsonrpc']=='2.0' and type(row['id']) is int and 1<=row['id']<=count and row['id'] not in by,'rpc_reply');by[row['id']]=row['result']
        return [by[i+1] for i in range(count)]
    def _final_fresh_and_send(self,wire,life):
        # One fixed private admission: the final-fresh TLS response is captured,
        # fsynced and fully postguarded. ONLY trusted pure decoding/snapshot,
        # exact intent read, bounds/age/envelope construction follows before send.
        # That same fresh full guard is the send callback's admission: no caller
        # flag, proof token, intervening SDK/native/TLS/fsync/control hook exists.
        self.fresh('before_send',self.bound,life)
        need(self.sent==self.index-2 and self.native==self.index-1 and self.sent<COUNT and self.native<=COUNT,'at_most_once_budget')
        self.store.current_record(self.intent_name);need(self.rpc_count<COUNT*15 and self.rpc_count-self.phase_rpc_start<15,'rpc_budget');self.rpc_count+=1
        prefix=f'rpc-{self.rpc_count-self.phase_rpc_start:02}-send';raw=encode([{'jsonrpc':'2.0','id':1,'method':'sendTransaction','params':[wire,{'encoding':'base64','skipPreflight':False,'maxRetries':0,'preflightCommitment':'finalized','minContextSlot':self.last_slot}]}])
        self.fresh_age();self.sent+=1
        try:status,headers,body=self.transport_leaf.public_rpc_transport(self.c,raw);self.last_rpc_response_at=time.monotonic()
        except BaseException as exc:
            partial=getattr(exc,'partial',None)
            def failure():
                self.store.write(prefix+'.request.json',raw)
                if partial:self.store.write(prefix+'.partial.bin',partial)
            self.store.capture(failure);raise
        self.store.capture(lambda:(self.store.write(prefix+'.request.json',raw),self.store.write(prefix+'.response.json',body),self.store.write(prefix+'.http.json',encode({'status':status,'headers':headers}))))
        acknowledged,=self.rpc_values(status,body,1);return acknowledged
    def snapshot(self,response,completed,floor):
        context=self.c.u64(response['context']['slot']);need(context>=floor,'snapshot_context');r=self.roles;keys=sorted({base.SYSTEM}|{r[k] for k in base.NAMES});values=response['value'];need(type(values) is list and len(values)==len(keys),'snapshot_coverage')
        a={k:self.c.normalize_account(v) for k,v in zip(keys,values)}
        for name in ('pivProgram','programData'):need(not a[r[name]]['exists'],'program_already_exists')
        uploaded=START if completed==2 else min((completed-1)*CHUNK,END);fees=(completed-2)*FEE
        for name,amount in [('g0',self.p['initialG0Lamports']-fees),('h2',self.p['initialH2Lamports'])]:need(a[r[name]]=={'exists':True,'owner':base.SYSTEM,'executable':False,'lamports':str(amount),'dataBase64':''},'payer_amount_or_authority')
        need(self.p['initialG0Lamports']-fees>=(COUNT-(completed-2))*FEE+5000,'payer_sufficiency')
        data=struct.pack('<IB',1,1)+self.c.b58decode(r['g0'],32)+self.image[:uploaded]+bytes(END-uploaded)
        need(a[r['loaderBuffer']]=={'exists':True,'owner':base.LOADER,'executable':False,'lamports':str(base.RENT),'dataBase64':b64(data)},'buffer_exact_readback')
        for key in [base.SYSTEM]+[r[n] for n in ('loaderProgram','computeBudget','rent','instructions')]:
            q=a[key];raw=base64.b64decode(q['dataBase64'],validate=True) if q['exists'] else b'';digest={'exists':q['exists'],'owner':q['owner'],'executable':q['executable'],'lamports':q['lamports'],'dataBytes':len(raw),'dataSha256':sha(raw)} if q['exists'] else {'exists':False,'owner':None,'executable':None,'lamports':None,'dataBytes':None,'dataSha256':None};need(digest==self.ref['baseline'][key],'runtime_account_changed')
        clock=a[r['clock']];need(clock['exists'] and clock['owner']==base.SYSVAR and not clock['executable'] and int(clock['lamports'])>0,'clock_owner');raw=base64.b64decode(clock['dataBase64'],validate=True);need(len(raw)==40,'clock_size');slot,_,_,_,unix=struct.unpack('<QqQQq',raw)
        need(floor<=slot<=context and (self.last_clock is None or slot>=self.last_clock[0] and unix>=self.last_clock[1]),'clock_regression');self.last_clock=(slot,unix);self.last_slot=context;return a,context
    def fresh(self,label,bound,life):
        self.guard();floor=self.last_slot;config={'commitment':'finalized','minContextSlot':floor};keys=sorted({base.SYSTEM}|{self.roles[k] for k in base.NAMES})
        genesis,quote,accounts,valid,height,rent=self.rpc(label,[('getGenesisHash',[]),('getFeeForMessage',[bound['reviewed']['messageBase64'],config]),('getMultipleAccounts',[keys,{'encoding':'base64',**config}]),('isBlockhashValid',[life['recentBlockhash'],config]),('getBlockHeight',[config]),('getMinimumBalanceForRentExemption',[892389,{'commitment':'finalized'}])])
        need(genesis==base.GENESIS and rent==base.RENT,'cluster_or_rent');height=self.c.u64(height);need(valid['value'] is True and self.c.u64(valid['context']['slot'])>=floor and height>=int(life['currentBlockHeight']) and height+40<=int(life['lastValidBlockHeight']),'original_lifetime');need(self.c.u64(quote['context']['slot'])>=floor and quote['value']==FEE and self.fees+FEE<=CAP,'fee_changed')
        self.pre_accounts,_=self.snapshot(accounts,self.index,floor);self.last_slot=max(self.last_slot,quote['context']['slot'],valid['context']['slot']);self.current={'height':height,'remaining':int(life['lastValidBlockHeight'])-height,'contextFloor':self.last_slot,'responseAtMonotonic':self.last_rpc_response_at};self.timings.append({'operation':self.index,'stage':label+'_margin',**self.current,'atSeconds':time.monotonic()-self.start});self.fresh_age();return self.current
    def sign(self,packet):
        self.guard();old=self.transport_leaf.invoke_signer
        def invoke(*args,**kwargs):
            self.fresh_age();capture=kwargs['_raw_capture'];kwargs['_raw_capture']=lambda raw,complete:self.store.capture(lambda:capture(raw,complete));result=old(*args,**kwargs);self.guard();return result
        self.transport_leaf.invoke_signer=invoke
        try:return super().sign(packet)
        finally:self.transport_leaf.invoke_signer=old;self.guard()
    def sdk(self,action):
        self.guard();result=action();self.guard();return result
    def confirm(self,wire,identity):
        # Observe this original's status first. Some RPC servers omit result for
        # an unavailable getTransaction: never request that receipt speculatively
        # while status is unknown. Strict JSON admission itself is unchanged.
        signature=self.c.b58encode(bytes.fromhex(identity['signatureHex']));deadline=time.monotonic()+30;floor=self.last_slot
        for n in range(8):
            need(time.monotonic()<deadline,'confirmation_bound_hold')
            genesis,statuses=self.rpc('status',[('getGenesisHash',[]),('getSignatureStatuses',[[signature],{'searchTransactionHistory':True}])])
            need(genesis==base.GENESIS and time.monotonic()<deadline,'confirmation_bound_or_cluster')
            rows=statuses.get('value');need(type(rows) is list and len(rows)==1 and self.c.u64(statuses['context']['slot'])>=floor,'unknown_hold');status=rows[0]
            if status is not None:
                need(type(status) is dict and 'err' in status,'status_hold');slot=self.c.u64(status['slot']);need(slot>=floor and self.c.u64(statuses['context']['slot'])>=slot,'status_context')
                if status.get('confirmationStatus')=='finalized':break
                need(status.get('confirmationStatus') in ('processed','confirmed') and status['err'] is None,'pending_contradiction')
            need(n<7 and time.monotonic()+1<deadline,'unknown_hold');time.sleep(1);self.guard()
        else:raise Stop('unknown_hold')
        # One combined final receipt/readback observation, still inside the same
        # thirty-second original-only budget. Missing/malformed evidence HOLDs.
        minimum=max(slot,floor)
        genesis,tx,readback=self.rpc('receipt',[('getGenesisHash',[]),('getTransaction',[signature,{'encoding':'base64','commitment':'finalized','maxSupportedTransactionVersion':0}]),('getMultipleAccounts',[sorted({base.SYSTEM}|{self.roles[k] for k in base.NAMES}),{'encoding':'base64','commitment':'finalized','minContextSlot':minimum}])])
        need(genesis==base.GENESIS and time.monotonic()<deadline,'confirmation_bound_or_cluster')
        need(type(tx) is dict and tx.get('slot')==slot and tx.get('version')=='legacy' and tx.get('transaction')==[wire,'base64'],'transaction_identity_hold');meta=tx.get('meta');need(type(meta) is dict and 'err' in meta and meta['err']==status['err'] and meta.get('fee')==FEE,'transaction_failure_hold');self.observed_fees+=FEE
        addresses=[a['address'] for a in self.bound['reviewed']['accountRoles']];pre=[int(self.pre_accounts[k]['lamports']) if self.pre_accounts[k]['exists'] else 0 for k in addresses];post=pre.copy();post[addresses.index(self.roles['g0'])]-=FEE
        need(meta.get('preBalances')==pre and meta.get('postBalances')==post and meta.get('preTokenBalances',[])==[] and meta.get('postTokenBalances',[])==[] and meta.get('loadedAddresses',{'readonly':[],'writable':[]})=={'readonly':[],'writable':[]} and meta.get('rewards',[])==[] and meta.get('innerInstructions',[]) in (None,[]),'transaction_balance_hold');need(meta['err'] is None,'finalized_failed_transaction_hold')
        self.snapshot(readback,self.index+1,minimum);self.guard();return {'phase':self.index,'offset':extent(self.index)[0],'endOffset':sum(extent(self.index)),'signature':signature,'slot':slot,'feeLamports':FEE,'wireSha256':identity['wireSha256'],'poststateSha256':sha(encode(readback))}
    def phase_frames(self,phase):
        folder=self.store.path/f'phase-{phase:04}';names=self.store.phase_names[phase];requests=sorted(n for n in names if n.endswith('.request.json'));frames=[]
        for ordinal,name in enumerate(requests,1):
            need(name.startswith(f'rpc-{ordinal:02}-'),'replay_rpc_order');prefix=name[:-len('.request.json')];request=parse(read(folder/name));response=parse(read(folder/(prefix+'.response.json')));http=parse(read(folder/(prefix+'.http.json')));need(http['status']==200 and type(request) is list and type(response) is list and len(request)==len(response),'replay_rpc_http')
            by={}
            for row in response:
                need(type(row) is dict and set(row)=={'jsonrpc','id','result'} and row['jsonrpc']=='2.0' and type(row['id']) is int and row['id'] not in by,'replay_rpc_reply');by[row['id']]=row['result']
            for n,q in enumerate(request,1):need(set(q)=={'jsonrpc','id','method','params'} and q['jsonrpc']=='2.0' and q['id']==n and n in by,'replay_rpc_request')
            frames.append((prefix.split('-',2)[2],request,[by[n] for n in range(1,len(request)+1)]))
        need(6<=len(frames)<=13 and [f[0] for f in frames[:4]]==['original','before_sign','before_send','send'] and frames[-1][0]=='receipt' and all(f[0]=='status' for f in frames[4:-1]),'replay_rpc_lifecycle')
        allowed={n for n in names if not n.startswith('rpc-')}|{prefix+suffix for prefix in [f'rpc-{n:02}-{f[0]}' for n,f in enumerate(frames,1)] for suffix in ('.request.json','.response.json','.http.json')};need(names==allowed,'replay_raw_inventory');return frames
    def replay_fresh(self,frame,bound,life,phase):
        _,qs,vs=frame;need([q['method'] for q in qs]==['getGenesisHash','getFeeForMessage','getMultipleAccounts','isBlockhashValid','getBlockHeight','getMinimumBalanceForRentExemption'],'replay_fresh_methods');genesis,quote,accounts,valid,height,rent=vs;config=qs[1]['params'][1];floor=config['minContextSlot'];need(config=={'commitment':'finalized','minContextSlot':floor} and floor>=self.last_slot,'replay_fresh_floor')
        need(qs[0]['params']==[] and qs[1]['params'][0]==bound['reviewed']['messageBase64'] and qs[2]['params']==[sorted({base.SYSTEM}|{self.roles[k] for k in base.NAMES}),{'encoding':'base64',**config}] and qs[3]['params']==[life['recentBlockhash'],config] and qs[4]['params']==[config] and qs[5]['params']==[892389,{'commitment':'finalized'}],'replay_fresh_params')
        need(genesis==base.GENESIS and rent==base.RENT and quote['value']==FEE and quote['context']['slot']>=floor and valid['value'] is True and valid['context']['slot']>=floor and self.c.u64(height)>=int(life['currentBlockHeight']) and height+40<=int(life['lastValidBlockHeight']),'replay_fresh_values');self.pre_accounts,_=self.snapshot(accounts,phase,floor);self.last_slot=max(self.last_slot,quote['context']['slot'],valid['context']['slot'])
    def semantic_replay(self):
        """Recompute exact templates/signatures and every finalized raw lifecycle."""
        self.store.audit(full=True)
        for phase,result in self.store.closed.items():
            self.index=phase;self.guard();folder=self.store.path/f'phase-{phase:04}';rows=[r for r in self.store.rows if r['data'].get('phase')==phase and r['kind'] not in ('FILE','BEGIN')];need([r['kind'] for r in rows]==['PREPARED','SIGN_ATTEMPT','SEND_INTENT','FINALIZED'],'replay_order');values={r['kind']:parse(read(folder/r['data']['file']))['data'] for r in rows};packet=values['PREPARED'];intent=values['SEND_INTENT'];life=packet['lifetime'];frames=self.phase_frames(phase);original,qs,vs=frames[0];need([q['method'] for q in qs]==['getGenesisHash','getLatestBlockhash','getBlockHeight'] and qs[0]['params']==[],'replay_original_methods');genesis,latest,height=vs;config=qs[1]['params'][0];need(config=={'commitment':'finalized','minContextSlot':self.last_slot} and qs[1]['params']==[config] and qs[2]['params']==[config] and genesis==base.GENESIS and latest['context']['slot']>=self.last_slot,'replay_original_context');self.last_slot=latest['context']['slot'];need(life=={'recentBlockhash':latest['value']['blockhash'],'lastValidBlockHeight':str(latest['value']['lastValidBlockHeight']),'currentBlockHeight':str(self.c.u64(height)),'minimumRemainingBlockHeights':'40'},'replay_original_lifetime')
            worker=self.sdk(lambda:base.Sdk(self.sources['tools/startup_two_operation.cjs'],1,self.template(phase)))
            try:
                bound=self.sdk(lambda:worker.call({'action':'BIND','blockhash':life['recentBlockhash']}));need(packet=={'id':f'UPLOAD_WRITE_{phase-1:04}','wireBase64':bound['wireBase64'],'reviewed':bound['reviewed'],'lifetime':life,'feeQuoteLamports':str(FEE)},'replay_prepared');identity=self.sdk(lambda:worker.call({'action':'VERIFY','wireBase64':intent['wireBase64']}));need(intent=={'operation':phase,'wireBase64':intent['wireBase64'],'identity':identity,'lifetime':life,'feeReservationLamports':FEE,'sendPolicy':{'encoding':'base64','skipPreflight':False,'maxRetries':0,'preflightCommitment':'finalized','minContextSlotRule':'MAX_FINAL_FRESH_CONTEXTS'}},'replay_intent')
            finally:worker.close()
            prepared_row=rows[0];prepared_digest=sha(encode(prepared_row));binding={'id':packet['id'],'preparedRecordDigest':prepared_digest,'preparedSha256':sha(encode(packet)),'unsignedWireSha256':sha(base64.b64decode(packet['wireBase64'])),'messageSha256':packet['reviewed']['messageSha256'],'requiredSigners':packet['reviewed']['requiredSigners']};body={'schema':'piv1-startup-signing-request-v1','cluster':'testnet','genesisHash':base.GENESIS,'operatorTail':prepared_digest,'operatorSourceSha256':self.auth['operatorSourceSha256'],'authorizationSha256':self.authorization_sha,'baseParentPath':self.p['baseParentPath'],'baseParentTail':self.auth['baseParentTail'],'manifestSha256':self.package_sha,'sealSha256':None,'packets':[binding]};request={**body,'requestId':sha(encode(body))};need(values['SIGN_ATTEMPT']['request']==request,'replay_signing_request');native=parse(read(folder/f'native-{phase}.json'));need(parse(read(folder/f'native-{phase}-complete.json'))=={'complete':True} and native=={'schema':'piv1-startup-signing-response-v1','requestId':request['requestId'],'packets':[{'id':packet['id'],'wireBase64':intent['wireBase64']}]},'replay_native_response')
            self.replay_fresh(frames[1],bound,life,phase);self.replay_fresh(frames[2],bound,life,phase);signature=self.c.b58encode(bytes.fromhex(identity['signatureHex']));_,sendqs,sendvs=frames[3];need(sendqs==[{'jsonrpc':'2.0','id':1,'method':'sendTransaction','params':[intent['wireBase64'],{'encoding':'base64','skipPreflight':False,'maxRetries':0,'preflightCommitment':'finalized','minContextSlot':self.last_slot}]}] and sendvs==[signature],'replay_single_send')
            final_status=None
            for number,(_,qs,vs) in enumerate(frames[4:-1]):
                need(qs==[{'jsonrpc':'2.0','id':1,'method':'getGenesisHash','params':[]},{'jsonrpc':'2.0','id':2,'method':'getSignatureStatuses','params':[[signature],{'searchTransactionHistory':True}]}] and vs[0]==base.GENESIS,'replay_status_request');statuses=vs[1];need(type(statuses['value']) is list and len(statuses['value'])==1 and self.c.u64(statuses['context']['slot'])>=self.last_slot,'replay_status_shape');status=statuses['value'][0]
                if status is None:continue
                need(type(status) is dict and 'err' in status and self.c.u64(status['slot'])>=self.last_slot and self.c.u64(statuses['context']['slot'])>=status['slot'] and status['err'] is None,'replay_status_context')
                if status['confirmationStatus'] in ('processed','confirmed'):continue
                need(final_status is None and status['confirmationStatus']=='finalized' and number==len(frames)-6,'replay_final_status');final_status=status
            need(final_status is not None,'replay_final_status_missing');minimum=max(final_status['slot'],self.last_slot)
            _,qs,vs=frames[-1];need(qs==[{'jsonrpc':'2.0','id':1,'method':'getGenesisHash','params':[]},{'jsonrpc':'2.0','id':2,'method':'getTransaction','params':[signature,{'encoding':'base64','commitment':'finalized','maxSupportedTransactionVersion':0}]},{'jsonrpc':'2.0','id':3,'method':'getMultipleAccounts','params':[sorted({base.SYSTEM}|{self.roles[k] for k in base.NAMES}),{'encoding':'base64','commitment':'finalized','minContextSlot':minimum}]}] and vs[0]==base.GENESIS,'replay_receipt_request');tx=vs[1];poststate=vs[2]
            need(type(tx) is dict and tx['slot']==final_status['slot'],'replay_final_receipt');addresses=[a['address'] for a in bound['reviewed']['accountRoles']];pre=[int(self.pre_accounts[k]['lamports']) if self.pre_accounts[k]['exists'] else 0 for k in addresses];post=pre.copy();post[addresses.index(self.roles['g0'])]-=FEE;meta=tx['meta'];need(tx['version']=='legacy' and tx['transaction']==[intent['wireBase64'],'base64'] and meta['err']==final_status['err'] is None and meta['fee']==FEE and meta['preBalances']==pre and meta['postBalances']==post and meta.get('innerInstructions',[]) in (None,[]) and meta.get('rewards',[])==[] and meta.get('preTokenBalances',[])==[] and meta.get('postTokenBalances',[])==[] and meta.get('loadedAddresses',{'readonly':[],'writable':[]})=={'readonly':[],'writable':[]},'replay_exact_receipt')
            self.snapshot(poststate,phase+1,minimum);need(result=={'phase':phase,'offset':extent(phase)[0],'endOffset':sum(extent(phase)),'signature':signature,'slot':tx['slot'],'feeLamports':FEE,'wireSha256':identity['wireSha256'],'poststateSha256':sha(encode(poststate))},'replay_final_result');self.fees+=FEE;self.observed_fees+=FEE;self.sent+=1;self.native+=1;self.native_reserved+=1;self.reserved+=FEE;self.rpc_count+=len(frames)
        self.index=2+len(self.store.closed);self.guard()
    def retain_hold(self,hold):
        try:
            # Failure retention has no sensitive callbacks or success admission.
            # If raw custody itself is corrupt, reopening independently refuses it.
            self.store.audit();saved=self.store.boundary;self.store.boundary=lambda:None
            try:self.store.append('HALT',hold)
            finally:self.store.boundary=saved
        except Exception:pass
    def run(self):
        try:
            while self.index<=934:
                # A planned pause is allowed only at a fully audited clean cursor.
                if time.monotonic()+120>=self.deadline or int(time.time())+120>=min(self.p['expiresUnix'],int(self.auth['expiresUnixTimestamp'])):
                    self.store.pause();return {'status':'PAUSED_CLEAN','nextPhase':self.index,'nextOffset':extent(self.index)[0],'confirmedFeeLamports':self.fees}
                self.audit_static();self.store.begin(self.index);self.phase_rpc_start=self.rpc_count;self.phase_deadline=time.monotonic()+120
                self.worker=self.sdk(lambda:base.Sdk(self.sources['tools/startup_two_operation.cjs'],1,self.template(self.index)));self.guard()
                genesis,original,height=self.rpc('original',[('getGenesisHash',[]),('getLatestBlockhash',[{'commitment':'finalized','minContextSlot':self.last_slot}]),('getBlockHeight',[{'commitment':'finalized','minContextSlot':self.last_slot}])]);need(genesis==base.GENESIS and original['context']['slot']>=self.last_slot,'original_cluster_context');self.last_slot=original['context']['slot'];self.bound=self.sdk(lambda:self.worker.call({'action':'BIND','blockhash':original['value']['blockhash']}));template=self.template(self.index);offset=4+32*template[3];need(base64.b64decode(self.bound['reviewed']['messageBase64'])==template[:offset]+self.c.b58decode(original['value']['blockhash'],32)+template[offset+32:],'bound_message_mismatch')
                life={'recentBlockhash':original['value']['blockhash'],'lastValidBlockHeight':str(original['value']['lastValidBlockHeight']),'currentBlockHeight':str(self.c.u64(height)),'minimumRemainingBlockHeights':'40'};packet={'id':f'UPLOAD_WRITE_{self.index-1:04}','wireBase64':self.bound['wireBase64'],'reviewed':self.bound['reviewed'],'lifetime':life,'feeQuoteLamports':str(FEE)};self.store.record('PREPARED',packet);wire=self.sign(packet);identity=self.sdk(lambda:self.worker.call({'action':'VERIFY','wireBase64':wire}));self.reserved+=FEE
                self.store.record('SEND_INTENT',{'operation':self.index,'wireBase64':wire,'identity':identity,'lifetime':life,'feeReservationLamports':FEE,'sendPolicy':{'encoding':'base64','skipPreflight':False,'maxRetries':0,'preflightCommitment':'finalized','minContextSlotRule':'MAX_FINAL_FRESH_CONTEXTS'}});self.intent_name=self.store.last_record_name
                ack=self._final_fresh_and_send(wire,life);need(ack==self.c.b58encode(bytes.fromhex(identity['signatureHex'])),'send_acknowledgement_hold');result=self.confirm(wire,identity)
                self.audit_static();self.store.finish(result);self.fees+=FEE;self.worker.close();self.worker=None;self.index+=1
                need(self.rpc_count-self.phase_rpc_start<=15,'phase_rpc_budget')
            self.audit_static();self.store.pause();return {'status':'FULL_BUFFER_VERIFIED','nextPhase':935,'uploadedBytes':END,'artifactSha256':base.ELF_SHA,'confirmedFeeLamports':self.fees,'nativeCalls':self.native,'sendCalls':self.sent,'lastFinalized':self.store.closed[934]}
        except BaseException as exc:
            hold={'status':'HOLD_NO_RETRY','reason':str(exc) if isinstance(exc,Stop) else type(exc).__name__,'phase':self.index,'sendCalls':self.sent,'nativeCalls':self.native,'nativeReservations':self.native_reserved,'confirmedFeeLamports':self.observed_fees,'admittedFeeLamports':self.fees,'reservedFeeLamports':self.reserved,'nextAction':'SEPARATELY_SCOPED_ORIGINAL_ONLY_READ_ONLY_OBSERVATION'}
            self.retain_hold(hold)
            raise Stop(hold['reason']) from None
        finally:
            if self.worker is not None:self.worker.close()
            self.store.close()
