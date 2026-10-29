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

# D-077 is deliberately a distinct exact scope. The original v1 executor and
# consumed journals retain their historical clean-only disposition unchanged.
RETIRE_FIRST=72
RETIRE_START=67876
SEND_CAP=863
NATIVE_CAP=933
RETIRE_CAP=70
SEND_FEE_CAP=4315000
RESERVATION_CAP=4665000


class RetiringStore(UploadStore):
    """Physical attempt directories; logical cursor advances only on FINALIZED.

    Retired/unsent sidecars are ALWAYS freshly read, including after a later
    original for that logical offset succeeds. The canonical raw prefix is still
    compared in full on every gate; only semantic parsing is operation-private.
    """
    first_phase=RETIRE_FIRST
    native_cap=NATIVE_CAP
    def __init__(self,*args):
        self.attempts={};self.finalized={};self.retired={};self.bridge=None;self.cursor=self.first_phase;self.pending=None
        super().__init__(*args)
    def begin(self,logical,bridge=False):
        need(self.phase is None and logical==self.cursor,'attempt_cursor')
        attempt=0 if bridge else 1+len([a for a in self.attempts if a])
        need((bridge and not self.attempts) or (not bridge and self.bridge is not None and 1<=attempt<=self.native_cap),'attempt_bound')
        self.phase=attempt;meta={'phase':attempt,'logicalPhase':logical,'offset':extent(logical)[0],'startedUnix':int(time.time()),'attemptKind':'PRIOR_BRIDGE' if bridge else 'WRITE'}
        p=self.path/f'phase-{attempt:04}';self.check_boundary();p.mkdir(mode=0o700);s=p.lstat();self.phase_dirs[attempt]=(s.st_dev,s.st_ino);self.phase_names[attempt]=set();self.attempts[attempt]=dict(meta);self.sync(self.fd);self.append('BEGIN',meta)
    def audit(self,full=False):
        self.stable();need(read(self.path/'journal.jsonl')==self.journal_raw and read(self.anchor_path)==self.anchor_raw,'raw_journal_changed')
        if not full and self._closed_prefix is not None:joff,aoff,tail,cursor,next_attempt,bridge,first=self._closed_prefix
        else:joff=aoff=0;tail=None;cursor=self.first_phase;next_attempt=1;bridge=False;first=0
        active=None;stage=None;proof=self._closed_prefix
        for n,(raw,anchor) in enumerate(zip(self.journal_raw[joff:].splitlines(keepends=True),self.anchor_raw[aoff:].splitlines(keepends=True)),first):
            row=parse(raw);a=parse(anchor);need(raw.endswith(b'\n') and row['sequence']==n and row['previous']==tail and a=={'sequence':n,'digest':sha(raw),'previous':tail},'journal_chain');tail=sha(raw);joff+=len(raw);aoff+=len(anchor);kind=row['kind'];data=row['data']
            if n==0:need(kind=='SCOPE' and data['packageSha256']==self.package_sha and data['authorizationSha256']==self.authorization_sha,'journal_scope');proof=(joff,aoff,tail,cursor,next_attempt,bridge,n+1)
            elif kind=='BEGIN':
                expected=next_attempt if bridge else 0;need(active is None and data['phase']==expected and data['logicalPhase']==cursor and data['offset']==extent(cursor)[0] and type(data['startedUnix']) is int and data['attemptKind']==('WRITE' if bridge else 'PRIOR_BRIDGE'),'journal_attempt');active=expected;stage='BEGIN'
            elif kind=='FILE':need(active==data['phase'] and '/' not in data['name'] and data['name'] not in ('','.','..') and type(data['bytes']) is int and data['bytes']>=0,'journal_file')
            elif kind=='HALT':need(n==len(self.journal_raw.splitlines())-1,'journal_halt_terminal')
            else:
                need(active==data['phase'] and kind in ('PREPARED','SIGN_ATTEMPT','SEND_INTENT','UNSENT_LIFETIME_REFUSAL','RETIRED_UNSENT','FINALIZED','PRIOR_BRIDGE'),'journal_kind')
                want={'PREPARED':'BEGIN','SIGN_ATTEMPT':'PREPARED','SEND_INTENT':'SIGN_ATTEMPT','UNSENT_LIFETIME_REFUSAL':'SEND_INTENT','RETIRED_UNSENT':'UNSENT_LIFETIME_REFUSAL','FINALIZED':'SEND_INTENT','PRIOR_BRIDGE':'BEGIN'}[kind];need(stage==want,'journal_semantic_order');stage=kind
                if kind in ('FINALIZED','RETIRED_UNSENT','PRIOR_BRIDGE'):
                    if kind=='PRIOR_BRIDGE':need(active==0 and not bridge,'bridge_once');bridge=True
                    else:
                        need(active==next_attempt,'attempt_sequence');next_attempt+=1
                        if kind=='FINALIZED':cursor+=1
                    active=None;proof=(joff,aoff,tail,cursor,next_attempt,bridge,n+1)
        need(len(self.journal_raw.splitlines())==len(self.anchor_raw.splitlines()),'journal_anchor_gap')
        need(set(os.listdir(self.fd))=={'journal.jsonl'}|{f'phase-{a:04}' for a in self.phase_dirs},'evidence_inventory_changed')
        for attempt,names in self.phase_names.items():need(set(os.listdir(self.path/f'phase-{attempt:04}'))==names,'phase_inventory_changed')
        for name,digest in self.expected.items():
            attempt=int(name[6:10])
            if full or attempt==self.phase or attempt not in self.finalized:
                s=(self.path/name).lstat();need(stat.S_ISREG(s.st_mode) and s.st_uid==os.getuid() and stat.S_IMODE(s.st_mode)==0o600 and s.st_nlink==1 and sha(read(self.path/name))==digest,'evidence_changed')
        if full:self._closed_prefix=proof
    def disposition(self,kind,data):
        self.audit(full=True);self.record(kind,data);self.audit(full=True);attempt=self.phase
        if kind=='FINALIZED':self.finalized[attempt]=data;self.closed[attempt]=data;self.cursor+=1
        elif kind=='RETIRED_UNSENT':self.retired[attempt]=data
        else:need(kind=='PRIOR_BRIDGE' and attempt==0,'disposition_kind');self.bridge=data
        self.attempts[attempt]['disposition']=kind;self.phase=None;self.pending=None;self.check_boundary()
    def reopen(self):
        jr=read(self.path/'journal.jsonl');ar=read(self.anchor_path);need(jr.endswith(b'\n') and ar.endswith(b'\n'),'partial_journal');jl=jr.splitlines(keepends=True);al=ar.splitlines(keepends=True);need(len(jl)==len(al),'partial_anchor');tail=None;active=None;stage=None
        for n,(raw,anchor) in enumerate(zip(jl,al)):
            row=parse(raw);a=parse(anchor);need(row['sequence']==n and row['previous']==tail and a=={'sequence':n,'digest':sha(raw),'previous':tail},'journal_chain');tail=sha(raw);kind=row['kind'];d=row['data'];self.rows.append(row)
            if n==0:need(kind=='SCOPE' and d['packageSha256']==self.package_sha and d['authorizationSha256']==self.authorization_sha and set(d)=={'packageSha256','authorizationSha256','startedUnix'},'reopen_scope')
            elif kind=='BEGIN':
                expected=0 if self.bridge is None else 1+len([a for a in self.attempts if a]);need(active is None and d['phase']==expected and d['logicalPhase']==self.cursor and d['offset']==extent(self.cursor)[0],'reopen_attempt');active=expected;stage='BEGIN';self.attempts[active]=dict(d);s=(self.path/f'phase-{active:04}').lstat();self.phase_dirs[active]=(s.st_dev,s.st_ino);self.phase_names[active]=set()
            elif kind=='FILE':
                need(active==d['phase'] and '/' not in d['name'] and d['name'] not in self.phase_names[active],'reopen_file');rel=f'phase-{active:04}/{d["name"]}';rawfile=read(self.path/rel);need(len(rawfile)==d['bytes'] and sha(rawfile)==d['sha256'],'reopen_sidecar');self.expected[rel]=d['sha256'];self.phase_names[active].add(d['name']);self.total_bytes+=d['bytes']
            else:
                need(kind in ('PREPARED','SIGN_ATTEMPT','SEND_INTENT','UNSENT_LIFETIME_REFUSAL','RETIRED_UNSENT','FINALIZED','PRIOR_BRIDGE') and active==d['phase'],'reopen_uncertainty');rel=f'phase-{active:04}/{d["file"]}';need(self.expected.get(rel)==d['sha256'],'reopen_record_pin');v=parse(read(self.path/rel));need(v['kind']==kind,'reopen_record_kind');stage=kind
                if kind in ('FINALIZED','RETIRED_UNSENT','PRIOR_BRIDGE'):
                    value=v['data'];self.attempts[active]['disposition']=kind
                    if kind=='FINALIZED':self.finalized[active]=value;self.closed[active]=value;self.cursor+=1
                    elif kind=='RETIRED_UNSENT':self.retired[active]=value
                    else:need(active==0,'bridge_identity');self.bridge=value
                    active=None
        need(active is None or stage=='UNSENT_LIFETIME_REFUSAL','pending_or_hold_reopen_forbidden');self.phase=active;self.pending=active
        self.journal_raw=jr;self.anchor_raw=ar;self.sequence=len(jl);self.tail=tail;self.audit(full=True)


