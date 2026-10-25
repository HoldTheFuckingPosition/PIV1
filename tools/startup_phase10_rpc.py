"""Phase10-only private-origin TLS and explicit same-slot five-plus-five reads.

The immutable operator persists actual requests/replies. Derived snapshots are
tagged, reconstructed from those bytes and never attributed to its collector.
No endpoint is read during import, binding preparation or retained replay.
"""
import base64
import hashlib
import http.client
import json
import os
from pathlib import Path
import re
import resource
import selectors
import signal
import ssl
import stat
import subprocess
import sys
import time
import urllib.parse

MODE = "QUICKNODE_TESTNET_SAME_SLOT_FIVE_PLUS_FIVE"
SCHEMA = "piv1-phase10-private-rpc-binding-v1"
CREDENTIAL = Path("/home/jerem/.piv1-rpc/quicknode-testnet.url")
BOUND = 16 * 1024**2
WORKER = r'''
import hashlib,os,resource,types
from pathlib import Path
resource.setrlimit(resource.RLIMIT_CORE,(0,0))
p=Path(os.environ['PIV1_PUBLIC_CHECK_REPO'])/'tools/startup_phase10_rpc.py'
raw=p.read_bytes() if not p.is_symlink() else b''
if hashlib.sha256(raw).hexdigest()!=os.environ['PIV1_PHASE10_RPC_SOURCE']:raise ValueError('route_source')
m=types.ModuleType('phase10_private_rpc_worker');m.__file__=str(p)
exec(compile(raw,str(p),'exec',dont_inherit=True),m.__dict__)
m.worker()
'''


class Stop(ValueError): pass


def need(ok, reason):
    if not ok: raise Stop(reason)


def encode(value): return (json.dumps(value,sort_keys=True,separators=(",",":"),allow_nan=False)+"\n").encode()
def sha(raw): return hashlib.sha256(raw).hexdigest()


def public(path, maximum=BOUND):
    fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
    try:
        before=os.fstat(fd)
        need(stat.S_ISREG(before.st_mode) and before.st_uid==os.getuid() and before.st_nlink==1 and 0<before.st_size<=maximum,"rpc_public_metadata")
        raw=b""
        while len(raw)<before.st_size:
            part=os.read(fd,min(65536,before.st_size-len(raw))); need(part,"rpc_public_short_read"); raw+=part
        identity=lambda s:(s.st_dev,s.st_ino,s.st_uid,s.st_nlink,s.st_mode,s.st_size,s.st_mtime_ns,s.st_ctime_ns)
        need(identity(before)==identity(os.fstat(fd))==identity(Path(path).lstat()),"rpc_public_changed")
        return raw
    finally: os.close(fd)


def binding(endpoint_sha256, source_sha256):
    need(all(type(x) is str and re.fullmatch("[0-9a-f]{64}",x) for x in (endpoint_sha256,source_sha256)),"rpc_binding_digest")
    return {"schema":SCHEMA,"mode":MODE,"sourceSha256":source_sha256,"endpointCredentialSha256":endpoint_sha256}


def check_binding(value):
    need(type(value) is dict and set(value)=={"schema","mode","sourceSha256","endpointCredentialSha256"} and
         value==binding(value["endpointCredentialSha256"],value["sourceSha256"]),"rpc_binding")
    path=Path(__file__)
    need(not path.is_symlink() and sha(public(path))==value["sourceSha256"],"rpc_source_pin")