class PriorUploadHistory:
    """Read-only raw lineage input; never opens an old Store or old authority."""
    def __init__(self,path,want):
        self.path=Path(path);self.raw=read(self.path,8*1024*1024);need(sha(self.raw)==want,'prior_manifest_pin');self.m=parse(self.raw);need(self.m['schema']=='piv1-prior-upload-handoff-v1','prior_schema');self.checked=False
        self.audit(True);self.derive()
    def audit(self,full=False):
        m=self.m;need(read(self.path,8*1024*1024)==self.raw,'prior_manifest_changed')
        for path,d in m['directories'].items():
            s=Path(path).lstat();need(stat.S_ISDIR(s.st_mode) and {'dev':s.st_dev,'ino':s.st_ino,'uid':s.st_uid,'mode':stat.S_IMODE(s.st_mode),'names':sorted(os.listdir(path))}==d,'prior_namespace_changed')
        for path,d in m['freshFiles'].items():need(len(read(path))==d['bytes'] and sha(read(path))==d['sha256'],'prior_fresh_file_changed')
        if full:
            for path,d in m['closedFiles'].items():need(2<=d['phase']<=71 and len(read(path))==d['bytes'] and sha(read(path))==d['sha256'],'prior_closed_file_changed')
        need(sha(read(m['journalPath']))==m['journalSha256'] and sha(read(m['anchorPath']))==m['anchorSha256'],'prior_raw_chain_changed')
        package=parse(read(m['packagePath']));need(set(m.get('historyMap',{}))=={str(base.REPO/'docs/PIV1_DECISIONS.md')},'prior_history_map_scope')
        for oldpath,want in package['historyPins'].items():
            mapped=m.get('historyMap',{}).get(oldpath);actual=mapped['frozenPath'] if mapped else oldpath
            need((mapped is None or mapped['sha256']==want) and sha(read(actual))==want,'prior_history_provenance')
        need(package['schema']=='piv1-dedicated-buffer-upload-v1' and package['startOffset']==956,'prior_package')
        for name,d in m['sourceMap'].items():
            expected=self.path.parent/('baseline-'+Path(name).name) if name in ('tools/startup_buffer_upload.py','tools/test_startup_buffer_upload.py') else base.REPO/name
            need(Path(d['frozenPath'])==expected and package['sourcePins'][name]==d['sha256'] and sha(read(d['frozenPath']))==d['sha256'],'prior_source_provenance')
        need(set(m['sourceMap'])==set(package['sourcePins']),'prior_complete_source_map')
    def derive(self):
        m=self.m;raws=read(m['journalPath']).splitlines(keepends=True);anchors=read(m['anchorPath']).splitlines(keepends=True);need(len(raws)==len(anchors),'prior_anchor_count');tail=None;phase=None;records={};files={}
        for n,(raw,anchor) in enumerate(zip(raws,anchors)):
            row=parse(raw);a=parse(anchor);need(row['sequence']==n and row['previous']==tail and a=={'sequence':n,'digest':sha(raw),'previous':tail},'prior_chain');tail=sha(raw);kind=row['kind'];d=row['data']
            if n==0:need(kind=='SCOPE' and row['data']['packageSha256']==sha(read(m['packagePath'])) and row['data']['authorizationSha256']==sha(read(m['authorizationPath'])),'prior_scope')
            elif kind=='BEGIN':phase=d['phase'];need(2<=phase<=72,'prior_phase');records[phase]=[];files[phase]={}
            elif kind=='FILE':
                need(d['phase']==phase,'prior_file_phase');path=str(Path(str(Path(m['journalPath']).parent))/f'phase-{phase:04}'/d['name']);expected=(m['closedFiles'] if phase<=71 else m['freshFiles']);need(path in expected and expected[path]['sha256']==d['sha256'] and expected[path]['bytes']==d['bytes'],'prior_file_commitment');files[phase][d['name']]=path
            elif kind=='HALT':need(phase==72 and n==len(raws)-1 and d['reason']=='original_lifetime' and d['sendCalls']==70 and d['nativeCalls']==71 and d['nativeReservations']==71 and d['admittedFeeLamports']==350000 and d['reservedFeeLamports']==355000,'prior_normal_refusal');self.halt=d
            else:
                need(d['phase']==phase and kind in ('PREPARED','SIGN_ATTEMPT','SEND_INTENT','FINALIZED'),'prior_record_kind');value=parse(read(files[phase][d['file']]));need(value['kind']==kind and sha(read(files[phase][d['file']]))==d['sha256'],'prior_record_pin');records[phase].append((kind,value['data']))
        need(set(records)==set(range(2,73)) and tail==m['priorNoSend']['haltDigest'],'prior_complete_cursor');verified=set(m['verifiedWireSha256']);actual=set()
        for phase,rows in records.items():
            need([k for k,_ in rows]==(['PREPARED','SIGN_ATTEMPT','SEND_INTENT','FINALIZED'] if phase<=71 else ['PREPARED','SIGN_ATTEMPT','SEND_INTENT']),'prior_semantic_order');v=dict(rows);packet=v['PREPARED'];intent=v['SEND_INTENT'];wire=base64.b64decode(intent['wireBase64'],validate=True);message=wire[65:];need(wire[0]==1 and sha(wire)==intent['identity']['wireSha256'] and sha(message)==intent['identity']['messageSha256'] and packet['reviewed']['messageBase64']==b64(message) and packet['id']==f'UPLOAD_WRITE_{phase-1:04}' and intent['operation']==phase and intent['feeReservationLamports']==FEE and intent['lifetime']==packet['lifetime'],'prior_original_identity');actual.add(sha(wire));need(parse(read(files[phase][f'native-{phase}-complete.json']))=={'complete':True},'prior_native_complete');native=parse(read(files[phase][f'native-{phase}.json']));need(native['packets']==[{'id':packet['id'],'wireBase64':intent['wireBase64']}] and native['requestId']==v['SIGN_ATTEMPT']['request']['requestId'],'prior_native_binding')
            if phase<=71:need(v['FINALIZED']['phase']==phase and v['FINALIZED']['offset']==extent(phase)[0] and v['FINALIZED']['endOffset']==sum(extent(phase)) and v['FINALIZED']['feeLamports']==FEE and v['FINALIZED']['wireSha256']==sha(wire),'prior_finalized_identity')
            else:
                need(not any('-send.' in name or '-status.' in name or '-receipt.' in name for name in files[phase]),'prior_no_send_inventory');self.old_intent=intent;self.old_packet=packet
        need(actual==verified and len(verified)==71,'prior_reused_crypto_bindings');reuse=m['cryptoReuse'];audit=parse(read(reuse['rootAuditPath']));review=parse(read(reuse['independentReviewPath']));need(reuse['exactJournalSha256']==m['journalSha256'] and reuse['exactAnchorSha256']==m['anchorSha256'] and reuse['verifiedSignatures']==71 and sha(read(reuse['rootAuditPath']))==reuse['rootAuditSha256'] and sha(read(reuse['independentReviewPath']))==reuse['independentReviewSha256'] and review['rootAuditSha256']==reuse['rootAuditSha256'] and audit['verifiedOriginalSignatures']==71 and audit['confirmedAdditionalWrites']==70 and audit['journalSha256']==m['journalSha256'] and audit['anchorSha256']==m['anchorSha256'] and audit['sourceCommit']==m['sourceCommit'] and reuse['auditSourcePath'] in m['freshFiles'],'prior_crypto_reuse_source');normal=parse(read(m['priorNoSend']['normalExitPath']));failure=parse(read(m['priorNoSend']['launcherFailurePath']));need(self.halt['phase']==72 and m['priorNoSend']['wireSha256']==self.old_intent['identity']['wireSha256'] and m['priorNoSend']['recentBlockhash']==self.old_intent['lifetime']['recentBlockhash'] and m['priorNoSend']['lastValidBlockHeight']==int(self.old_intent['lifetime']['lastValidBlockHeight']) and normal['exitCode']==1 and normal['outerTimeout'] is False and normal['automaticRetry'] is False and failure['reason']=='original_lifetime' and failure['status']=='HOLD_NO_RETRY' and failure['scopeConsumed'] is True,'prior_normal_exit_binding');self.records=records;self.files=files;self.raw_rows=[parse(r) for r in raws];self.checked=True

    def validate_semantics(self,image,ref,c):
        """Reapply the frozen pure historical reducer; never open old authority.

        Its only SDK results are reconstructed exact accepted bytes and the
        already independently verified SAME71 wire identities. All receipt,
        Clock, balances, full payload and request semantics are derived again.
        This reuse is limited to manifest-bound old wires, not a caller validity
        certificate or a path for any new original/signature.
        """
        m=self.m;source=m['sourceMap']['tools/startup_buffer_upload.py'];legacy=base.module('startup_buffer_upload_prior',read(source['frozenPath']));oldp=parse(read(m['packagePath']));authraw=read(m['authorizationPath']);view=legacy.UploadExecution.__new__(legacy.UploadExecution)
        view.p=oldp;view.package_sha=sha(read(m['packagePath']));view.auth=parse(authraw);view.authorization_sha=sha(authraw);view.ref=ref;view.roles=ref['roles'];view.image=image;view.c=c;view.sources={'tools/startup_two_operation.cjs':read(base.REPO/'tools/startup_two_operation.cjs')};view.last_slot=oldp['initialContextSlot'];view.last_clock=(oldp['initialClock']['slot'],oldp['initialClock']['unixTimestamp']);view.fees=view.observed_fees=view.sent=view.native=view.native_reserved=view.reserved=view.rpc_count=0
        finals={phase:dict(rows)['FINALIZED'] for phase,rows in self.records.items() if phase<=71};view.store=types.SimpleNamespace(path=Path(m['journalPath']).parent,closed=finals,rows=self.raw_rows,phase_names={phase:set(files) for phase,files in self.files.items()},audit=lambda full=False:self.audit(full));view.guard=lambda:self.audit()
        stages={}
        class PublicPriorWorker:
            def close(self):pass
        def same_input_public_result(unused_action):
            phase=view.index;stage=stages.get(phase,0);stages[phase]=stage+1;v=dict(self.records[phase]);packet=v['PREPARED'];intent=v['SEND_INTENT'];template=view.template(phase);at=4+32*template[3];message=template[:at]+c.b58decode(packet['lifetime']['recentBlockhash'],32)+template[at+32:];wire=base64.b64decode(intent['wireBase64'],validate=True)
            need(sha(wire) in m['verifiedWireSha256'] and wire[65:]==message and packet['reviewed']['messageBase64']==b64(message),'prior_crypto_same_input')
            if stage==0:return PublicPriorWorker()
            if stage==1:return {'wireBase64':b64(b'\x01'+bytes(64)+message),'reviewed':packet['reviewed']}
            need(stage==2,'prior_public_result_order');return {'wireSha256':sha(wire),'messageSha256':sha(message),'signatureHex':wire[1:65].hex()}
        view.sdk=same_input_public_result;view.semantic_replay();need(view.index==72 and view.fees==350000 and view.sent==70 and view.native==70,'prior_rederived_finalized_cursor')
        rows=dict(self.records[72]);packet=rows['PREPARED'];intent=rows['SEND_INTENT'];frames=[]
        for ordinal,label in ((1,'original'),(2,'before_sign'),(3,'before_send')):
            prefix=f'rpc-{ordinal:02}-{label}';request=parse(read(self.files[72][prefix+'.request.json']));http=parse(read(self.files[72][prefix+'.http.json']));values=UploadExecution.rpc_values(http['status'],read(self.files[72][prefix+'.response.json']),len(request));frames.append((label,request,values))
        view.index=72;life=packet['lifetime'];_,qs,vs=frames[0];config={'commitment':'finalized','minContextSlot':view.last_slot};need(qs==[{'jsonrpc':'2.0','id':1,'method':'getGenesisHash','params':[]},{'jsonrpc':'2.0','id':2,'method':'getLatestBlockhash','params':[config]},{'jsonrpc':'2.0','id':3,'method':'getBlockHeight','params':[config]}] and vs[0]==base.GENESIS and vs[1]['context']['slot']>=view.last_slot,'prior_unsent_original');view.last_slot=vs[1]['context']['slot'];need(life=={'recentBlockhash':vs[1]['value']['blockhash'],'lastValidBlockHeight':str(vs[1]['value']['lastValidBlockHeight']),'currentBlockHeight':str(c.u64(vs[2])),'minimumRemainingBlockHeights':'40'},'prior_unsent_lifetime')
        bound=RetiringUploadExecution._expected_prepared(view,72,packet);prepared=[r for r in self.raw_rows if r['kind']=='PREPARED' and r['data'].get('phase')==72];need(len(prepared)==1,'prior_unsent_prepared_once');digest=sha(encode(prepared[0]));binding={'id':packet['id'],'preparedRecordDigest':digest,'preparedSha256':sha(encode(packet)),'unsignedWireSha256':sha(base64.b64decode(packet['wireBase64'])),'messageSha256':packet['reviewed']['messageSha256'],'requiredSigners':packet['reviewed']['requiredSigners']};body={'schema':'piv1-startup-signing-request-v1','cluster':'testnet','genesisHash':base.GENESIS,'operatorTail':digest,'operatorSourceSha256':view.auth['operatorSourceSha256'],'authorizationSha256':view.authorization_sha,'baseParentPath':oldp['baseParentPath'],'baseParentTail':view.auth['baseParentTail'],'manifestSha256':view.package_sha,'sealSha256':None,'packets':[binding]};need(rows['SIGN_ATTEMPT']['request']=={**body,'requestId':sha(encode(body))},'prior_unsent_sign_request');view.replay_fresh(frames[1],bound,life,72);refusal=RetiringUploadExecution._replay_fresh(view,frames[2],bound,life,True);need(refusal['height']+40>int(life['lastValidBlockHeight']),'prior_unsent_normal_refusal')
        # The already scoped original-only observation deliberately preserved a
        # missing receipt as UNKNOWN. It is historical input, never a parser
        # exception for current RPC or proof of zero fee/noninclusion.
        e=m['expiryObservation'];need(parse(read(e['httpPath']))['status']==200,'prior_expiry_http');qs=parse(read(e['requestPath']));responses=parse(read(e['responsePath']));by={r['id']:r for r in responses};need(set(by)==set(range(1,7)) and [q['method'] for q in qs]==['getGenesisHash','getSignatureStatuses','getTransaction','getMultipleAccounts','getBlockHeight','isBlockhashValid'],'prior_expiry_shape');need(by[1]['result']==base.GENESIS and by[2]['result']['value']==[None] and set(by[3])=={'jsonrpc','id'} and by[5]['result']==e['expectedHeight'] and by[5]['result']>int(life['lastValidBlockHeight']) and by[6]['result']['value'] is False and qs[2]['params'][0]==m['priorNoSend']['signature'] and qs[5]['params'][0]==life['recentBlockhash'],'prior_expiry_unknown_original')
        # This historical observation requested six identities, including Clock,
        # rather than the executor's eleven-account runtime snapshot. Reapply
        # exactly its observed scope; the new bridge freshly checks all eleven.
        keys=sorted(ref['roles'][n] for n in ('g0','h2','pivProgram','programData','loaderBuffer','clock'));config={'commitment':'finalized','minContextSlot':view.last_slot}
        need(qs[3]['params']==[keys,{'encoding':'base64',**config}] and qs[4]['params']==[config] and qs[5]['params']==[life['recentBlockhash'],config] and qs[1]['params']==[[m['priorNoSend']['signature']],{'searchTransactionHistory':True}],'prior_expiry_request')
        result=by[4]['result'];context=c.u64(result['context']['slot']);need(context==e['expectedContextSlot'] and context>=view.last_slot and len(result['value'])==len(keys) and c.u64(by[2]['result']['context']['slot'])>=view.last_slot and c.u64(by[6]['result']['context']['slot'])>=view.last_slot,'prior_expiry_context');accounts={key:c.normalize_account(value) for key,value in zip(keys,result['value'])};roles=ref['roles']
        for name in ('pivProgram','programData'):need(not accounts[roles[name]]['exists'],'prior_expiry_program_present')
        for name,amount in (('g0',999560000),('h2',49204880)):need(accounts[roles[name]]=={'exists':True,'owner':base.SYSTEM,'executable':False,'lamports':str(amount),'dataBase64':''},'prior_expiry_payer')
        payload=struct.pack('<IB',1,1)+c.b58decode(roles['g0'],32)+image[:RETIRE_START]+bytes(END-RETIRE_START);need(accounts[roles['loaderBuffer']]=={'exists':True,'owner':base.LOADER,'executable':False,'lamports':str(base.RENT),'dataBase64':b64(payload)},'prior_expiry_buffer')
        clock=accounts[roles['clock']];need(clock['exists'] and clock['owner']==base.SYSVAR and not clock['executable'] and int(clock['lamports'])>0,'prior_expiry_clock_owner');raw=base64.b64decode(clock['dataBase64'],validate=True);need(len(raw)==40,'prior_expiry_clock_size');slot,_,_,_,unix=struct.unpack('<QqQQq',raw);need(view.last_slot<=slot<=context and slot>=view.last_clock[0] and unix>=view.last_clock[1],'prior_expiry_clock_regression');self.audit(True)


class RetiringUploadExecution(UploadExecution):
    """D-074/D-075/D-077 remaining upload; never reopens a consumed scope.

    Healthy originals share one worker. ONLY a fully retained before-send
    lifetime refusal returns a typed stop. Retirement requires a later worker
    plus the trusted attestor's actual normal-process-exit receipt; a local
    exception handler cannot mint that receipt or authorize a replacement.
    """
    first_phase=RETIRE_FIRST
    start_offset=RETIRE_START
    native_cap=NATIVE_CAP
    send_cap=SEND_CAP
    retire_cap=RETIRE_CAP
    send_fee_cap=SEND_FEE_CAP
    reservation_cap=RESERVATION_CAP
    history_type=PriorUploadHistory
    store_type=RetiringStore
    def eligible_stop(self,exc):return str(exc)=='original_lifetime'
    def __init__(self,package_path,package_sha,authorization_path,authorization_sha,profile_path,evidence_path):
        self.package_path=Path(package_path);self.raw=read(self.package_path,2*1024*1024);need(sha(self.raw)==package_sha,'package_pin');self.p=parse(self.raw);self.package_sha=package_sha
        self.authorization_path=Path(authorization_path);self.authorization_sha=authorization_sha;self.profile_path=profile_path;self.auth_raw=read(self.authorization_path,32768);self.auth=parse(self.auth_raw)
        self.start=time.monotonic();self.deadline=self.start+min(self.p['wallSeconds'],28800);self.timings=[];self.index=self.first_phase
        self.normal_exit_directory_identity=self.p['normalExitDirectoryIdentity']
        self.sent=self.native=self.native_reserved=self.fees=self.observed_fees=self.observed_rent=self.reserved=self.retirements=self.rpc_count=0
        self.last_slot=self.p['initialContextSlot'];self.last_clock=(self.p['initialClock']['slot'],self.p['initialClock']['unixTimestamp']);self.current=None;self.sources={};self.worker=None;self.guard_busy=False;self.guard_ready=False;self.normal_exit_pins={};self.refusal=None
        self.prior=self.history_type(self.p['priorHistoryPath'],self.p['priorHistorySha256'])
        self.store=self.store_type(evidence_path,package_sha,authorization_sha,self.p['progressAnchorPath']);self.store.boundary=self.guard
        self.deadline=min(self.deadline,self.start+max(0,self.store.rows[0]['data']['startedUnix']+self.p['wallSeconds']-int(time.time())))
        need(str(self.store.path)==self.p['evidencePath'],'evidence_package_binding')
        try:
            self.audit_static();self.prior.validate_semantics(self.image,self.ref,self.c);self._load_existing_exits();self.guard_ready=True;self.semantic_replay()
        except BaseException as exc:
            self.retain_hold({'status':'HOLD_NO_RETRY','phase':self.index,'reason':'initialization_'+(str(exc) if isinstance(exc,Stop) else type(exc).__name__)})
            self.store.close();raise
    def source_audit(self):
        super().source_audit();self.prior.audit()
        need(sha(read(self.p['exitAttestorPath']))==self.p['exitAttestorSha256'],'exit_attestor_changed')
        for path,want in self.normal_exit_pins.items():need(sha(read(path))==want,'normal_exit_evidence_changed')
        directory=Path(self.p['normalExitDirectory']);s=directory.lstat();need(stat.S_ISDIR(s.st_mode) and s.st_uid==os.getuid() and stat.S_IMODE(s.st_mode)==0o700 and {'dev':s.st_dev,'ino':s.st_ino,'uid':s.st_uid,'mode':stat.S_IMODE(s.st_mode)}==self.normal_exit_directory_identity,'exit_directory_security')
        if self.guard_ready:need(set(os.listdir(directory))=={Path(p).name for p in self.normal_exit_pins if Path(p).parent==directory},'exit_directory_inventory')
    def audit_static(self):
        p=self.p;need(p['schema']=='piv1-dedicated-buffer-upload-retirement-v1' and p['cluster']=='testnet' and p['genesisHash']==base.GENESIS and p['authority']=='D-074' and p['securityPolicies']==['D-075','D-077'] and p['firstPhase']==RETIRE_FIRST and p['lastPhase']==934 and p['startOffset']==RETIRE_START and p['endOffset']==END and p['nativeCap']==NATIVE_CAP and p['sendCap']==SEND_CAP and p['retirementCap']==RETIRE_CAP and p['feeCapLamports']==SEND_FEE_CAP and p['reservationCapLamports']==RESERVATION_CAP and p['perOperationFeeCapLamports']==FEE and p['rentLamports']==0 and p['oldPhase72ReserveLamports']==5000 and p['historicalUnknownReserveLamports']==5000 and 0<p['wallSeconds']<=28800,'retirement_package_scope')
        need(set(base.REQUIRED_SOURCES)|{'tools/startup_buffer_upload.py','tools/startup_public_journal.py'}<=set(p['sourcePins']),'incomplete_source_pins');self.source_audit();need(p['roles']=={k:self.roles[k] for k in base.NAMES},'role_binding')
        need(p['initialG0Lamports']==999560000 and p['initialH2Lamports']==49204880 and p['initialG0Lamports']>=RESERVATION_CAP+10000 and p['baseParentPath'] in p['historyDirectories'] and p['authorityBindingPath'] in p['historyPins'],'retirement_budget_or_parent')
        need(type(p['initialContextSlot']) is int and set(p['initialClock'])=={'slot','unixTimestamp'} and all(type(v) is int for v in p['initialClock'].values()) and 0<=p['initialClock']['slot']<=p['initialContextSlot'],'initial_clock_floor')
        self.authorized();self.prior.audit(True);self.store.audit(True)
    def resources(self):
        need(time.monotonic()<self.deadline,'execution_deadline');v=os.statvfs(self.store.path);need(v.f_bavail*v.f_frsize>=2*1024**3,'disk_reserve');need(self.p['externalEvidenceReserveBytes']==268435456 and len(self.store.journal_raw)+len(self.store.anchor_raw)+self.store.total_bytes<=8*1024**3-self.p['externalEvidenceReserveBytes'],'evidence_budget')
    def authorized(self):
        super().authorized();need(self.native_reserved<=self.native_cap and self.sent<=self.send_cap and self.retirements<=self.retire_cap and self.reserved<=self.reservation_cap and self.fees<=self.send_fee_cap,'global_retirement_caps')
    def _record_value(self,attempt,kind):
        rows=[r for r in self.store.rows if r['kind']==kind and r['data'].get('phase')==attempt];need(len(rows)==1,'attempt_record_once');r=rows[0];raw=read(self.store.path/f'phase-{attempt:04}'/r['data']['file']);need(sha(raw)==r['data']['sha256'],'attempt_record_pin');v=parse(raw);need(v['kind']==kind,'attempt_record_kind');return r,v['data']
    def _candidate_outcome(self,attempt):
        row,candidate=self._record_value(attempt,'UNSENT_LIFETIME_REFUSAL');return {'status':'UNSENT_LIFETIME_STOP_PENDING_EXIT','attemptId':attempt,'logicalPhase':candidate['logicalPhase'],'candidateRecordName':row['data']['file'],'candidateRecordSha256':row['data']['sha256'],'journalSha256':sha(self.store.journal_raw),'anchorSha256':sha(self.store.anchor_raw),'journalTail':self.store.tail,'wireSha256':candidate['wireSha256'],'signature':candidate['signature'],'nativeCalls':candidate['nativeCalls'],'nativeReservations':candidate['nativeReservations'],'sendCalls':candidate['sendCalls'],'reservedFeeLamports':candidate['reservedFeeLamports']}
    def _validate_exit(self,attempt,expected=None):
        row,candidate=self._record_value(attempt,'UNSENT_LIFETIME_REFUSAL');path=Path(self.p['normalExitDirectory'])/f'attempt-{attempt:04}.public.json';raw=read(path,32768);receipt=parse(raw)
        keys={'schema','packageSha256','authorizationSha256','attestorSha256','attemptId','logicalPhase','candidateRecordSha256','journalSha256','anchorSha256','journalTail','workerResultSha256','workerResultPath','workerPid','exitCode','normalWait','childrenReaped','timeout','platformStop','resourceStop','nativeCalls','nativeReservations','sendCalls','reservedFeeLamports','startedUnix','completedUnix','stdoutSha256','stderrSha256'};need(set(receipt)==keys and receipt['schema']=='piv1-d077-normal-unsent-exit-v1' and receipt['packageSha256']==self.package_sha and receipt['authorizationSha256']==self.authorization_sha and receipt['attestorSha256']==self.p['exitAttestorSha256'] and receipt['attemptId']==attempt and receipt['logicalPhase']==candidate['logicalPhase'] and receipt['candidateRecordSha256']==row['data']['sha256'],'normal_exit_binding')
        need(receipt['exitCode']==0 and receipt['normalWait'] is True and receipt['childrenReaped'] is True and receipt['timeout'] is False and receipt['platformStop'] is False and receipt['resourceStop'] is None and type(receipt['workerPid']) is int and receipt['workerPid']>0 and type(receipt['startedUnix']) is int and type(receipt['completedUnix']) is int and self.store.rows[0]['data']['startedUnix']<=receipt['startedUnix']<=self.store.attempts[attempt]['startedUnix']<=receipt['completedUnix']<=int(time.time()),'normal_exit_not_complete')
        for key in ('nativeCalls','nativeReservations','sendCalls','reservedFeeLamports'):need(receipt[key]==candidate[key],'normal_exit_counts')
        result_path=Path(receipt['workerResultPath']);need(result_path.parent==Path(self.p['workerResultDirectory']) and result_path.name==f'worker-result-{receipt["workerPid"]}.public.json','normal_exit_result_path');result_raw=read(result_path,32768);outcome=parse(result_raw);need(sha(result_raw)==receipt['workerResultSha256'] and result_raw==encode(outcome),'normal_exit_result_bytes')
        need(outcome=={'status':'UNSENT_LIFETIME_STOP_PENDING_EXIT','attemptId':attempt,'logicalPhase':candidate['logicalPhase'],'candidateRecordName':row['data']['file'],'candidateRecordSha256':row['data']['sha256'],'journalSha256':receipt['journalSha256'],'anchorSha256':receipt['anchorSha256'],'journalTail':receipt['journalTail'],'wireSha256':candidate['wireSha256'],'signature':candidate['signature'],'nativeCalls':candidate['nativeCalls'],'nativeReservations':candidate['nativeReservations'],'sendCalls':candidate['sendCalls'],'reservedFeeLamports':candidate['reservedFeeLamports']},'normal_exit_outcome')
        # The stopped prefix must equal the exact anchor/journal at that normal
        # return; later retirement rows may follow it, never alter that prefix.
        cutoff=row['sequence']+1;prefix=b''.join(encode(r) for r in self.store.rows[:cutoff]);anchors=[];tail=None
        for r in self.store.rows[:cutoff]:rr=encode(r);anchors.append(encode({'sequence':r['sequence'],'digest':sha(rr),'previous':tail}));tail=sha(rr)
        need(sha(prefix)==receipt['journalSha256'] and sha(b''.join(anchors))==receipt['anchorSha256'] and tail==receipt['journalTail'],'normal_exit_prefix')
        if expected is not None:need(receipt==expected,'normal_exit_retirement_changed')
        self.normal_exit_pins[str(path)]=sha(raw);self.normal_exit_pins[str(result_path)]=sha(result_raw)
        for suffix,key in (('stdout','stdoutSha256'),('stderr','stderrSha256')):
            stream=Path(self.p['workerResultDirectory'])/f'worker-{receipt["workerPid"]}.{suffix}';stream_raw=read(stream,1024*1024);need(sha(stream_raw)==receipt[key],'normal_exit_raw_stream');need(stream_raw==(result_raw if suffix=='stdout' else b''),'normal_exit_stream_semantics');self.normal_exit_pins[str(stream)]=sha(stream_raw)
        return receipt
    def _load_existing_exits(self):
        for attempt in sorted(self.store.attempts):
            if attempt and (attempt in self.store.retired or attempt==self.store.pending):self._validate_exit(attempt)
    def snapshot(self,response,completed,floor):
        context=self.c.u64(response['context']['slot']);need(context>=floor and completed in (self.index,self.index+1),'snapshot_context');r=self.roles;keys=sorted({base.SYSTEM}|{r[k] for k in base.NAMES});need(type(response['value']) is list and len(response['value'])==len(keys),'snapshot_coverage');a={k:self.c.normalize_account(v) for k,v in zip(keys,response['value'])};fees=self.fees+(FEE if completed==self.index+1 else 0)
        for name in ('pivProgram','programData'):need(not a[r[name]]['exists'],'program_already_exists')
        for name,amount in [('g0',self.p['initialG0Lamports']-fees),('h2',self.p['initialH2Lamports'])]:need(a[r[name]]=={'exists':True,'owner':base.SYSTEM,'executable':False,'lamports':str(amount),'dataBase64':''},'payer_amount_or_authority')
        need(self.p['initialG0Lamports']-self.reservation_cap-self.p.get('oldSeparateReservesLamports',10000)>=0 and fees<=self.send_fee_cap,'conservative_funding_floor');uploaded=min((completed-1)*CHUNK,END);data=struct.pack('<IB',1,1)+self.c.b58decode(r['g0'],32)+self.image[:uploaded]+bytes(END-uploaded);need(a[r['loaderBuffer']]=={'exists':True,'owner':base.LOADER,'executable':False,'lamports':str(base.RENT),'dataBase64':b64(data)},'buffer_exact_readback')
        for key in [base.SYSTEM]+[r[n] for n in ('loaderProgram','computeBudget','rent','instructions')]:
            q=a[key];raw=base64.b64decode(q['dataBase64'],validate=True) if q['exists'] else b'';digest={'exists':q['exists'],'owner':q['owner'],'executable':q['executable'],'lamports':q['lamports'],'dataBytes':len(raw),'dataSha256':sha(raw)} if q['exists'] else {'exists':False,'owner':None,'executable':None,'lamports':None,'dataBytes':None,'dataSha256':None};need(digest==self.ref['baseline'][key],'runtime_account_changed')
        clock=a[r['clock']];need(clock['exists'] and clock['owner']==base.SYSVAR and not clock['executable'] and int(clock['lamports'])>0,'clock_owner');raw=base64.b64decode(clock['dataBase64'],validate=True);need(len(raw)==40,'clock_size');slot,_,_,_,unix=struct.unpack('<QqQQq',raw);need(floor<=slot<=context and (self.last_clock is None or slot>=self.last_clock[0] and unix>=self.last_clock[1]),'clock_regression');self.last_clock=(slot,unix);self.last_slot=context;return a,context
    def fresh(self,label,bound,life):
        self.guard();floor=self.last_slot;config={'commitment':'finalized','minContextSlot':floor};keys=sorted({base.SYSTEM}|{self.roles[k] for k in base.NAMES})
        genesis,quote,accounts,valid,height,rent=self.rpc(label,[('getGenesisHash',[]),('getFeeForMessage',[bound['reviewed']['messageBase64'],config]),('getMultipleAccounts',[keys,{'encoding':'base64',**config}]),('isBlockhashValid',[life['recentBlockhash'],config]),('getBlockHeight',[config]),('getMinimumBalanceForRentExemption',[892389,{'commitment':'finalized'}])])
        need(genesis==base.GENESIS and rent==base.RENT,'cluster_or_rent');height=self.c.u64(height);need(type(valid['value']) is bool and self.c.u64(valid['context']['slot'])>=floor and height>=int(life['currentBlockHeight']),'lifetime_reply');need(self.c.u64(quote['context']['slot'])>=floor and quote['value']==FEE and self.fees+FEE<=SEND_FEE_CAP,'fee_changed');self.pre_accounts,_=self.snapshot(accounts,self.index,floor);self.last_slot=max(self.last_slot,quote['context']['slot'],valid['context']['slot'])
        self.current={'height':height,'remaining':int(life['lastValidBlockHeight'])-height,'contextFloor':self.last_slot,'responseAtMonotonic':self.last_rpc_response_at};self.timings.append({'operation':self.index,'stage':label+'_margin',**self.current,'atSeconds':time.monotonic()-self.start})
        if not(valid['value'] is True and height+40<=int(life['lastValidBlockHeight'])):
            if label=='before_send' and ((valid['value'] is True and 0<=self.current['remaining']<40) or (valid['value'] is False and height>int(life['lastValidBlockHeight']))):self.refusal={'stage':label,'height':height,'valid':valid['value'],'lastValidBlockHeight':int(life['lastValidBlockHeight']),'contextFloor':self.last_slot}
            raise Stop('original_lifetime')
        self.fresh_age();return self.current
    def sign(self,packet):
        need(self.native_reserved<self.native_cap and self.reserved+FEE<=self.reservation_cap,'native_reservation_cap');self.reserved+=FEE;return super().sign(packet)
    def _final_fresh_and_send(self,wire,life):
        # Inherited macro's historical counter equalities are specific to v1;
        # the new scope uses physical durable native attempts and actual sends.
        self.fresh('before_send',self.bound,life);need(self.sent<self.send_cap and self.native<=self.native_cap and self.native==self.native_reserved,'at_most_once_budget');self.store.current_record(self.intent_name);need(self.rpc_count<933*15+8 and self.rpc_count-self.phase_rpc_start<15,'rpc_budget');self.rpc_count+=1
        prefix=f'rpc-{self.rpc_count-self.phase_rpc_start:02}-send';raw=encode([{'jsonrpc':'2.0','id':1,'method':'sendTransaction','params':[wire,{'encoding':'base64','skipPreflight':False,'maxRetries':0,'preflightCommitment':'finalized','minContextSlot':self.last_slot}]}]);self.fresh_age();self.sent+=1
        try:status,headers,body=self.transport_leaf.public_rpc_transport(self.c,raw);self.last_rpc_response_at=time.monotonic()
        except BaseException as exc:
            partial=getattr(exc,'partial',None)
            def failure():
                self.store.write(prefix+'.request.json',raw)
                if partial:self.store.write(prefix+'.partial.bin',partial)
            self.store.capture(failure);raise
        self.store.capture(lambda:(self.store.write(prefix+'.request.json',raw),self.store.write(prefix+'.response.json',body),self.store.write(prefix+'.http.json',encode({'status':status,'headers':headers}))));ack,=self.rpc_values(status,body,1);return ack
    def _normal_refusal(self,packet,wire,identity):
        need(self.refusal is not None and self.refusal['stage']=='before_send' and self.native==self.native_reserved and self.retirements<self.retire_cap,'unsent_refusal_eligibility')
        attempt=self.store.phase;rows=[r['kind'] for r in self.store.rows if r['data'].get('phase')==attempt and r['kind'] not in ('BEGIN','FILE')];need(rows==['PREPARED','SIGN_ATTEMPT','SEND_INTENT'],'unsent_refusal_sequence');need(not any('-send.' in n or '.partial.' in n for n in self.store.phase_names[attempt]),'unsent_no_send_inventory')
        need(parse(read(self.store.path/f'phase-{attempt:04}'/f'native-{self.index}-complete.json'))=={'complete':True},'unsent_native_incomplete');native=parse(read(self.store.path/f'phase-{attempt:04}'/f'native-{self.index}.json'));_,signing=self._record_value(attempt,'SIGN_ATTEMPT');need(native=={'schema':'piv1-startup-signing-response-v1','requestId':signing['request']['requestId'],'packets':[{'id':packet['id'],'wireBase64':wire}]},'unsent_native_binding');checked=self.sdk(lambda:self.worker.call({'action':'VERIFY','wireBase64':wire}));need(checked==identity,'unsent_crypto_binding')
        self.guard();proof={'logicalPhase':self.index,'wireSha256':identity['wireSha256'],'signature':self.c.b58encode(bytes.fromhex(identity['signatureHex'])),'refusal':self.refusal,'nativeCalls':self.native,'nativeReservations':self.native_reserved,'sendCalls':self.sent,'reservedFeeLamports':self.reserved,'sendCallsBeforeAttempt':self.attempt_send_start,'nativeCallsBeforeAttempt':self.attempt_native_start,'sourceSha256':self.p['sourcePins']['tools/startup_buffer_upload.py'],'normalReturn':'BEFORE_SEND_ORIGINAL_LIFETIME_ONLY','requiredNextAction':'TRUSTED_NORMAL_PROCESS_EXIT_THEN_READONLY_EXPIRY'}
        need(proof['sendCalls']==proof['sendCallsBeforeAttempt'] and proof['nativeCalls']==proof['nativeCallsBeforeAttempt']+1,'unsent_callback_counts');self.store.record('UNSENT_LIFETIME_REFUSAL',proof);self.store.audit(True);return self._candidate_outcome(attempt)
    def _expiry_observe(self,intent,bridge=False):
        signature=self.c.b58encode(bytes.fromhex(intent['identity']['signatureHex']));life=intent['lifetime'];deadline=time.monotonic()+30;previous_height=int(life['currentBlockHeight']);floor=self.last_slot
        for observation in range(8):
            need(time.monotonic()<deadline,'expiry_observation_bound');config={'commitment':'finalized','minContextSlot':self.last_slot};keys=sorted({base.SYSTEM}|{self.roles[k] for k in base.NAMES})
            genesis,height,valid,accounts,statuses=self.rpc('expiry',[('getGenesisHash',[]),('getBlockHeight',[config]),('isBlockhashValid',[life['recentBlockhash'],config]),('getMultipleAccounts',[keys,{'encoding':'base64',**config}]),('getSignatureStatuses',[[signature],{'searchTransactionHistory':True}])])
            need(time.monotonic()<deadline and genesis==base.GENESIS,'expiry_bound_or_cluster');height=self.c.u64(height);need(height>=previous_height and type(valid['value']) is bool and self.c.u64(valid['context']['slot'])>=self.last_slot,'expiry_lifetime_reply');previous_height=height
            need(type(statuses['value']) is list and len(statuses['value'])==1 and self.c.u64(statuses['context']['slot'])>=self.last_slot,'expiry_status_reply');need(statuses['value'][0] is None,'unsent_chain_status_contradiction')
            self.snapshot(accounts,self.index,max(floor,self.last_slot));self.last_slot=max(self.last_slot,self.c.u64(valid['context']['slot']),self.c.u64(statuses['context']['slot']))
            if height>int(life['lastValidBlockHeight']) and valid['value'] is False:
                return {'signature':signature,'wireSha256':intent['identity']['wireSha256'],'recentBlockhash':life['recentBlockhash'],'lastValidBlockHeight':int(life['lastValidBlockHeight']),'finalizedHeight':height,'hashValid':False,'contextFloor':self.last_slot,'accountsSha256':sha(encode(accounts)),'originalStatus':'UNKNOWN_NULL','originalReceipt':'UNOBSERVED_UNKNOWN','feeLamports':'UNKNOWN_RESERVED_5000','feeReleased':False,'observationCount':observation+1,'priorBridge':bridge}
            need(observation<7 and time.monotonic()+1<deadline,'unsent_hash_not_expired_hold');time.sleep(1);self.guard()
        raise Stop('unsent_hash_not_expired_hold')
    def _retire_pending(self):
        attempt=self.store.pending;need(attempt is not None and attempt>0 and self.retirements<self.retire_cap,'retirement_pending_required');receipt=self._validate_exit(attempt);_,candidate=self._record_value(attempt,'UNSENT_LIFETIME_REFUSAL');_,intent=self._record_value(attempt,'SEND_INTENT')
        self.index=candidate['logicalPhase'];self.phase_rpc_start=self.rpc_count-len(self._frames(attempt));self.phase_deadline=self.start+max(0,self.store.attempts[attempt]['startedUnix']+120-int(time.time()));self.guard();observation=self._expiry_observe(intent)
        retired={'attemptId':attempt,'logicalPhase':self.index,'wireSha256':intent['identity']['wireSha256'],'normalExitReceiptSha256':sha(encode(receipt)),'expiry':observation,'reservationLamports':FEE,'feeReleased':False,'oldPacketPermanentlyForbidden':True,'nextOffset':extent(self.index)[0]};self.prior.audit(True);self.store.disposition('RETIRED_UNSENT',retired);self.retirements+=1;self.refusal=None
    def _initial_bridge(self):
        self.store.begin(self.index,bridge=True);self.phase_rpc_start=self.rpc_count;self.phase_deadline=time.monotonic()+120;observation=self._expiry_observe(self.prior.old_intent,bridge=True)
        self.prior.audit(True);self.store.disposition('PRIOR_BRIDGE',{'logicalPhase':RETIRE_FIRST,'offset':RETIRE_START,'priorManifestSha256':self.p['priorHistorySha256'],'oldWireSha256':self.prior.old_intent['identity']['wireSha256'],'expiry':observation,'oldPhase72ReserveLamports':5000,'oldAuReserveLamports':5000,'countsAgainstNewRetirementCap':False,'oldScopeAdopted':False})
    def run(self):
        try:
            if self.store.bridge is None:self._initial_bridge()
            if self.store.pending is not None:self._retire_pending()
            while self.index<=934:
                if time.monotonic()+120>=self.deadline or int(time.time())+120>=min(self.p['expiresUnix'],int(self.auth['expiresUnixTimestamp'])):
                    self.store.pause();return {'status':'PAUSED_CLEAN','nextPhase':self.index,'nextOffset':extent(self.index)[0],'confirmedFeeLamports':self.fees,'nativeCalls':self.native,'sendCalls':self.sent,'retirements':self.retirements}
                need(self.native_reserved<self.native_cap and self.sent<self.send_cap and self.reserved+FEE<=self.reservation_cap,'remaining_global_caps');self.audit_static();self.store.begin(self.index);self.phase_rpc_start=self.rpc_count;self.phase_deadline=time.monotonic()+120;self.attempt_send_start=self.sent;self.attempt_native_start=self.native;self.refusal=None
                self.worker=self.sdk(lambda:base.Sdk(self.sources['tools/startup_two_operation.cjs'],1,self.template(self.index)));self.guard()
                genesis,original,height=self.rpc('original',[('getGenesisHash',[]),('getLatestBlockhash',[{'commitment':'finalized','minContextSlot':self.last_slot}]),('getBlockHeight',[{'commitment':'finalized','minContextSlot':self.last_slot}])]);need(genesis==base.GENESIS and original['context']['slot']>=self.last_slot,'original_cluster_context');self.last_slot=original['context']['slot'];self.bound=self.sdk(lambda:self.worker.call({'action':'BIND','blockhash':original['value']['blockhash']}));template=self.template(self.index);offset=4+32*template[3];need(base64.b64decode(self.bound['reviewed']['messageBase64'])==template[:offset]+self.c.b58decode(original['value']['blockhash'],32)+template[offset+32:],'bound_message_mismatch')
                life={'recentBlockhash':original['value']['blockhash'],'lastValidBlockHeight':str(original['value']['lastValidBlockHeight']),'currentBlockHeight':str(self.c.u64(height)),'minimumRemainingBlockHeights':'40'};packet={'id':f'UPLOAD_WRITE_{self.index-1:04}','wireBase64':self.bound['wireBase64'],'reviewed':self.bound['reviewed'],'lifetime':life,'feeQuoteLamports':str(FEE)};self.store.record('PREPARED',packet);wire=self.sign(packet);identity=self.sdk(lambda:self.worker.call({'action':'VERIFY','wireBase64':wire}))
                self.store.record('SEND_INTENT',{'operation':self.index,'wireBase64':wire,'identity':identity,'lifetime':life,'feeReservationLamports':FEE,'sendPolicy':{'encoding':'base64','skipPreflight':False,'maxRetries':0,'preflightCommitment':'finalized','minContextSlotRule':'MAX_FINAL_FRESH_CONTEXTS'}});self.intent_name=self.store.last_record_name
                try:ack=self._final_fresh_and_send(wire,life)
                except Stop as exc:
                    if not self.eligible_stop(exc):raise
                    return self._normal_refusal(packet,wire,identity)
                need(ack==self.c.b58encode(bytes.fromhex(identity['signatureHex'])),'send_acknowledgement_hold');result=self.confirm(wire,identity);self.audit_static();self.store.disposition('FINALIZED',result);self.fees+=FEE;self.worker.close();self.worker=None;self.index+=1;need(self.rpc_count-self.phase_rpc_start<=15,'phase_rpc_budget')
            self.audit_static();self.store.pause();return {'status':'FULL_BUFFER_VERIFIED','nextPhase':935,'uploadedBytes':END,'artifactSha256':base.ELF_SHA,'confirmedFeeLamports':self.fees,'nativeCalls':self.native,'sendCalls':self.sent,'retirements':self.retirements,'reservedFeeLamports':self.reserved,'oldPhase72ReserveLamports':5000,'oldAuReserveLamports':5000,'lastFinalized':self.store.finalized[max(self.store.finalized)]}
        except BaseException as exc:
            hold={'status':'HOLD_NO_RETRY','reason':str(exc) if isinstance(exc,Stop) else type(exc).__name__,'phase':self.index,'sendCalls':self.sent,'nativeCalls':self.native,'nativeReservations':self.native_reserved,'confirmedFeeLamports':self.observed_fees,'admittedFeeLamports':self.fees,'reservedFeeLamports':self.reserved,'retirements':self.retirements};self.retain_hold(hold);raise Stop(hold['reason']) from None
        finally:
            if self.worker is not None:self.worker.close()
            self.store.close()
    def _frames(self,attempt):
        folder=self.store.path/f'phase-{attempt:04}';names=self.store.phase_names[attempt];frames=[]
        for ordinal,name in enumerate(sorted(n for n in names if n.endswith('.request.json')),1):
            need(name.startswith(f'rpc-{ordinal:02}-'),'replay_rpc_order');prefix=name[:-len('.request.json')];request=parse(read(folder/name));body=read(folder/(prefix+'.response.json'));http=parse(read(folder/(prefix+'.http.json')));values=self.rpc_values(http['status'],body,len(request))
            for n,q in enumerate(request,1):need(set(q)=={'jsonrpc','id','method','params'} and q['jsonrpc']=='2.0' and q['id']==n,'replay_rpc_request')
            frames.append((prefix.split('-',2)[2],request,values))
        need(len(frames)<=15,'replay_rpc_budget');allowed={n for n in names if not n.startswith('rpc-')}|{f'rpc-{n:02}-{f[0]}'+suffix for n,f in enumerate(frames,1) for suffix in ('.request.json','.response.json','.http.json')};need(allowed==names,'replay_raw_inventory');return frames
    def _verify_many_public(self,packets):
        """One bounded public-only checker call; no private data or SDK migration."""
        need(0<len(packets)<=NATIVE_CAP,'public_replay_crypto_bound');payload=encode(packets);need(len(payload)<=8*1024*1024,'public_replay_input_bound')
        extension="\nmodule.exports.verifyMany=xs=>xs.map(x=>{const d=checker.verifyPacket(x);need(sdk.Transaction.from(d.wire).serialize().equals(d.wire),'signed_exact');return {wireSha256:d.wireSha256,messageSha256:d.messageSha256,signatureHex:d.firstSignatureHex};});\n"
        script="const fs=require('node:fs'),Module=require('node:module');const m=new Module(process.argv[1]);m.filename=process.argv[1];m.paths=Module._nodeModulePaths(require('node:path').dirname(m.filename));m._compile(Buffer.from(process.argv[2],'base64').toString(),m.filename);const raw=fs.readFileSync(0);if(raw.length>8388608)process.exit(2);const out=JSON.stringify(m.exports.verifyMany(JSON.parse(raw)));if(out.length>1048576)process.exit(3);process.stdout.write(out);"
        self.guard();result=base.subprocess.run([str(base.NODE),'-e',script,str(base.REPO/'tools/startup_two_operation.cjs'),b64(self.sources['tools/startup_two_operation.cjs']+extension.encode())],input=payload,stdout=base.subprocess.PIPE,stderr=base.subprocess.PIPE,cwd=base.REPO,env={'PIV1_PUBLIC_CHECK_REPO':str(base.REPO)},timeout=20,check=False,preexec_fn=base.no_core);self.guard();need(result.returncode==0 and not result.stderr and len(result.stdout)<=1024*1024,'public_replay_crypto');values=parse(result.stdout);need(type(values) is list and len(values)==len(packets),'public_replay_crypto_result');return values
    def _expected_prepared(self,logical,packet):
        template=self.template(logical);at=4+32*template[3];message=template[:at]+self.c.b58decode(packet['lifetime']['recentBlockhash'],32)+template[at+32:];reviewed={'messageBase64':b64(message),'messageSha256':sha(message),'feePayer':self.roles['g0'],'requiredSigners':[self.roles['g0']],'accountRoles':[{'address':self.roles[k],'signer':k=='g0','writable':k in ('g0','loaderBuffer')} for k in ('g0','loaderBuffer','computeBudget','loaderProgram')],'lookupTables':[]};expected={'id':f'UPLOAD_WRITE_{logical-1:04}','wireBase64':b64(b'\x01'+bytes(64)+message),'reviewed':reviewed,'lifetime':packet['lifetime'],'feeQuoteLamports':str(FEE)};need(packet==expected,'replay_prepared');return {'wireBase64':expected['wireBase64'],'reviewed':reviewed}
    def _replay_fresh(self,frame,bound,life,failed=False):
        _,qs,vs=frame;need([q['method'] for q in qs]==['getGenesisHash','getFeeForMessage','getMultipleAccounts','isBlockhashValid','getBlockHeight','getMinimumBalanceForRentExemption'],'replay_fresh_methods');genesis,quote,accounts,valid,height,rent=vs;config=qs[1]['params'][1];floor=self.last_slot;need(config=={'commitment':'finalized','minContextSlot':floor},'replay_fresh_floor')
        need(qs[0]['params']==[] and qs[1]['params'][0]==bound['reviewed']['messageBase64'] and qs[2]['params']==[sorted({base.SYSTEM}|{self.roles[k] for k in base.NAMES}),{'encoding':'base64',**config}] and qs[3]['params']==[life['recentBlockhash'],config] and qs[4]['params']==[config] and qs[5]['params']==[892389,{'commitment':'finalized'}],'replay_fresh_params');height=self.c.u64(height);need(genesis==base.GENESIS and rent==base.RENT and quote['value']==FEE and self.c.u64(quote['context']['slot'])>=floor and type(valid['value']) is bool and self.c.u64(valid['context']['slot'])>=floor and height>=int(life['currentBlockHeight']),'replay_fresh_values');self.pre_accounts,_=self.snapshot(accounts,self.index,floor);self.last_slot=max(self.last_slot,quote['context']['slot'],valid['context']['slot']);remaining=int(life['lastValidBlockHeight'])-height
        if failed:need((valid['value'] is True and 0<=remaining<40) or (valid['value'] is False and height>int(life['lastValidBlockHeight'])),'replay_normal_refusal');return {'stage':'before_send','height':height,'valid':valid['value'],'lastValidBlockHeight':int(life['lastValidBlockHeight']),'contextFloor':self.last_slot}
        need(valid['value'] is True and remaining>=40,'replay_original_lifetime')
    def _replay_expiry(self,frames,intent,bridge=False):
        need(1<=len(frames)<=8 and all(f[0]=='expiry' for f in frames),'replay_expiry_bound');signature=self.c.b58encode(bytes.fromhex(intent['identity']['signatureHex']));life=intent['lifetime'];height=int(life['currentBlockHeight']);last=None
        for n,(_,qs,vs) in enumerate(frames):
            config={'commitment':'finalized','minContextSlot':self.last_slot};need(qs==[{'jsonrpc':'2.0','id':1,'method':'getGenesisHash','params':[]},{'jsonrpc':'2.0','id':2,'method':'getBlockHeight','params':[config]},{'jsonrpc':'2.0','id':3,'method':'isBlockhashValid','params':[life['recentBlockhash'],config]},{'jsonrpc':'2.0','id':4,'method':'getMultipleAccounts','params':[sorted({base.SYSTEM}|{self.roles[k] for k in base.NAMES}),{'encoding':'base64',**config}]},{'jsonrpc':'2.0','id':5,'method':'getSignatureStatuses','params':[[signature],{'searchTransactionHistory':True}]}],'replay_expiry_request');genesis,new_height,valid,accounts,statuses=vs;new_height=self.c.u64(new_height);need(genesis==base.GENESIS and new_height>=height and type(valid['value']) is bool and self.c.u64(valid['context']['slot'])>=self.last_slot and statuses['value']==[None] and self.c.u64(statuses['context']['slot'])>=self.last_slot,'replay_expiry_values');height=new_height;self.snapshot(accounts,self.index,self.last_slot);self.last_slot=max(self.last_slot,valid['context']['slot'],statuses['context']['slot']);expired=height>int(life['lastValidBlockHeight']) and valid['value'] is False;need(not expired or n==len(frames)-1,'replay_expiry_after_complete');last={'signature':signature,'wireSha256':intent['identity']['wireSha256'],'recentBlockhash':life['recentBlockhash'],'lastValidBlockHeight':int(life['lastValidBlockHeight']),'finalizedHeight':height,'hashValid':False,'contextFloor':self.last_slot,'accountsSha256':sha(encode(accounts)),'originalStatus':'UNKNOWN_NULL','originalReceipt':'UNOBSERVED_UNKNOWN','feeLamports':'UNKNOWN_RESERVED_5000','feeReleased':False,'observationCount':n+1,'priorBridge':bridge}
        need(expired,'replay_expiry_not_proved');return last
    def _replay_success(self,frames,wire,identity):
        need(6<=len(frames)<=13 and frames[3][0]=='send' and all(f[0]=='status' for f in frames[4:-1]) and frames[-1][0]=='receipt','replay_success_lifecycle');signature=self.c.b58encode(bytes.fromhex(identity['signatureHex']));_,qs,vs=frames[3];need(qs==[{'jsonrpc':'2.0','id':1,'method':'sendTransaction','params':[wire,{'encoding':'base64','skipPreflight':False,'maxRetries':0,'preflightCommitment':'finalized','minContextSlot':self.last_slot}]}] and vs==[signature],'replay_single_send');final=None
        for n,(_,qs,vs) in enumerate(frames[4:-1]):
            need(qs==[{'jsonrpc':'2.0','id':1,'method':'getGenesisHash','params':[]},{'jsonrpc':'2.0','id':2,'method':'getSignatureStatuses','params':[[signature],{'searchTransactionHistory':True}]}] and vs[0]==base.GENESIS and type(vs[1]['value']) is list and len(vs[1]['value'])==1 and self.c.u64(vs[1]['context']['slot'])>=self.last_slot,'replay_status');status=vs[1]['value'][0]
            if status is None:continue
            need(status['err'] is None and self.c.u64(status['slot'])>=self.last_slot and self.c.u64(vs[1]['context']['slot'])>=status['slot'],'replay_status_context')
            if status['confirmationStatus'] in ('processed','confirmed'):continue
            need(status['confirmationStatus']=='finalized' and n==len(frames)-6,'replay_final_status');final=status
        need(final is not None,'replay_final_status_missing');minimum=max(final['slot'],self.last_slot);_,qs,vs=frames[-1];need(qs==[{'jsonrpc':'2.0','id':1,'method':'getGenesisHash','params':[]},{'jsonrpc':'2.0','id':2,'method':'getTransaction','params':[signature,{'encoding':'base64','commitment':'finalized','maxSupportedTransactionVersion':0}]},{'jsonrpc':'2.0','id':3,'method':'getMultipleAccounts','params':[sorted({base.SYSTEM}|{self.roles[k] for k in base.NAMES}),{'encoding':'base64','commitment':'finalized','minContextSlot':minimum}]}] and vs[0]==base.GENESIS,'replay_receipt_request');tx=vs[1];poststate=vs[2];need(type(tx) is dict and tx['slot']==final['slot'] and tx['version']=='legacy' and tx['transaction']==[wire,'base64'],'replay_receipt_original');meta=tx['meta'];addresses=[a['address'] for a in self.bound['reviewed']['accountRoles']];pre=[int(self.pre_accounts[k]['lamports']) if self.pre_accounts[k]['exists'] else 0 for k in addresses];post=pre.copy();post[addresses.index(self.roles['g0'])]-=FEE;need(meta['err'] is None and meta['fee']==FEE and meta['preBalances']==pre and meta['postBalances']==post and meta.get('innerInstructions',[]) in (None,[]) and meta.get('rewards',[])==[] and meta.get('preTokenBalances',[])==[] and meta.get('postTokenBalances',[])==[] and meta.get('loadedAddresses',{'readonly':[],'writable':[]})=={'readonly':[],'writable':[]},'replay_exact_receipt');self.snapshot(poststate,self.index+1,minimum);return {'phase':self.index,'offset':extent(self.index)[0],'endOffset':sum(extent(self.index)),'signature':signature,'slot':tx['slot'],'feeLamports':FEE,'wireSha256':identity['wireSha256'],'poststateSha256':sha(encode(poststate))}
    def semantic_replay(self):
        self.store.audit(True);crypto_packets=[];by_attempt={}
        for attempt in sorted(self.store.attempts):
            if not attempt:continue
            rows=[r['kind'] for r in self.store.rows if r['data'].get('phase')==attempt and r['kind'] not in ('BEGIN','FILE')];need(rows in (['PREPARED','SIGN_ATTEMPT','SEND_INTENT','FINALIZED'],['PREPARED','SIGN_ATTEMPT','SEND_INTENT','UNSENT_LIFETIME_REFUSAL'],['PREPARED','SIGN_ATTEMPT','SEND_INTENT','UNSENT_LIFETIME_REFUSAL','RETIRED_UNSENT']),'replay_attempt_order');_,packet=self._record_value(attempt,'PREPARED');_,intent=self._record_value(attempt,'SEND_INTENT');by_attempt[attempt]=(packet,intent,rows);crypto_packets.append({'signedWireBase64':intent['wireBase64'],'reviewed':packet['reviewed']})
        identities=self._verify_many_public(crypto_packets) if crypto_packets else [];crypto=dict(zip(sorted(by_attempt),identities));self.index=self.first_phase
        for attempt,meta in sorted(self.store.attempts.items()):
            need(meta['logicalPhase']==self.index,'replay_logical_cursor');frames=self._frames(attempt)
            if attempt==0:
                expected={'logicalPhase':RETIRE_FIRST,'offset':RETIRE_START,'priorManifestSha256':self.p['priorHistorySha256'],'oldWireSha256':self.prior.old_intent['identity']['wireSha256'],'expiry':self._replay_expiry(frames,self.prior.old_intent,True),'oldPhase72ReserveLamports':5000,'oldAuReserveLamports':5000,'countsAgainstNewRetirementCap':False,'oldScopeAdopted':False};need(self.store.bridge==expected,'replay_prior_bridge');self.rpc_count+=len(frames);continue
            packet,intent,rows=by_attempt[attempt];need([f[0] for f in frames[:3]]==['original','before_sign','before_send'],'replay_attempt_frames');self.bound=self._expected_prepared(self.index,packet);life=packet['lifetime'];_,qs,vs=frames[0];config={'commitment':'finalized','minContextSlot':self.last_slot};need(qs==[{'jsonrpc':'2.0','id':1,'method':'getGenesisHash','params':[]},{'jsonrpc':'2.0','id':2,'method':'getLatestBlockhash','params':[config]},{'jsonrpc':'2.0','id':3,'method':'getBlockHeight','params':[config]}] and vs[0]==base.GENESIS and self.c.u64(vs[1]['context']['slot'])>=self.last_slot,'replay_original');original=vs[1];self.last_slot=original['context']['slot'];need(life=={'recentBlockhash':original['value']['blockhash'],'lastValidBlockHeight':str(original['value']['lastValidBlockHeight']),'currentBlockHeight':str(self.c.u64(vs[2])),'minimumRemainingBlockHeights':'40'},'replay_lifetime')
            prepared,_=self._record_value(attempt,'PREPARED');digest=sha(encode(prepared));binding={'id':packet['id'],'preparedRecordDigest':digest,'preparedSha256':sha(encode(packet)),'unsignedWireSha256':sha(base64.b64decode(packet['wireBase64'])),'messageSha256':packet['reviewed']['messageSha256'],'requiredSigners':packet['reviewed']['requiredSigners']};body={'schema':'piv1-startup-signing-request-v1','cluster':'testnet','genesisHash':base.GENESIS,'operatorTail':digest,'operatorSourceSha256':self.auth['operatorSourceSha256'],'authorizationSha256':self.authorization_sha,'baseParentPath':self.p['baseParentPath'],'baseParentTail':self.auth['baseParentTail'],'manifestSha256':self.package_sha,'sealSha256':None,'packets':[binding]};_,sign=self._record_value(attempt,'SIGN_ATTEMPT');need(sign['request']=={**body,'requestId':sha(encode(body))},'replay_sign_request');identity=crypto[attempt];need(intent=={'operation':self.index,'wireBase64':intent['wireBase64'],'identity':identity,'lifetime':life,'feeReservationLamports':FEE,'sendPolicy':{'encoding':'base64','skipPreflight':False,'maxRetries':0,'preflightCommitment':'finalized','minContextSlotRule':'MAX_FINAL_FRESH_CONTEXTS'}},'replay_intent');folder=self.store.path/f'phase-{attempt:04}';need(parse(read(folder/f'native-{self.index}-complete.json'))=={'complete':True} and parse(read(folder/f'native-{self.index}.json'))=={'schema':'piv1-startup-signing-response-v1','requestId':sign['request']['requestId'],'packets':[{'id':packet['id'],'wireBase64':intent['wireBase64']}]},'replay_native')
            self._replay_fresh(frames[1],self.bound,life);before_native=self.native;before_send=self.sent;self.native+=1;self.native_reserved+=1;self.reserved+=FEE
            if rows[-1]=='FINALIZED':
                self._replay_fresh(frames[2],self.bound,life);result=self._replay_success(frames,intent['wireBase64'],identity);need(self.store.finalized[attempt]==result,'replay_finalized');self.sent+=1;self.fees+=FEE;self.observed_fees+=FEE;self.index+=1
            else:
                refusal=self._replay_fresh(frames[2],self.bound,life,True);_,candidate=self._record_value(attempt,'UNSENT_LIFETIME_REFUSAL');expected={'logicalPhase':self.index,'wireSha256':identity['wireSha256'],'signature':self.c.b58encode(bytes.fromhex(identity['signatureHex'])),'refusal':refusal,'nativeCalls':self.native,'nativeReservations':self.native_reserved,'sendCalls':self.sent,'reservedFeeLamports':self.reserved,'sendCallsBeforeAttempt':before_send,'nativeCallsBeforeAttempt':before_native,'sourceSha256':self.p['sourcePins']['tools/startup_buffer_upload.py'],'normalReturn':'BEFORE_SEND_ORIGINAL_LIFETIME_ONLY','requiredNextAction':'TRUSTED_NORMAL_PROCESS_EXIT_THEN_READONLY_EXPIRY'};need(candidate==expected and not any('-send.' in n or '.partial.' in n for n in self.store.phase_names[attempt]),'replay_unsent_refusal');receipt=self._validate_exit(attempt)
                if rows[-1]=='RETIRED_UNSENT':
                    expired=self._replay_expiry(frames[3:],intent);need(self.store.retired[attempt]=={'attemptId':attempt,'logicalPhase':self.index,'wireSha256':identity['wireSha256'],'normalExitReceiptSha256':sha(encode(receipt)),'expiry':expired,'reservationLamports':FEE,'feeReleased':False,'oldPacketPermanentlyForbidden':True,'nextOffset':extent(self.index)[0]},'replay_retired');self.retirements+=1
                else:need(attempt==self.store.pending and len(frames)==3,'replay_pending_refusal')
            self.rpc_count+=len(frames)
        need(self.index==self.store.cursor,'replay_store_cursor');self.authorized();self.guard()