def credential(expected):
    parent=CREDENTIAL.parent.lstat()
    need(stat.S_ISDIR(parent.st_mode) and parent.st_uid==os.getuid() and stat.S_IMODE(parent.st_mode)==0o700,"rpc_credential_parent")
    fd=os.open(CREDENTIAL,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
    try:
        before=os.fstat(fd)
        need(stat.S_ISREG(before.st_mode) and before.st_uid==os.getuid() and before.st_nlink==1 and
             stat.S_IMODE(before.st_mode)==0o600 and 0<before.st_size<=4096,"rpc_credential_metadata")
        raw=os.read(fd,4097)
        identity=lambda s:(s.st_dev,s.st_ino,s.st_uid,s.st_gid,s.st_nlink,s.st_mode,s.st_size,s.st_mtime_ns,s.st_ctime_ns)
        need(len(raw)==before.st_size and identity(before)==identity(os.fstat(fd))==identity(CREDENTIAL.lstat()) and sha(raw)==expected,"rpc_credential_changed")
    finally: os.close(fd)
    value=raw.decode("ascii").strip(); need(value and all(33<=ord(c)<=126 for c in value),"rpc_credential_ascii")
    url=urllib.parse.urlsplit(value)
    need(url.scheme=="https" and not url.username and not url.password and url.port in (None,443) and
         not url.query and not url.fragment and url.hostname is not None and
         re.fullmatch(r"[a-z0-9-]+\.solana-testnet\.quiknode\.pro",url.hostname) and
         re.fullmatch(r"/[A-Za-z0-9_-]{16,256}/?",url.path),"rpc_credential_shape")
    protected=(raw.strip(),value.encode(),url.hostname.encode(),url.path.encode(),url.path.strip("/").encode())
    return url,protected


def verify_credential(value):
    check_binding(value)
    try: credential(value["endpointCredentialSha256"])
    except Exception: raise Stop("rpc_credential_refused") from None


def worker():
    """Isolated private worker; no secret-bearing exceptions or output."""
    frame={"schema":"piv1-phase10-private-rpc-worker-v1","kind":"FAILURE"}; body=bytearray(); connection=None
    try:
        raw=sys.stdin.buffer.read(BOUND+1); need(0<len(raw)<=BOUND,"rpc_request_bound")
        url,protected=credential(os.environ["PIV1_PHASE10_ENDPOINT_SHA256"])
        connection=http.client.HTTPSConnection(url.hostname,443,timeout=30,context=ssl.create_default_context())
        connection.request("POST",url.path,body=raw,headers={"Content-Type":"application/json","Accept":"application/json","User-Agent":"piv1-phase10-private-rpc-v1"})
        response=connection.getresponse(); headers=list(response.getheaders()); need(len(encode(headers))<=16384,"rpc_headers_bound")
        while True:
            part=response.read(min(65536,BOUND+1-len(body)))
            if not part:
                need(response.length in (None,0),"rpc_incomplete_body"); break
            body.extend(part); need(len(body)<=BOUND,"rpc_body_bound")
        need(not any(x and (x in body or x in encode(headers)) for x in protected),"rpc_secret_echo")
        frame.update(kind="RESPONSE",status=response.status,headers=headers,bodyBase64=base64.b64encode(body).decode())
    except Exception as exc:
        if isinstance(exc,http.client.IncompleteRead): body.extend(exc.partial[:max(0,BOUND+1-len(body))])
        if body and "protected" in locals() and not any(x and x in body for x in protected):
            frame.update(kind="TRANSPORT_FAILURE",partialBase64=base64.b64encode(body).decode())
    finally:
        if connection is not None: connection.close()
    sys.stdout.buffer.write(encode(frame)); sys.stdout.buffer.flush()


def transport(core,c,value,raw):
    """One absolute30s accepted worker call; kill/reap before failure returns."""
    deadline=time.monotonic()+core.RPC_SECONDS; check_binding(value)
    need(not any(os.environ.get(k) for k in (*c.PROXIES,*core.RPC_TLS_ENV)),"rpc_environment")
    need(type(raw) is bytes and 0<len(raw)<=BOUND and c.MAX_JSON==BOUND,"rpc_request_bound")
    repo=Path(__file__).resolve().parent.parent; process=None; selector=selectors.DefaultSelector(); streams=(); output=bytearray(); accepted=False
    try:
        need(sha(public(repo/"tools/collect_startup_public_evidence.py"))==core.RPC_COLLECTOR_SHA,"rpc_original_collector_pin")
        need(time.monotonic()<deadline,"rpc_worker_deadline")
        process=subprocess.Popen([str(Path(sys.executable).resolve()),"-I","-S","-c",WORKER],cwd=repo,
            env={"PIV1_PUBLIC_CHECK_REPO":str(repo),"PIV1_PHASE10_RPC_SOURCE":value["sourceSha256"],"PIV1_PHASE10_ENDPOINT_SHA256":value["endpointCredentialSha256"]},
            stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,close_fds=True,start_new_session=True)
        streams=(process.stdin,process.stdout); position=0
        for stream in streams:
            os.set_blocking(stream.fileno(),False); selector.register(stream,selectors.EVENT_WRITE if stream is process.stdin else selectors.EVENT_READ)
        while selector.get_map():
            remaining=deadline-time.monotonic(); need(remaining>0,"rpc_worker_deadline")
            for event,_ in selector.select(min(.25,remaining)):
                need(time.monotonic()<deadline,"rpc_worker_deadline"); stream=event.fileobj
                if stream is process.stdin:
                    position+=os.write(stream.fileno(),raw[position:position+65536])
                    if position==len(raw): selector.unregister(stream); stream.close()
                else:
                    chunk=os.read(stream.fileno(),65536)
                    if not chunk: selector.unregister(stream)
                    else: output.extend(chunk); need(len(output)<=core.RPC_OUTPUT_BOUND,"rpc_worker_output_bound")
        frame=json.loads(bytes(output)); need(encode(frame)==bytes(output) and type(frame) is dict and frame.get("schema")=="piv1-phase10-private-rpc-worker-v1","rpc_worker_frame")
        if frame.get("kind")=="TRANSPORT_FAILURE":
            need(set(frame)=={"schema","kind","partialBase64"},"rpc_worker_partial_frame"); partial=c.b64(frame["partialBase64"],BOUND+1,True)
        else:
            need(set(frame)=={"schema","kind","status","headers","bodyBase64"} and frame["kind"]=="RESPONSE" and
                 type(frame["status"]) is int and 100<=frame["status"]<=599,"rpc_worker_response")
            headers=frame["headers"]
            need(type(headers) is list and len(encode(headers))<=16384 and all(type(p) is list and len(p)==2 and all(type(x) is str for x in p) for p in headers),"rpc_worker_headers")
            body=c.b64(frame["bodyBase64"],BOUND,True)
        remaining=deadline-time.monotonic(); need(remaining>0 and process.wait(timeout=remaining)==0,"rpc_worker_exit")
        check_binding(value)
        need(sha(public(repo/"tools/collect_startup_public_evidence.py"))==core.RPC_COLLECTOR_SHA,"rpc_original_collector_pin")
        need(time.monotonic()<deadline,"rpc_worker_deadline")
        if frame["kind"]=="TRANSPORT_FAILURE": raise c.TransportFailure("phase10_private_transport_failure",partial)
        accepted=True; return frame["status"],headers,body
    except c.TransportFailure: raise
    except Exception: raise c.TransportFailure("phase10_private_worker_refused") from None
    finally:
        try:
            if process is not None and not accepted:
                try:
                    if process.returncode is None:
                        try: os.killpg(process.pid,signal.SIGKILL)
                        except ProcessLookupError: pass
                        except OSError:
                            try: process.kill()
                            except OSError: pass
                finally: process.wait()
        finally:
            selector.close()
            for stream in streams:
                if not stream.closed: stream.close()


def expanded(calls,keys):
    need(len(keys)==len(set(keys))==10,"rpc_exact_ten_keys")
    actual=[]; split=None
    for method,params in calls:
        if method=="getMultipleAccounts":
            need(split is None and type(params) is list and len(params)==2 and params[0]==keys and
                 type(params[1]) is dict and set(params[1])=={"encoding","commitment","minContextSlot"} and
                 params[1]["encoding"]=="base64" and params[1]["commitment"]=="finalized" and
                 type(params[1]["minContextSlot"]) is int and 0<=params[1]["minContextSlot"]<2**64,"rpc_exact_split_request")
            split=len(actual)
            actual += [(method,[keys[:5],params[1]]),(method,[keys[5:],params[1]])]
        else: actual.append((method,params))
    need(1<=len(actual)<=5,"rpc_expanded_batch_bound")
    return actual,split


def derived(c,results,calls,keys,raw_sha,value):
    actual,index=expanded(calls,keys)
    need(len(results)==len(actual),"rpc_expanded_result_count")
    if index is None: return results
    parts=results[index:index+2]; minimum=actual[index][1][1]["minContextSlot"]
    need(all(type(p) is dict and type(p.get("context")) is dict and type(p.get("value")) is list and len(p["value"])==5 for p in parts),"rpc_exact_five_value_parts")
    slots=[c.u64(p["context"].get("slot")) for p in parts]
    need(slots[0]==slots[1] and slots[0]>=minimum,"rpc_split_same_slot")
    need(all(p["context"].get("apiVersion")=="4.4.0-beta.1" for p in parts),"rpc_split_api_version")
    snapshot={"schema":"DERIVED_SPLIT_SNAPSHOT","context":{"slot":slots[0],"apiVersion":"4.4.0-beta.1"},
              "value":parts[0]["value"]+parts[1]["value"],"nativeCollectorProvenance":False,
              "transportBinding":value,"sharedRawResponseSha256":raw_sha,
              "parts":[{"replyId":index+i+1,"keys":keys[i*5:(i+1)*5]} for i in range(2)]}
    return results[:index]+[snapshot]+results[index+2:]


def receipt_binding(value,mode):
    need(mode in (MODE,"OFFLINE_INJECTED_TEST"),"rpc_actual_transport_mode")
    return {"schema":"piv1-phase10-rpc-receipt-binding-v1","transportMode":mode,"binding":value}


class Rpc:
    def __init__(self,core,c,j,store,value,keys,mode,*,injected=None):
        check_binding(value); need((injected is None)==(mode==MODE),"rpc_injection_mode")
        self.core,self.c,self.store,self.value,self.keys,self.mode=core,c,store,value,keys,mode
        store.write("rpc-transport.json",encode(receipt_binding(value,mode)))
        self.rpc=core.Rpc(c,j,store,(lambda raw:transport(core,c,value,raw)) if injected is None else injected)

    def query(self,calls):
        if self.mode==MODE: verify_credential(self.value)
        actual,_=expanded(calls,self.keys); results=self.rpc.query(actual)
        if self.mode==MODE: verify_credential(self.value)
        raw=public(self.store.path/f"rpc-{self.rpc.count:02}.response.bin")
        return derived(self.c,results,calls,self.keys,sha(raw),self.value)


def retained(core,c,directory,index,calls,keys,value,mode,*,_parse_http=None):
    check_binding(value)
    need(core.encode(receipt_binding(value,mode))==public(Path(directory)/"rpc-transport.json"),"rpc_retained_origin_binding")
    actual,_=expanded(calls,keys); prefix=Path(directory)/f"rpc-{index:02}"
    queries=[{"jsonrpc":"2.0","id":i+1,"method":m,"params":p} for i,(m,p) in enumerate(actual)]
    need(public(Path(str(prefix)+".request.json"))==core.encode(queries),"rpc_retained_request")
    saved=public(Path(str(prefix)+".http.json"))
    # Only successor_replay supplies its entry-validated, one-replay parser.
    http=(_parse_http if _parse_http is not None else core.helpers()[3].parse)(saved)
    need(core.encode(http)==saved and http["status"]==200,"rpc_retained_http")
    raw=public(Path(str(prefix)+".response.bin")); replies=c.strict_json(raw)
    need(type(replies) is list and len(replies)==len(queries),"rpc_retained_count"); by={}
    for reply in replies:
        need(type(reply) is dict and reply.get("jsonrpc")=="2.0" and type(reply.get("id")) is int and
             1<=reply["id"]<=len(queries) and reply["id"] not in by and "result" in reply and "error" not in reply,"rpc_retained_reply")
        by[reply["id"]]=reply["result"]
    return derived(c,[by[i+1] for i in range(len(queries))],calls,keys,sha(raw),value)