class FreshnessStore(RetiringStore):
    """Distinct D078 namespace; no consumed store is ever opened."""
    first_phase=76
    native_cap=928


class Prior76Bridge:
    """Selected unsent76 proof, not a certificate for old completed uploads."""
    def __init__(self,path,want):
        self.path=Path(path);self.raw=read(path,2*1024*1024);need(sha(self.raw)==want,'prior76_manifest_pin');self.m=parse(self.raw);need(self.m['schema']=='piv1-prior76-bridge-v1','prior76_schema');self.audit(True);self.derive()
    def audit(self,full=False):
        m=self.m;need(read(self.path,2*1024*1024)==self.raw,'prior76_manifest_changed')
        for path,d in m['directories'].items():
            s=Path(path).lstat();need(stat.S_ISDIR(s.st_mode) and {'dev':s.st_dev,'ino':s.st_ino,'uid':s.st_uid,'mode':stat.S_IMODE(s.st_mode),'names':sorted(os.listdir(path))}==d,'prior76_namespace')
        for path,d in m['files'].items():
            raw=read(path);need(len(raw)==d['bytes'] and sha(raw)==d['sha256'],'prior76_fresh_input')
        old=parse(read(m['packagePath']))
        need(set(m['sourceMap'])==set(old['sourcePins']),'prior76_source_coverage')
        required={m[k] for k in ('journalPath','anchorPath','packagePath','grantPath','rightsPath','oncePath','activePath','attestorPath','workerFailurePath','parentFailurePath','resourcesPath','holdPath','parentExitPath','parentStdoutPath','parentStderrPath','workerStdoutPath','workerStderrPath')}
        for name,d in m['sourceMap'].items():
            expected=self.path.parent/('baseline-'+Path(name).name) if name in ('tools/startup_buffer_upload.py','tools/test_startup_buffer_upload.py') else base.REPO/name
            need(Path(d['frozenPath'])==expected and d['sha256']==old['sourcePins'][name] and sha(read(d['frozenPath']))==d['sha256'],'prior76_source_binding');required.add(d['frozenPath'])
        need(required<=set(m['files']) and m['phasePath'] in m['directories'] and str(Path(m['phasePath']).parent) in m['directories'],'prior76_required_fresh_inputs')
        need(len(m['oldPhase72Files'])==14 and len(m['fundingFiles'])==6 and all(p in m['files'] for p in m['oldPhase72Files']+m['fundingFiles']) and m['oldAuUnknownSha256']=='6e25b64791a97a3a84a625afca4f899ff77c89303f58ffebc0653922d02be270','prior76_unresolved_coverage')
    def derive(self):
        m=self.m;jr=read(m['journalPath']);ar=read(m['anchorPath']);need(jr.endswith(b'\n') and ar.endswith(b'\n'),'prior76_partial_chain');rows=[];tail=None;selected=[];files={};anchors=ar.splitlines(keepends=True);raws=jr.splitlines(keepends=True);need(len(raws)==len(anchors)==142,'prior76_complete_chain')
        for n,(raw,anchor) in enumerate(zip(raws,anchors)):
            row=parse(raw);need(raw==encode(row) and row['sequence']==n and row['previous']==tail and parse(anchor)=={'sequence':n,'digest':sha(raw),'previous':tail},'prior76_raw_chain');tail=sha(raw);rows.append(row);d=row['data']
            if d.get('phase')==5:
                if row['kind']=='FILE':
                    path=str(Path(m['phasePath'])/d['name']);need(path in m['files'] and m['files'][path]=={'sha256':d['sha256'],'bytes':d['bytes']},'prior76_file_reference');files[d['name']]=path
                elif row['kind']!='BEGIN':selected.append(row)
        need(rows[0]['kind']=='SCOPE' and rows[0]['data']['packageSha256']==sha(read(m['packagePath'])) and rows[0]['data']['authorizationSha256']==sha(read(m['grantPath'])),'prior76_scope_binding')
        halt=rows[-1];need(halt['kind']=='HALT' and halt['data']=={'status':'HOLD_NO_RETRY','reason':'fresh_sample_aged_hold','phase':76,'sendCalls':4,'nativeCalls':5,'nativeReservations':5,'confirmedFeeLamports':20000,'admittedFeeLamports':20000,'reservedFeeLamports':25000,'retirements':0},'prior76_normal_refusal')
        need([r['kind'] for r in selected]==['PREPARED','SIGN_ATTEMPT','SEND_INTENT'] and len(files)==14 and set(files)==set(os.listdir(m['phasePath'])) and not any('-send.' in n or '.partial.' in n for n in files),'prior76_no_send_completeness');values={r['kind']:parse(read(files[r['data']['file']]))['data'] for r in selected};self.old_intent=values['SEND_INTENT'];self.packet=values['PREPARED'];self.signing=values['SIGN_ATTEMPT'];self.prepared_row=selected[0];self.files=files
        need(parse(read(files['native-76-complete.json']))=={'complete':True} and parse(read(files['native-76.json']))=={'schema':'piv1-startup-signing-response-v1','requestId':self.signing['request']['requestId'],'packets':[{'id':self.packet['id'],'wireBase64':self.old_intent['wireBase64']}]},'prior76_complete_native')
        exit=parse(read(m['parentExitPath']));worker=parse(read(m['workerFailurePath']));parent=parse(read(m['parentFailurePath']));resources=parse(read(m['resourcesPath']));hold=parse(read(m['holdPath']))
        need(exit['exitCode']==1 and exit['outerTimeout'] is False and exit['automaticRetry'] is False and worker=={'reason':'fresh_sample_aged_hold','scopeConsumed':True,'status':'HOLD_NO_RETRY'} and parent=={'reason':'worker_nonzero_exit','scopeConsumed':True,'status':'HOLD_NO_RETRY'} and resources['workerInvocations']==1 and resources['minimumFreeBytes']>=2*1024**3 and resources['peakRssSwapBytes']<=2684354560 and hold['retryAllowed'] is False,'prior76_owned_normal_exit')
        need(sha(read(m['parentStdoutPath']))==exit['stdoutSha256'] and sha(read(m['parentStderrPath']))==exit['stderrSha256'] and read(m['workerStderrPath'])==b'' and parse(read(m['workerStdoutPath']))=={'status':'HOLD_NO_RETRY','scopeConsumed':True},'prior76_exit_raw_binding')
        rights=parse(read(m['rightsPath']));grant=parse(read(m['grantPath']));once=parse(read(m['oncePath']));oldp=parse(read(m['packagePath']));need(rights['packageSha256']==grant['manifestSha256']==once['packageSha256']==sha(read(m['packagePath'])) and rights['grantSha256']==once['grantSha256']==sha(read(m['grantPath'])) and once['consumed'] is True and rights['activeSha256']==sha(read(m['activePath'])) and oldp['exitAttestorSha256']==sha(read(m['attestorPath'])),'prior76_consumed_authority_binding')
    def validate_semantics(self,image,ref,c):
        m=self.m;oldp=parse(read(m['packagePath']));auth=parse(read(m['grantPath']));view=RetiringUploadExecution.__new__(RetiringUploadExecution);view.p=oldp;view.roles=ref['roles'];view.ref=ref;view.image=image;view.c=c;view.index=76;view.fees=20000;view.last_clock=None
        frames=[]
        for n,label in enumerate(('original','before_sign','before_send'),1):
            prefix=f'rpc-{n:02}-{label}';qs=parse(read(self.files[prefix+'.request.json']));http=parse(read(self.files[prefix+'.http.json']));vs=UploadExecution.rpc_values(http['status'],read(self.files[prefix+'.response.json']),len(qs));frames.append((label,qs,vs))
        _,qs,vs=frames[0];config=qs[1]['params'][0];need(set(config)=={'commitment','minContextSlot'} and config['commitment']=='finalized' and type(config['minContextSlot']) is int and config['minContextSlot']>=oldp['initialContextSlot'],'prior76_original_floor');view.last_slot=config['minContextSlot'];need(qs==[{'jsonrpc':'2.0','id':1,'method':'getGenesisHash','params':[]},{'jsonrpc':'2.0','id':2,'method':'getLatestBlockhash','params':[config]},{'jsonrpc':'2.0','id':3,'method':'getBlockHeight','params':[config]}] and vs[0]==base.GENESIS and c.u64(vs[1]['context']['slot'])>=view.last_slot,'prior76_original');view.last_slot=vs[1]['context']['slot'];life=self.packet['lifetime'];need(life=={'recentBlockhash':vs[1]['value']['blockhash'],'lastValidBlockHeight':str(vs[1]['value']['lastValidBlockHeight']),'currentBlockHeight':str(c.u64(vs[2])),'minimumRemainingBlockHeights':'40'},'prior76_lifetime');bound=view._expected_prepared(76,self.packet)
        digest=sha(encode(self.prepared_row));binding={'id':self.packet['id'],'preparedRecordDigest':digest,'preparedSha256':sha(encode(self.packet)),'unsignedWireSha256':sha(base64.b64decode(self.packet['wireBase64'])),'messageSha256':self.packet['reviewed']['messageSha256'],'requiredSigners':self.packet['reviewed']['requiredSigners']};body={'schema':'piv1-startup-signing-request-v1','cluster':'testnet','genesisHash':base.GENESIS,'operatorTail':digest,'operatorSourceSha256':auth['operatorSourceSha256'],'authorizationSha256':sha(read(m['grantPath'])),'baseParentPath':oldp['baseParentPath'],'baseParentTail':auth['baseParentTail'],'manifestSha256':sha(read(m['packagePath'])),'sealSha256':None,'packets':[binding]};need(self.signing['request']=={**body,'requestId':sha(encode(body))},'prior76_sign_request')
        view._replay_fresh(frames[1],bound,life);view._replay_fresh(frames[2],bound,life);wire=self.old_intent['wireBase64'];self.audit();worker=base.Sdk(read(base.REPO/'tools/startup_two_operation.cjs'),1,view.template(76))
        try:
            self.audit();worker.call({'action':'BIND','blockhash':life['recentBlockhash']});self.audit();identity=worker.call({'action':'VERIFY','wireBase64':wire});self.audit()
        finally:worker.close()
        need(self.old_intent=={'operation':76,'wireBase64':wire,'identity':identity,'lifetime':life,'feeReservationLamports':5000,'sendPolicy':{'encoding':'base64','skipPreflight':False,'maxRetries':0,'preflightCommitment':'finalized','minContextSlotRule':'MAX_FINAL_FRESH_CONTEXTS'}},'prior76_exact_original_intent')


class FreshnessUploadExecution(RetiringUploadExecution):
    """D078 same-original refresh, strict age and literal exited-unsent proof."""
    first_phase=76
    start_offset=71700
    native_cap=928
    send_cap=859
    retire_cap=69
    send_fee_cap=4295000
    reservation_cap=4640000
    history_type=Prior76Bridge
    store_type=FreshnessStore
    def __init__(self,package_path,package_sha,authorization_path,authorization_sha,profile_path,evidence_path):
        absolute_deadline=time.monotonic()+max(0,1793331434-time.time());super().__init__(package_path,package_sha,authorization_path,authorization_sha,profile_path,evidence_path);self.deadline=min(self.deadline,absolute_deadline)
    def eligible_stop(self,exc):return str(exc) in ('original_lifetime','fresh_sample_aged_hold')
    def audit_static(self):
        p=self.p;need(p['schema']=='piv1-dedicated-buffer-upload-freshness-v1' and p['cluster']=='testnet' and p['genesisHash']==base.GENESIS and p['authority']=='D-074' and p['securityPolicies']==['D-075','D-077','D-078'] and p['firstPhase']==76 and p['lastPhase']==934 and p['startOffset']==71700 and p['endOffset']==END and p['nativeCap']==928 and p['sendCap']==859 and p['retirementCap']==69 and p['feeCapLamports']==4295000 and p['reservationCapLamports']==4640000 and p['perOperationFeeCapLamports']==5000 and p['rentLamports']==0 and p['oldSeparateReservesLamports']==15000 and p['expiresUnix']==1793331434 and p['maxAdditionalFreshBatches']==2 and 0<p['wallSeconds']<=28800,'freshness_package_scope')
        need(set(base.REQUIRED_SOURCES)|{'tools/startup_buffer_upload.py','tools/startup_public_journal.py'}<=set(p['sourcePins']),'incomplete_source_pins');self.source_audit();need(p['roles']=={k:self.roles[k] for k in base.NAMES} and p['initialG0Lamports']==999540000 and p['initialH2Lamports']==49204880 and p['initialG0Lamports']-self.reservation_cap-15000==994885000,'freshness_roles_budget');need(p['baseParentPath'] in p['historyDirectories'] and p['authorityBindingPath'] in p['historyPins'],'freshness_parent_binding');self.authorized();self.prior.audit(True);self.store.audit(True)
        need(type(p['initialContextSlot']) is int and set(p['initialClock'])=={'slot','unixTimestamp'} and all(type(v) is int for v in p['initialClock'].values()) and 0<=p['initialClock']['slot']<=p['initialContextSlot'],'initial_clock_floor')
    def resources(self):
        super().resources();need(self.p['priorScopeEvidenceReserveBytes']>=self.prior.m['retainedEvidenceBytes'] and self.p['externalEvidenceReserveBytes']==268435456 and len(self.store.journal_raw)+len(self.store.anchor_raw)+self.store.total_bytes<=8*1024**3-self.p['externalEvidenceReserveBytes']-self.p['priorScopeEvidenceReserveBytes'],'aggregate_evidence_budget')
    def authorized(self):
        super().authorized();need(int(self.auth['expiresUnixTimestamp'])<=1793331434 and self.p['expiresUnix']==1793331434,'absolute_authority_expiry')
    def sign(self,packet):
        self.extra_fresh_batches=0;return super().sign(packet)
    def fresh(self,label,bound,life):
        need(label in ('before_sign','before_send'),'freshness_stage');self.fresh_stage=label;self.fresh_bound=bound;self.fresh_life=life
        need(life['recentBlockhash'] not in {self.prior.old_intent['lifetime']['recentBlockhash']}|{self._record_value(attempt,'SEND_INTENT')[1]['lifetime']['recentBlockhash'] for attempt in self.store.retired},'retired_original_hash_forbidden')
        try:return super().fresh(label,bound,life)
        except Stop as exc:
            if str(exc)=='original_lifetime' and self.refusal is not None:self.refusal={**self.refusal,'reason':'original_lifetime','sampleAgeSeconds':None}
            raise
    def fresh_age(self):
        while True:
            try:return super().fresh_age()
            except Stop as exc:
                if str(exc)!='fresh_sample_aged_hold':raise
                age=time.monotonic()-self.current['responseAtMonotonic'];need(age>1,'age_refusal_measurement')
                if self.extra_fresh_batches>=2:
                    if self.fresh_stage=='before_send':self.refusal={'stage':'before_send','height':self.current['height'],'valid':True,'lastValidBlockHeight':int(self.fresh_life['lastValidBlockHeight']),'contextFloor':self.last_slot,'reason':'fresh_sample_aged_hold','sampleAgeSeconds':age}
                    raise
                self.extra_fresh_batches+=1;self.store.write(f'fresh-retry-{self.extra_fresh_batches}.json',encode({'logicalPhase':self.index,'stage':self.fresh_stage,'additionalBatch':self.extra_fresh_batches,'priorRpcOrdinal':self.rpc_count-self.phase_rpc_start,'sampleAgeSeconds':age,'messageSha256':self.fresh_bound['reviewed']['messageSha256'],'recentBlockhash':self.fresh_life['recentBlockhash']}));self.fresh(self.fresh_stage,self.fresh_bound,self.fresh_life)
    def _final_fresh_and_send(self,wire,life):
        self.fresh('before_send',self.bound,life);need(self.sent<self.send_cap and self.native<=self.native_cap and self.native==self.native_reserved,'at_most_once_budget');self.store.current_record(self.intent_name)
        while True:
            self.fresh_age();raw=encode([{'jsonrpc':'2.0','id':1,'method':'sendTransaction','params':[wire,{'encoding':'base64','skipPreflight':False,'maxRetries':0,'preflightCommitment':'finalized','minContextSlot':self.last_slot}]}])
            try:super().fresh_age();break
            except Stop as exc:
                if str(exc)!='fresh_sample_aged_hold':raise
                self.fresh_age()
        need(self.rpc_count<self.native_cap*15+8 and self.rpc_count-self.phase_rpc_start<15,'rpc_budget');self.rpc_count+=1;prefix=f'rpc-{self.rpc_count-self.phase_rpc_start:02}-send';self.sent+=1
        try:status,headers,body=self.transport_leaf.public_rpc_transport(self.c,raw);self.last_rpc_response_at=time.monotonic()
        except BaseException as exc:
            partial=getattr(exc,'partial',None)
            self.store.capture(lambda:(self.store.write(prefix+'.request.json',raw),self.store.write(prefix+'.partial.bin',partial)) if partial else self.store.write(prefix+'.request.json',raw));raise
        self.store.capture(lambda:(self.store.write(prefix+'.request.json',raw),self.store.write(prefix+'.response.json',body),self.store.write(prefix+'.http.json',encode({'status':status,'headers':headers}))));ack,=self.rpc_values(status,body,1);return ack
    def _normal_refusal(self,packet,wire,identity):
        need(self.refusal is not None and self.refusal['stage']=='before_send' and self.native==self.native_reserved and self.retirements<self.retire_cap,'unsent_refusal_eligibility');attempt=self.store.phase;rows=[r['kind'] for r in self.store.rows if r['data'].get('phase')==attempt and r['kind'] not in ('BEGIN','FILE')];need(rows==['PREPARED','SIGN_ATTEMPT','SEND_INTENT'] and not any('-send.' in n or '.partial.' in n for n in self.store.phase_names[attempt]),'unsent_no_send_inventory')
        need(parse(read(self.store.path/f'phase-{attempt:04}'/f'native-{self.index}-complete.json'))=={'complete':True},'unsent_native_incomplete');native=parse(read(self.store.path/f'phase-{attempt:04}'/f'native-{self.index}.json'));_,signing=self._record_value(attempt,'SIGN_ATTEMPT');need(native=={'schema':'piv1-startup-signing-response-v1','requestId':signing['request']['requestId'],'packets':[{'id':packet['id'],'wireBase64':wire}]},'unsent_native_binding');checked=self.sdk(lambda:self.worker.call({'action':'VERIFY','wireBase64':wire}));need(checked==identity,'unsent_crypto_binding');self.guard()
        proof={'logicalPhase':self.index,'wireSha256':identity['wireSha256'],'signature':self.c.b58encode(bytes.fromhex(identity['signatureHex'])),'refusal':self.refusal,'nativeCalls':self.native,'nativeReservations':self.native_reserved,'sendCalls':self.sent,'reservedFeeLamports':self.reserved,'sendCallsBeforeAttempt':self.attempt_send_start,'nativeCallsBeforeAttempt':self.attempt_native_start,'sourceSha256':self.p['sourcePins']['tools/startup_buffer_upload.py'],'normalReturn':'BEFORE_SEND_ADMISSION_ONLY','requiredNextAction':'TRUSTED_NORMAL_PROCESS_EXIT_THEN_READONLY_EXPIRY'};need(proof['sendCalls']==proof['sendCallsBeforeAttempt'] and proof['nativeCalls']==proof['nativeCallsBeforeAttempt']+1,'unsent_callback_counts');self.store.record('UNSENT_LIFETIME_REFUSAL',proof);self.store.audit(True);return self._candidate_outcome(attempt)
    def _bridge_expected(self,frames):
        return {'logicalPhase':76,'offset':71700,'priorManifestSha256':self.p['priorHistorySha256'],'oldWireSha256':self.prior.old_intent['identity']['wireSha256'],'expiry':self._replay_expiry(frames,self.prior.old_intent,True),'oldPhase76ReserveLamports':5000,'oldPhase72ReserveLamports':5000,'oldAuReserveLamports':5000,'old76CountsAgainstOriginalUnsentCap':True,'countsAgainstNewRetirementCap':False,'oldScopeAdopted':False}
    def _initial_bridge(self):
        self.store.begin(self.index,bridge=True);self.phase_rpc_start=self.rpc_count;self.phase_deadline=time.monotonic()+120;observation=self._expiry_observe(self.prior.old_intent,True);self.prior.audit(True);self.store.disposition('PRIOR_BRIDGE',{'logicalPhase':76,'offset':71700,'priorManifestSha256':self.p['priorHistorySha256'],'oldWireSha256':self.prior.old_intent['identity']['wireSha256'],'expiry':observation,'oldPhase76ReserveLamports':5000,'oldPhase72ReserveLamports':5000,'oldAuReserveLamports':5000,'old76CountsAgainstOriginalUnsentCap':True,'countsAgainstNewRetirementCap':False,'oldScopeAdopted':False})
    def run(self):
        outcome=super().run()
        if outcome['status']=='FULL_BUFFER_VERIFIED':outcome.update(oldPhase76ReserveLamports=5000,oldSeparateReservesLamports=15000)
        return outcome
    def _fresh_groups(self,frames,attempt):
        need(frames and frames[0][0]=='original','replay_original_order');position=1;groups=[];previous=[]
        for stage in ('before_sign','before_send'):
            group=[]
            while position<len(frames) and frames[position][0]==stage:
                if group:previous.append((stage,position,frames[position-1]))
                group.append(frames[position]);position+=1
            need(group,'replay_missing_fresh');groups.append(group)
        need(len(previous)<=2,'replay_shared_refresh_budget');folder=self.store.path/f'phase-{attempt:04}';expected_names={f'fresh-retry-{n}.json' for n in range(1,len(previous)+1)};need({n for n in self.store.phase_names[attempt] if n.startswith('fresh-retry-')}==expected_names,'replay_refresh_inventory')
        for n,(stage,ordinal,_) in enumerate(previous,1):
            trace=parse(read(folder/f'fresh-retry-{n}.json'));need(set(trace)=={'logicalPhase','stage','additionalBatch','priorRpcOrdinal','sampleAgeSeconds','messageSha256','recentBlockhash'} and trace['logicalPhase']==self.index and trace['stage']==stage and trace['additionalBatch']==n and trace['priorRpcOrdinal']==ordinal and trace['messageSha256']==self._record_value(attempt,'PREPARED')[1]['reviewed']['messageSha256'] and trace['recentBlockhash']==self._record_value(attempt,'PREPARED')[1]['lifetime']['recentBlockhash'] and type(trace['sampleAgeSeconds']) in (int,float) and 1<trace['sampleAgeSeconds']<120,'replay_refresh_trace')
        return groups,frames[position:]
    def _replay_refusal_group(self,group,bound,life,candidate,extra):
        proof=candidate['refusal'];need(set(proof)=={'stage','height','valid','lastValidBlockHeight','contextFloor','reason','sampleAgeSeconds'},'replay_refusal_fields')
        for frame in group[:-1]:self._replay_fresh(frame,bound,life)
        if proof['reason']=='original_lifetime':
            expected=self._replay_fresh(group[-1],bound,life,True);need(proof['sampleAgeSeconds'] is None,'replay_lifetime_age');return {**expected,'reason':'original_lifetime','sampleAgeSeconds':None}
        need(proof['reason']=='fresh_sample_aged_hold' and extra==2 and type(proof['sampleAgeSeconds']) in (int,float) and 1<proof['sampleAgeSeconds']<120,'replay_age_refusal');self._replay_fresh(group[-1],bound,life);return {'stage':'before_send','height':self.c.u64(group[-1][2][4]),'valid':True,'lastValidBlockHeight':int(life['lastValidBlockHeight']),'contextFloor':self.last_slot,'reason':'fresh_sample_aged_hold','sampleAgeSeconds':proof['sampleAgeSeconds']}

    def semantic_replay(self):
        self.store.audit(True);crypto_packets=[];by_attempt={}
        for attempt in sorted(self.store.attempts):
            if not attempt:continue
            rows=[r['kind'] for r in self.store.rows if r['data'].get('phase')==attempt and r['kind'] not in ('BEGIN','FILE')];need(rows in (['PREPARED','SIGN_ATTEMPT','SEND_INTENT','FINALIZED'],['PREPARED','SIGN_ATTEMPT','SEND_INTENT','UNSENT_LIFETIME_REFUSAL'],['PREPARED','SIGN_ATTEMPT','SEND_INTENT','UNSENT_LIFETIME_REFUSAL','RETIRED_UNSENT']),'replay_attempt_order');_,packet=self._record_value(attempt,'PREPARED');_,intent=self._record_value(attempt,'SEND_INTENT');by_attempt[attempt]=(packet,intent,rows);crypto_packets.append({'signedWireBase64':intent['wireBase64'],'reviewed':packet['reviewed']})
        identities=self._verify_many_public(crypto_packets) if crypto_packets else [];crypto=dict(zip(sorted(by_attempt),identities));self.index=self.first_phase
        for attempt,meta in sorted(self.store.attempts.items()):
            need(meta['logicalPhase']==self.index,'replay_logical_cursor');frames=self._frames(attempt)
            if attempt==0:
                expected=self._bridge_expected(frames);need(self.store.bridge==expected,'replay_prior_bridge');self.rpc_count+=len(frames);continue
            packet,intent,rows=by_attempt[attempt];groups,tail=self._fresh_groups(frames,attempt);self.bound=self._expected_prepared(self.index,packet);life=packet['lifetime'];need(life['recentBlockhash'] not in {self.prior.old_intent['lifetime']['recentBlockhash']}|{self._record_value(a,'SEND_INTENT')[1]['lifetime']['recentBlockhash'] for a in self.store.retired if a<attempt},'replay_retired_original_hash');_,qs,vs=frames[0];config={'commitment':'finalized','minContextSlot':self.last_slot};need(qs==[{'jsonrpc':'2.0','id':1,'method':'getGenesisHash','params':[]},{'jsonrpc':'2.0','id':2,'method':'getLatestBlockhash','params':[config]},{'jsonrpc':'2.0','id':3,'method':'getBlockHeight','params':[config]}] and vs[0]==base.GENESIS and self.c.u64(vs[1]['context']['slot'])>=self.last_slot,'replay_original');original=vs[1];self.last_slot=original['context']['slot'];need(life=={'recentBlockhash':original['value']['blockhash'],'lastValidBlockHeight':str(original['value']['lastValidBlockHeight']),'currentBlockHeight':str(self.c.u64(vs[2])),'minimumRemainingBlockHeights':'40'},'replay_lifetime')
            prepared,_=self._record_value(attempt,'PREPARED');digest=sha(encode(prepared));binding={'id':packet['id'],'preparedRecordDigest':digest,'preparedSha256':sha(encode(packet)),'unsignedWireSha256':sha(base64.b64decode(packet['wireBase64'])),'messageSha256':packet['reviewed']['messageSha256'],'requiredSigners':packet['reviewed']['requiredSigners']};body={'schema':'piv1-startup-signing-request-v1','cluster':'testnet','genesisHash':base.GENESIS,'operatorTail':digest,'operatorSourceSha256':self.auth['operatorSourceSha256'],'authorizationSha256':self.authorization_sha,'baseParentPath':self.p['baseParentPath'],'baseParentTail':self.auth['baseParentTail'],'manifestSha256':self.package_sha,'sealSha256':None,'packets':[binding]};_,sign=self._record_value(attempt,'SIGN_ATTEMPT');need(sign['request']=={**body,'requestId':sha(encode(body))},'replay_sign_request');identity=crypto[attempt];need(intent=={'operation':self.index,'wireBase64':intent['wireBase64'],'identity':identity,'lifetime':life,'feeReservationLamports':FEE,'sendPolicy':{'encoding':'base64','skipPreflight':False,'maxRetries':0,'preflightCommitment':'finalized','minContextSlotRule':'MAX_FINAL_FRESH_CONTEXTS'}},'replay_intent');folder=self.store.path/f'phase-{attempt:04}';need(parse(read(folder/f'native-{self.index}-complete.json'))=={'complete':True} and parse(read(folder/f'native-{self.index}.json'))=={'schema':'piv1-startup-signing-response-v1','requestId':sign['request']['requestId'],'packets':[{'id':packet['id'],'wireBase64':intent['wireBase64']}]},'replay_native')
            for frame in groups[0]:self._replay_fresh(frame,self.bound,life)
            before_native=self.native;before_send=self.sent;self.native+=1;self.native_reserved+=1;self.reserved+=FEE
            if rows[-1]=='FINALIZED':
                for frame in groups[1]:self._replay_fresh(frame,self.bound,life)
                result=self._replay_success([frames[0],groups[0][-1],groups[1][-1]]+tail,intent['wireBase64'],identity);need(self.store.finalized[attempt]==result,'replay_finalized');self.sent+=1;self.fees+=FEE;self.observed_fees+=FEE;self.index+=1
            else:
                _,candidate=self._record_value(attempt,'UNSENT_LIFETIME_REFUSAL');refusal=self._replay_refusal_group(groups[1],self.bound,life,candidate,len(groups[0])+len(groups[1])-2);expected={'logicalPhase':self.index,'wireSha256':identity['wireSha256'],'signature':self.c.b58encode(bytes.fromhex(identity['signatureHex'])),'refusal':refusal,'nativeCalls':self.native,'nativeReservations':self.native_reserved,'sendCalls':self.sent,'reservedFeeLamports':self.reserved,'sendCallsBeforeAttempt':before_send,'nativeCallsBeforeAttempt':before_native,'sourceSha256':self.p['sourcePins']['tools/startup_buffer_upload.py'],'normalReturn':'BEFORE_SEND_ADMISSION_ONLY','requiredNextAction':'TRUSTED_NORMAL_PROCESS_EXIT_THEN_READONLY_EXPIRY'};need(candidate==expected and not any('-send.' in n or '.partial.' in n for n in self.store.phase_names[attempt]),'replay_unsent_refusal');receipt=self._validate_exit(attempt)
                if rows[-1]=='RETIRED_UNSENT':
                    expired=self._replay_expiry(tail,intent);need(self.store.retired[attempt]=={'attemptId':attempt,'logicalPhase':self.index,'wireSha256':identity['wireSha256'],'normalExitReceiptSha256':sha(encode(receipt)),'expiry':expired,'reservationLamports':FEE,'feeReleased':False,'oldPacketPermanentlyForbidden':True,'nextOffset':extent(self.index)[0]},'replay_retired');self.retirements+=1
                else:need(attempt==self.store.pending and not tail,'replay_pending_refusal')
            self.rpc_count+=len(frames)
        need(self.index==self.store.cursor,'replay_store_cursor');self.authorized();self.guard()
