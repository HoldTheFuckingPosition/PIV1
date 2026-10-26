#!/usr/bin/env python3
"""Durable Testnet release workflow with a separately gated future signer.

Live actions require a launcher-pinned authorization receipt. This is a local
trusted-launcher boundary, not cryptographic verification of founder authority.
"""
import argparse
import base64
import copy
import fcntl
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import resource
import selectors
import signal
import stat
import subprocess
import sys
import time

SCHEMA = "piv1-startup-release-operator-v1"
CANDIDATE_CONTROLLER_SCHEMA = "piv1-candidate-offline-funding-controller-v1"
CANDIDATE_CONTROLLER_LOCK = b"piv1-candidate-offline-funding-controller-lock-v1\n"
CANDIDATE_PREFIX_COUNT = 935
CANDIDATE_PREFIX_RECORDS = 3+2*CANDIDATE_PREFIX_COUNT
CANDIDATE_STARTUP_COUNT = 947
CANDIDATE_STARTUP_RECORDS = 3+2*CANDIDATE_STARTUP_COUNT
ARTIFACT = {"bytes": 894552, "sha256": "e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99"}
READBACK_SHA = "fbc28f1bc5618459e1178bbea45f3db54fedd21363d7abdcfe4794ef0b0e9adb"
STATE_RULES_SHA = "c85e56752aa2cb56d7e909bc61cdabcc844a34f1dc6623015d612c4dbd071395"
MANIFEST_SHA = "a25bd657ee56fc1ad93a93b62ad58eb9104bc7d0435fc0637c2d46d429e78f5c"
SIGNER_BACKEND_SHA = "88138f40cc24e83c46474318f89f06713dbde84da45e1f119a786ceeaadf36e4"
RPC_COLLECTOR_SHA = "96309ac61a07dc23f204cd7c799b25ec67624af4d1103b9057b25728db85b7f6"
RPC_SECONDS = 30
RPC_BODY_BOUND = 16 * 1024 * 1024
RPC_OUTPUT_BOUND = 4 * ((RPC_BODY_BOUND + 3) // 3) + 32768
RPC_TLS_ENV = ("SSL_CERT_FILE", "SSL_CERT_DIR", "SSLKEYLOGFILE", "OPENSSL_CONF", "OPENSSL_MODULES", "OPENSSL_ENGINES")
# Two HTTP budgets plus five existing 20-second Node budgets and 20 seconds
# for local work. This is an acceptance cap, not a successful-cost guarantee.
COLLECTION_SECONDS = 180
COLLECTION_PIPE_BOUND = 65536
MAX_PHASES, MAX_BATCH, MAX_RECORDS = 949, 64, 8192
MAX_JOURNAL = 128 * 1024 * 1024
ACTION_EVIDENCE = 256 * 1024 * 1024
MAX_EVIDENCE = 8 * 1024 * 1024 * 1024
SUCCESS_SERIALIZATION = 1024 * 1024
LOCK = b"piv1-startup-release-operator-lock-v1\n"
KINDS = {"BUFFER_CREATE", "UPLOAD_WRITE", "BOOTSTRAP", "RECIPIENTS", "INITIALIZER", "HEARTBEAT"}
DESCRIBE_SCRIPT = r'''
const fs=require('node:fs'),c=require(process.env.PIV1_PUBLIC_CHECK_REPO+'/clients/cli/startup-public-check.cjs');
c.verifySources();const raw=fs.readFileSync(0);if(raw.length>1048576)throw Error('input_bound');
const input=JSON.parse(raw),packets=Array.isArray(input)?input:input.packets;
if(!Array.isArray(input)&&Object.hasOwn(input,'runtimeMetadata')&&(input.runtimeMetadata!==true||Object.keys(input).sort().join(',')!=='packets,runtimeMetadata'))throw Error('runtime_metadata_mode');
const out=packets.map(x=>{const d=c.decodeWire(x.wireBase64,x.lookupTables);
 if(!d.fullyResolved||d.signatures.some(s=>s.some(v=>v!==0)))throw Error('unsigned_resolved_required');
 const reviewed={messageBase64:d.message.toString('base64'),messageSha256:d.messageSha256,feePayer:d.feePayer,
 requiredSigners:d.requiredSigners,accountRoles:d.roles,lookupTables:x.lookupTables};
 return !Array.isArray(input)&&input.runtimeMetadata===true?{reviewed,instructions:d.instructions,lookups:d.lookups}:reviewed;});
let result=out;
if(!Array.isArray(input)&&input.runtimeMetadata!==true){
 const {PublicKey}=require(process.env.PIV1_PUBLIC_CHECK_REPO+'/spikes/task-0.4-jito/node_modules/@solana/web3.js/lib/index.cjs.js');
 const p=input.plan,program=new PublicKey(p.program),text=x=>Buffer.from(x),key=x=>new PublicKey(x).toBuffer();
 const derive=(seeds,id=program)=>{const [a,b]=PublicKey.findProgramAddressSync(seeds,id);return {address:a.toBase58(),bump:b}};
 const seeds=['config','distribution','guardian-registry'].map(x=>[text(x)]);
 for(let i=0;i<6;i++)seeds.push([text('guardian-reward'),key(p.guardianSlots[i]),Buffer.alloc(8),Buffer.from([i])]);
 for(const name of ['pending-sol','principal-sol','operational-sol','distribution-escrow','kif-sol','principal-jito-vault','pending-jito-vault'])seeds.push([text(name)]);
 const targets=seeds.map(s=>derive(s));if(targets.some((x,i)=>x.address!==p.alt.addresses[i]))throw Error('startup_target_pda');
 const authority=derive([text('authority')]);if(authority.address!==p.pivAuthority)throw Error('startup_authority_pda');
 const index=Buffer.alloc(8);index.writeBigUInt64LE(BigInt(input.governance.transactionIndex));
 const proposal=derive([text('multisig'),key(p.multisig),text('transaction'),index,text('proposal')],new PublicKey('SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf'));
 if(proposal.address!==p.proposal)throw Error('startup_proposal_pda');
 result={packets:out,targetBumps:targets.map(x=>x.bump),authorityBump:authority.bump,proposalBump:proposal.bump};
}
const text=JSON.stringify(result);if(Buffer.byteLength(text)>524288)throw Error('output_bound');process.stdout.write(text);
'''

RPC_TRANSPORT_SCRIPT = r'''
import base64,hashlib,json,os,resource,sys,types
from pathlib import Path
resource.setrlimit(resource.RLIMIT_CORE,(0,0))
path=Path(os.environ['PIV1_PUBLIC_CHECK_REPO'])/'tools/collect_startup_public_evidence.py'
if path.is_symlink():raise ValueError('collector_source_pin')
source=path.read_bytes()
if hashlib.sha256(source).hexdigest()!='96309ac61a07dc23f204cd7c799b25ec67624af4d1103b9057b25728db85b7f6':raise ValueError('collector_source_pin')
c=types.ModuleType('fixed_public_rpc_collector');c.__file__=str(path)
exec(compile(source,str(path),'exec',dont_inherit=True),c.__dict__)
raw=sys.stdin.buffer.read(c.MAX_JSON+1)
if not raw or len(raw)>c.MAX_JSON:raise ValueError('request_bound')
frame={'schema':'piv1-startup-rpc-child-v1'}
try:
    status,headers,body=c.tls_post(raw)
    if type(status) is not int or not 100<=status<=599 or type(body) is not bytes or len(body)>c.MAX_JSON:raise ValueError('response_bound')
    if len(c.encode(headers))>16384:raise ValueError('headers_bound')
    frame.update(kind='RESPONSE',status=status,headers=headers,bodyBase64=base64.b64encode(body).decode())
except c.TransportFailure as exc:
    if type(exc.partial) is not bytes or len(exc.partial)>c.MAX_JSON+1:raise ValueError('partial_bound')
    frame.update(kind='TRANSPORT_FAILURE',partialBase64=base64.b64encode(exc.partial).decode())
except Exception:
    frame.update(kind='FAILURE')
sys.stdout.buffer.write(c.encode(frame));sys.stdout.buffer.flush()
'''

COLLECTION_SCRIPT = r'''
import hashlib,json,os,resource,sys,types
from pathlib import Path
resource.setrlimit(resource.RLIMIT_CORE,(0,0))
path=Path(os.environ['PIV1_PUBLIC_CHECK_REPO'])/'tools/collect_startup_initializer_storage.py'
if path.is_symlink():raise ValueError('readback_source_pin')
source=path.read_bytes()
if hashlib.sha256(source).hexdigest()!='fbc28f1bc5618459e1178bbea45f3db54fedd21363d7abdcfe4794ef0b0e9adb':raise ValueError('readback_source_pin')
r=types.ModuleType('fixed_native_collection');r.__file__=str(path)
exec(compile(source,str(path),'exec',dont_inherit=True),r.__dict__)
c,s,j=r.helpers()
raw=sys.stdin.buffer.read(65537)
if not raw or len(raw)>65536:raise ValueError('collection_input_bound')
value=j.parse(raw)
if set(value)!={'schema','mode','request','outputPath','parentPath','sealSha256'} or value['schema']!='piv1-startup-collection-request-v1':raise ValueError('collection_request')
if value['mode']=='RECOVER':
    if value['parentPath'] is not None or value['sealSha256'] is not None or value['request']['includeExpiry'] is not False:raise ValueError('recovery_scope')
    report=c.collect(value['request'],value['outputPath'],_transport=None)
elif value['mode']=='GOVERNANCE':
    report=r.collect(value['request'],value['parentPath'],value['outputPath'],_transport=None)
else:raise ValueError('collection_mode')
frame={'schema':'piv1-startup-collection-result-v1','mode':value['mode'],'requestSha256':hashlib.sha256(raw).hexdigest(),'outputPath':value['outputPath'],'manifestReceipt':report['manifestReceipt']}
sys.stdout.buffer.write(c.encode(frame));sys.stdout.buffer.flush()
'''


class Stop(ValueError): pass


def need(ok, reason):
    if not ok: raise Stop(reason)


def encode(x):
    return (json.dumps(x, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False) + "\n").encode()


def sha(x): return hashlib.sha256(x).hexdigest()


def _replay_source_hashers():
    """Share only SHA work for freshly reread source in one replay invocation."""
    sources = [None, None, None]  # Operator, selected manifest, signer backend.
    def digest(index, raw):
        prior = sources[index]
        if prior is not None and raw == prior[0]: return prior[1]
        value = sha(raw)
        if prior is None: sources[index] = (raw, value)
        return value
    return (lambda raw: digest(0, raw), lambda raw: digest(1, raw), lambda raw: digest(2, raw))


def fields(x, names): need(type(x) is dict and set(x) == set(names), "object_fields")


def account_payload(digests, keys=None):
    """Base64 bytes from freshly source-derived, verified account predicates."""
    return sum(4*((digests[key]["dataBytes"]+2)//3) for key in (digests if keys is None else keys) if digests[key]["exists"])


def evidence_bytes(j, path, names=None):
    """Count retained regular files in one fixed collection, not an evidence tree."""
    fd=j.directory(path)
    try:
        selected=os.listdir(fd) if names is None else names
        need(len(selected)<=64,"successful_evidence_file_count")
        total=0
        for name in selected:
            need(re.fullmatch(r"[a-z0-9.-]+",name) is not None and name not in (".",".."),"successful_evidence_name")
            source=os.open(name,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK,dir_fd=fd)
            try:
                meta=os.fstat(source);j.secure_meta(meta);total+=meta.st_size
            finally:os.close(source)
        return total
    finally:os.close(fd)


def successful_evidence(j, paths, payload, *, preflight_queries=None, compound_response=False):
    # Successful collections only. Orphans, recaptures and failed/UNKNOWN
    # attempts still consume the existing aggregate ceiling; no finite retry
    # reserve or whole-upload completion guarantee is established here.
    names=None if preflight_queries is None else {f"rpc-{i:02}.{suffix}" for i in range(1,preflight_queries+1) for suffix in ("request.json","started.json","response.bin","http.json")}
    if names is not None and compound_response:names.add("signer-response.json")
    need(sum(evidence_bytes(j,path,names) for path in paths)<=payload+SUCCESS_SERIALIZATION,"successful_evidence_envelope")


def helpers():
    repo = Path(os.environ.get("PIV1_PUBLIC_CHECK_REPO", Path(__file__).resolve().parent.parent))
    path = repo / "tools/collect_startup_initializer_storage.py"
    need(not path.is_symlink() and sha(path.read_bytes()) == READBACK_SHA, "readback_source_pin")
    spec = importlib.util.spec_from_file_location("release_pinned_readback", path)
    r = importlib.util.module_from_spec(spec); spec.loader.exec_module(r)
    c, s, j = r.helpers()
    return r, c, s, j


def describe(j, packets, *, state_plan=None, governance=None, _runtime_metadata=False):
    """Same pinned Node/SDK and strict decoder, used only on zero-signature wires."""
    need(type(_runtime_metadata) is bool and (not _runtime_metadata or state_plan is None and governance is None), "runtime_metadata_mode")
    repo = j.pins(); raw = encode({"packets":packets,"runtimeMetadata":True} if _runtime_metadata else packets if state_plan is None else {"packets":packets,"plan":state_plan,"governance":governance})
    need(len(raw) <= 1048576, "describe_input_bound")
    process = subprocess.Popen([str(j.NODE), "-e", DESCRIBE_SCRIPT], cwd=repo,
        env={"PIV1_PUBLIC_CHECK_REPO": str(repo)}, stdin=subprocess.PIPE,
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, close_fds=True)
    selector = selectors.DefaultSelector(); result = {process.stdout: bytearray(), process.stderr: bytearray()}
    streams = (process.stdin, process.stdout, process.stderr); position = 0; deadline = time.monotonic() + 20
    try:
        for stream in streams:
            os.set_blocking(stream.fileno(), False)
            selector.register(stream, selectors.EVENT_WRITE if stream is process.stdin else selectors.EVENT_READ)
        while selector.get_map():
            need(time.monotonic() < deadline, "describe_timeout")
            for event, _ in selector.select(0.25):
                stream = event.fileobj
                if stream is process.stdin:
                    position += os.write(stream.fileno(), raw[position:position + 65536])
                    if position == len(raw): selector.unregister(stream); stream.close()
                else:
                    chunk = os.read(stream.fileno(), 65536)
                    if not chunk: selector.unregister(stream)
                    else:
                        result[stream].extend(chunk)
                        need(len(result[stream]) <= (524288 if stream is process.stdout else 16384), "describe_output_bound")
        need(process.wait(timeout=max(.001, deadline-time.monotonic())) == 0 and not result[process.stderr], "describe_failure")
        return j.parse(bytes(result[process.stdout]))
    finally:
        if process.poll() is None: process.kill(); process.wait()
        selector.close()
        for stream in streams:
            if not stream.closed: stream.close()


def semantic(reviewed):
    """Mask only the 32-byte recent blockhash. All other message bytes are frozen."""
    fields(reviewed, ("messageBase64", "messageSha256", "feePayer", "requiredSigners", "accountRoles", "lookupTables"))
    try: raw = base64.b64decode(reviewed["messageBase64"], validate=True)
    except Exception as exc: raise Stop("message_base64") from exc
    need(base64.b64encode(raw).decode() == reviewed["messageBase64"] and sha(raw) == reviewed["messageSha256"], "message_hash")
    need(len(raw) >= 4 and (raw[0] < 128 or raw[0] == 128), "message_version")
    cursor = (1 if raw[0] == 128 else 0) + 3; count = 0; start = cursor
    for i in range(3):
        need(cursor < len(raw), "message_truncated")
        b = raw[cursor]; cursor += 1; count |= (b & 127) << (7*i)
        if not b & 128: break
    else: raise Stop("message_shortvec")
    encoded = []; number = count
    while True:
        part = number & 127; number >>= 7; encoded.append(part | (128 if number else 0))
        if not number: break
    need(raw[start:cursor] == bytes(encoded) and 1 <= count <= 256, "message_shortvec")
    offset = cursor + 32*count
    need(offset+32 <= len(raw), "message_blockhash")
    masked = raw[:offset] + bytes(32) + raw[offset+32:]
    return {**{k: reviewed[k] for k in ("feePayer", "requiredSigners", "accountRoles", "lookupTables")}, "maskedMessageSha256": sha(masked)}


def state_digest(c, value):
    """Compact exact full-account expectations; full bytes remain in evidence."""
    if value["exists"] is False:
        return {"exists": False, "owner": None, "executable": None, "lamports": None, "dataBytes": None, "dataSha256": None}
    raw = c.b64(value["dataBase64"], c.MAX_ACCOUNT, True)
    return {"exists": True, "owner": value["owner"], "executable": value["executable"], "lamports": value["lamports"], "dataBytes": len(raw), "dataSha256": sha(raw)}


_STATE_RULE_CODE = {}


def state_rules():
    path = Path(__file__).with_name("startup_state_rules.py")
    need(not path.is_symlink(), "startup_rules_source_pin")
    raw = path.read_bytes()
    need(sha(raw) == STATE_RULES_SHA, "startup_rules_source_pin")
    key = (str(path), STATE_RULES_SHA)
    code = _STATE_RULE_CODE.get(key)
    if code is None:
        code = compile(raw, str(path), "exec", dont_inherit=True)
        _STATE_RULE_CODE[key] = code
    spec = importlib.util.spec_from_file_location("release_startup_rules",path)
    module = importlib.util.module_from_spec(spec); exec(code, module.__dict__)
    module.verify_sources(Path(os.environ.get("PIV1_PUBLIC_CHECK_REPO",Path(__file__).resolve().parent.parent)))
    return module


_MANIFEST_MODULE = None


def startup_manifest(*, _replay_source_sha=None):
    global _MANIFEST_MODULE
    path = Path(__file__).with_name("startup_manifest.py")
    need(not path.is_symlink() and (sha if _replay_source_sha is None else _replay_source_sha[1])(path.read_bytes()) == MANIFEST_SHA, "startup_manifest_source_pin")
    if _MANIFEST_MODULE is None:
        spec = importlib.util.spec_from_file_location("release_selected_manifest",path)
        module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
        _MANIFEST_MODULE = module
    return _MANIFEST_MODULE


def operator_api():
    from types import SimpleNamespace
    return SimpleNamespace(**{name:globals()[name] for name in ("READBACK_SHA","STATE_RULES_SHA","MANIFEST_SHA","ARTIFACT","semantic","describe","state_rules","state_digest")})


def full_startup(manifest):
    return manifest.get("schema") == "piv1-selected-startup-release-manifest-v1"


def active_phase(c, state, phase):
    return startup_manifest().effective(operator_api(),c,state,phase) if full_startup(state["manifest"]) else phase


def typed_phase(phase):
    return any("startupRule" in x for name in ("expectedPrestate","expectedPoststate") for x in phase[name].values())


def phase_clock(phase):
    return phase.get("completeStartup") is True or any(x.get("startupRule") in ("PROPOSAL","HEARTBEAT_REWARD_0") for name in ("expectedPrestate","expectedPoststate") for x in phase[name].values())


def verify_states(c, state, phase, stage, accounts, **context):
    if state["manifest"].get("schema") == "piv1-unselected-startup-preview-manifest-v2":
        operator_source = Path(__file__).absolute()
        need(not operator_source.is_symlink(), "candidate_state_operator_source")
        operator_raw = operator_source.read_bytes()
        adapter = startup_manifest()
        need(type(state) is dict and set(state) <= {"manifest","candidateRuntime","candidatePreparation","candidateFunding"}, "candidate_state_scope")
        need(set(context) <= {"context_slot","inclusion_slot","pre_clock"} and "context_slot" in context, "candidate_state_context")
        need(type(phase) is dict and type(phase.get("index")) is int and 0 <= phase["index"] < 947, "candidate_state_phase")
        index = phase["index"]
        need(encode(phase) == encode(state["manifest"]["phases"][index]), "candidate_state_phase")
        binding = None; preparation = state.get("candidatePreparation"); retained = None
        if index >= 937:
            need(type(preparation) is dict, "candidate_state_preparation_required")
            fields(preparation,("path","sha256"))
            retained = candidate_read_preparation_seal(preparation["path"],preparation["sha256"])
            need(encode(retained["seal"]["manifest"]) == encode(state["manifest"]), "candidate_state_seal_manifest")
            binding = retained["seal"]["initializerBinding"]
        else: need(preparation is None, "candidate_state_early_preparation")
        funding = state.get("candidateFunding")
        if funding is not None:
            fields(funding,("receipt","expectedReceiptSha256"))
        result = adapter.candidate_verify_accounts(state["manifest"],adapter.candidate_phase(index),stage,accounts,
            prior_runtime=state.get("candidateRuntime"),clock_binding=binding,
            funding_receipt=None if funding is None else funding["receipt"],
            funding_receipt_sha256=None if funding is None else funding["expectedReceiptSha256"],**context)
        if retained is not None:
            reread = candidate_read_preparation_seal(preparation["path"],preparation["sha256"])
            need(encode(retained) == encode(reread), "candidate_state_preparation_changed")
            result.update(preparationSealSha256=retained["sealDigest"],initializerBindingSha256=retained["initializerBindingSha256"])
        startup_manifest()  # Fresh source-pin check after verification and any seal readback.
        for relative, expected_sha in adapter.JITO_SOURCE_PINS.items():
            source = operator_source.parent.parent / relative
            need(not source.is_symlink() and sha(source.read_bytes()) == expected_sha, "candidate_state_jito_source_changed")
        need(not operator_source.is_symlink() and operator_source.read_bytes() == operator_raw, "candidate_state_operator_source_changed")
        return result
    expected = phase["expectedPrestate" if stage == "PRE" else "expectedPoststate"]
    need(set(expected) <= set(accounts), "state_account_missing")
    for address, rule in expected.items():
        if "startupRule" not in rule and "jitoRule" not in rule:
            actual_digest = startup_manifest().digest(operator_api(),c,address,accounts[address]) if full_startup(state["manifest"]) else state_digest(c,accounts[address])
            need(actual_digest == rule, "prestate_mismatch" if stage == "PRE" else "poststate_mismatch")
    if full_startup(state["manifest"]):
        startup_manifest().jito_digests(c,phase,accounts,context.get("pre_clock" if stage=="PRE" else "post_clock"))
    if typed_phase(phase): return state_rules().verify(c,state,phase,stage,expected,accounts,**context)
    return copy.deepcopy(state.get("runtimeFacts",{}))


def states_shape(c, j, states):
    need(type(states) is dict and 1 <= len(states) <= 64, "account_subset_bound")
    for key, a in states.items():
        c.b58decode(key, 32)
        if type(a) is dict and "startupRule" in a:
            state_rules().shape(c,j,a); continue
        fields(a, ("exists", "owner", "executable", "lamports", "dataBytes", "dataSha256"))
        need(type(a["exists"]) is bool, "account_exists")
        if not a["exists"]:
            need(all(a[k] is None for k in a if k != "exists"), "absent_account")
        else:
            c.b58decode(a["owner"], 32); j.u64(a["lamports"]); j.digest(a["dataSha256"])
            need(type(a["executable"]) is bool and type(a["dataBytes"]) is int and 0 <= a["dataBytes"] <= c.MAX_ACCOUNT, "account_shape")


def inspect_parent(j, path, expected):
    report = j.apply({"schema": j.SCHEMA, "action": "INSPECT", "expectedTail": expected, "segment": None, "event": None}, path)
    return report


def manifest_shape(c, j, m, parent):
    fields(m, ("schema", "packageSha256", "artifact", "sourcePins", "initializerPreManifestSha256", "parentTail", "historicalLedger", "proposedFeeBudget", "phases") + (("startupProfile",) if full_startup(m) else ()))
    need(m["schema"] in (SCHEMA,"piv1-selected-startup-release-manifest-v1") and m["artifact"] == ARTIFACT and m["parentTail"] == parent["tail"], "manifest_identity")
    j.digest(m["packageSha256"]); j.digest(m["initializerPreManifestSha256"])
    expected_pins = {"readback": READBACK_SHA, "checker": j.CHECKER_SHA, "sdk": j.SDK_SHA, "journal": "bfea079a5740fa66e437930d50793a18289beeae1760af4164e986295c8a6d37"}
    need(parent.get("nextSegmentAllowedByDeclarations") is True and parent.get("initializerProfile") is None, "completed_unsealed_base_required")
    historical = m["historicalLedger"]
    fields(historical, ("cumulativeFeeCapLamports", "reservedFeeLamports", "actualFeeLamports"))
    for value in historical.values(): j.u64(value)
    need(historical["reservedFeeLamports"] == parent["reservedFeeLamports"] and historical["actualFeeLamports"] == parent["actualFeeLamports"] and j.u64(historical["cumulativeFeeCapLamports"]) >= max(j.u64(historical["reservedFeeLamports"]), j.u64(historical["actualFeeLamports"])), "historical_ledger_mismatch")
    budget = m["proposedFeeBudget"]
    fields(budget, ("namespace", "status", "feeCapLamports", "historicalLedgerReference"))
    need(budget["namespace"] == "piv1-testnet-startup-release-" + m["packageSha256"] and budget["status"] == "PROPOSAL_ONLY_NOT_AUTHORIZED" and budget["feeCapLamports"] == "4815000", "proposed_fee_namespace")
    need(budget["historicalLedgerReference"] == {"parentTail": m["parentTail"], "ledger": historical}, "proposed_fee_historical_reference")
    phases = m["phases"]
    need(type(phases) is list and 1 <= len(phases) <= MAX_PHASES, "phase_capacity")
    ids = set(); initializer = []; seen_initializer = False
    for index, p in enumerate(phases):
        fields(p, ("id", "kind", "initializerIndex", "reviewed", "feeCapLamports", "minimumRemainingBlockHeights", "expectedPrestate", "expectedPoststate"))
        need(type(p["id"]) is str and re.fullmatch(r"[A-Za-z0-9_-]{1,64}", p["id"]) and p["id"] not in ids, "phase_id")
        ids.add(p["id"]); need(p["kind"] in KINDS, "phase_kind")
        j.u64(p["feeCapLamports"]); need(0 < j.u64(p["minimumRemainingBlockHeights"]), "lifetime_margin")
        if not full_startup(m):
            states_shape(c, j, p["expectedPrestate"]); states_shape(c, j, p["expectedPoststate"])
        if p["kind"] == "INITIALIZER":
            need(p["reviewed"] is None and type(p["initializerIndex"]) is int and p["initializerIndex"] == len(initializer), "initializer_deferred_exact_order")
            initializer.append(index); seen_initializer = True
        else:
            need(p["initializerIndex"] is None and p["reviewed"] is not None, "phase_template")
            semantic(p["reviewed"])
            need(not seen_initializer or p["kind"] == "HEARTBEAT", "prephase_after_initializer")
            if p["kind"] == "HEARTBEAT": need(len(initializer) == 9 and index == len(phases)-1, "heartbeat_after_nine")
    need(not initializer or len(initializer) == 9 and initializer == list(range(initializer[0], initializer[0]+9)), "nine_initializer_order")
    if full_startup(m) or any(typed_phase(p) for p in phases): expected_pins["stateRules"] = STATE_RULES_SHA
    if full_startup(m):
        adapter = startup_manifest()
        expected_pins.update(startupManifest=MANIFEST_SHA,startupOracle=adapter.ORACLE_SHA)
        adapter.validate(operator_api(),j,m)
    need(m["sourcePins"] == expected_pins, "manifest_source_pins")


def journal_path(base): return Path(os.path.abspath(base)).with_name(Path(base).name + ".release-operator")


def signing_request(c, state, previous, ids, authorization_sha256, *, _replay_source_sha=None, _phase_positions=None):
    """Public request only; no signer backend or proof of key control."""
    need(type(ids) is list and 1 <= len(ids) <= MAX_BATCH and all(type(x) is str for x in ids) and len(set(ids)) == len(ids), "handoff_batch")
    ordered = [p["id"] for p in state["manifest"]["phases"]] if _phase_positions is None else None
    need(all(pid in state["prepared"] and pid not in state["signed"] and pid not in state["intents"] for pid in ids), "handoff_phase")
    def position(pid):
        if _phase_positions is not None and type(pid) is str and pid in _phase_positions: return _phase_positions[pid]
        return (ordered if ordered is not None else [p["id"] for p in state["manifest"]["phases"]]).index(pid)
    need(ids == sorted(ids, key=position), "handoff_order")
    packets = []
    for pid in ids:
        prepared = state["prepared"][pid]
        packets.append({"id":pid,"preparedRecordDigest":state["preparedAt"][pid],"preparedSha256":sha(encode(prepared)),
            "unsignedWireSha256":sha(base64.b64decode(prepared["wireBase64"],validate=True)),
            "messageSha256":prepared["reviewed"]["messageSha256"],"requiredSigners":copy.deepcopy(prepared["reviewed"]["requiredSigners"])})
    body = {"schema":"piv1-startup-signing-request-v1","cluster":"testnet","genesisHash":c.GENESIS,"operatorTail":previous,
        "operatorSourceSha256":(sha if _replay_source_sha is None else _replay_source_sha[0])(Path(__file__).read_bytes()),"authorizationSha256":authorization_sha256,
        "baseParentPath":state["baseParentPath"],"baseParentTail":state["manifest"]["parentTail"],
        "manifestSha256":state["manifestSha256"],"sealSha256":sha(encode(state["seal"])) if state["seal"] is not None else None,"packets":packets}
    return {**body,"requestId":sha(encode(body))}


def signed_entry(c, state, packet):
    fields(packet,("id","wireBase64")); pid=packet["id"]
    need(type(pid) is str and pid in state["prepared"] and pid not in state["signed"] and pid not in state["intents"], "signed_phase")
    wire=c.packet(packet["wireBase64"]); prepared=state["prepared"][pid]
    unsigned=base64.b64decode(prepared["wireBase64"],validate=True); count=wire["wire"][0]; offset=1+64*count
    need(count == len(prepared["reviewed"]["requiredSigners"]) and unsigned[0] == count and wire["wire"][offset:] == unsigned[offset:], "signed_prepared_message_binding")
    need(wire["messageSha256"] == prepared["reviewed"]["messageSha256"] and all(any(wire["wire"][1+64*i:65+64*i]) for i in range(count)), "signed_required_signatures")
    return {**packet,"preparedSha256":sha(encode(prepared)),"wireSha256":sha(wire["wire"]),"signatureHex":wire["signatureHex"],"signature":wire["signature"]}


def signing_response(state, response, authorization_sha256):
    fields(response,("schema","requestId","packets"))
    pending=state["handoff"]
    need(pending is not None and response["schema"] == "piv1-startup-signing-response-v1" and response["requestId"] == pending["requestId"], "signing_request_binding")
    need(authorization_sha256 == pending["authorizationSha256"], "signing_authorization_changed")
    packets=response["packets"]
    need(type(packets) is list and len(packets) == len(pending["packets"]), "signing_response_batch")
    for packet, expected in zip(packets,pending["packets"]):
        fields(packet,("id","wireBase64"))
        need(packet["id"] == expected["id"], "signing_response_order")
    return packets


def signer_backend_source(*, _replay_source_sha=None):
    path=Path(__file__).with_name("startup_signer_backend.cjs")
    need(not path.is_symlink() and (sha if _replay_source_sha is None else _replay_source_sha[2])(path.read_bytes()) == SIGNER_BACKEND_SHA,"signer_backend_source_pin")
    return path


def import_response(c,j,state,response,authorization_pin):
    packets=signing_response(state,response,authorization_pin);entries=[]
    for packet in packets:
        signed=signed_entry(c,state,packet)
        need(not any(x["wireSha256"] == signed["wireSha256"] or x["signatureHex"] == signed["signatureHex"] for x in state["signedHistory"]+entries),"global_signed_replay")
        phase=next(p for p in state["manifest"]["phases"] if p["id"] == packet["id"])
        check_signed(j,frozen(state),phase,state["prepared"][packet["id"]],signed);entries.append(signed)
    return {"requestId":response["requestId"],"responseSha256":sha(encode(response)),"authorizationSha256":authorization_pin,"entries":entries}


def invoke_signer(j,payload):
    """Fixed executable/argv, bounded pipes/acceptance; no raw diagnostics."""
    deadline=time.monotonic()+20
    repo=j.pins();backend=signer_backend_source();raw=encode(payload)
    need(len(raw)<=1048576,"signer_input_bound")
    process=None;selector=selectors.DefaultSelector();streams=();output=bytearray()
    try:
        need(time.monotonic()<deadline,"signer_timeout")
        process=subprocess.Popen([str(j.NODE),str(backend),"--allow-live-sign"],cwd=repo,
            env={"PIV1_PUBLIC_CHECK_REPO":str(repo)},stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,close_fds=True,preexec_fn=disable_signer_core_dumps)
        streams=(process.stdin,process.stdout);position=0
        for stream in streams:
            os.set_blocking(stream.fileno(),False);selector.register(stream,selectors.EVENT_WRITE if stream is process.stdin else selectors.EVENT_READ)
        while selector.get_map():
            need(time.monotonic()<deadline,"signer_timeout")
            for event,_ in selector.select(.25):
                stream=event.fileobj
                if stream is process.stdin:
                    position+=os.write(stream.fileno(),raw[position:position+65536])
                    if position==len(raw):selector.unregister(stream);stream.close()
                else:
                    chunk=os.read(stream.fileno(),65536)
                    if not chunk:selector.unregister(stream)
                    else:output.extend(chunk);need(len(output)<=262144,"signer_output_bound")
        remaining=deadline-time.monotonic();need(remaining>0,"signer_timeout")
        need(process.wait(timeout=remaining)==0,"signer_backend_refused")
        response=j.parse(bytes(output))
    except Exception as exc:
        raise Stop("signer_attempt_unresolved") from None
    finally:
        if process is not None and process.poll() is None:process.kill();process.wait()
        selector.close()
        for stream in streams:
            if not stream.closed:stream.close()
    need(time.monotonic()<deadline,"signer_attempt_unresolved")
    return response


def disable_signer_core_dumps():
    resource.setrlimit(resource.RLIMIT_CORE,(0,0))


COMPOUND_PREPARE_DOMAIN = b"piv1-startup-compound-prepared-content-v1\0"


def compound_prepared_digest(previous, entries, quote):
    """Content commitment for compound records, not their enclosing digest."""
    return sha(COMPOUND_PREPARE_DOMAIN + encode({"previousTail":previous,"entries":entries,"quote":quote}))


def replay_prepare(c, j, state, m, ids, record, d, full, *, prepared_digest=None, _replay_source_sha=None, _phase_positions=None):
    capture=type(d) is dict
    if capture:
        fields(d,("entries","quote"));need(full,"full_startup_required_for_quote")
        entries=d["entries"];need(type(entries) is list and 1<=len(entries)<=23,"quote_batch_bound")
        next_floor=verify_quote(c,j,state,entries,d["quote"],record,_replay_source_sha=_replay_source_sha,_phase_positions=_phase_positions)
    else:entries=d
    need(type(entries) is list and 1 <= len(entries) <= MAX_BATCH and len({p["id"] for p in entries}) == len(entries), "prepare_replay_batch")
    for p in entries:
        indexed = _phase_positions is not None and type(p["id"]) is str and p["id"] in _phase_positions
        need((indexed or p["id"] in ids) and p["id"] not in state["intents"] and (_phase_positions[p["id"]] if indexed else ids.index(p["id"])) >= state["cursor"], "prepare_after_intent")
        phase = m["phases"][_phase_positions[p["id"]] if indexed else ids.index(p["id"])]
        need(semantic(p["reviewed"]) == semantic(phase_template(state, phase)), "prepared_semantics_changed")
        need(p["lifetime"]["minimumRemainingBlockHeights"] == phase["minimumRemainingBlockHeights"] and j.u64(p["feeQuoteLamports"]) <= j.u64(phase["feeCapLamports"]), "prepared_limits_changed")
        if full: need(p["feeQuoteLamports"] == phase["feeCapLamports"], "startup_exact_fee_profile")
        prior = state["prepared"].get(p["id"])
        if prior and prior["lifetime"]["recentBlockhash"] == p["lifetime"]["recentBlockhash"]:
            need(prior["lifetime"]["lastValidBlockHeight"] == p["lifetime"]["lastValidBlockHeight"] and j.u64(p["lifetime"]["currentBlockHeight"]) >= j.u64(prior["lifetime"]["currentBlockHeight"]), "same_blockhash_lifetime_changed")
        state["prepared"][p["id"]] = p
        state["preparedAt"][p["id"]] = record["digest"] if prepared_digest is None else prepared_digest
        state["signed"].pop(p["id"], None)
        if capture:state["quoteByPhase"][p["id"]]=d["quote"]
        else:state["quoteByPhase"].pop(p["id"],None)
    if capture:
        state["floor"]=max(state["floor"],next_floor)
        state["transportMode"]=d["quote"]["transportMode"]
    state["handoff"] = None


def replay_sign_attempt(c, j, state, record, d, full, unresolved, *, _replay_source_sha=None, _phase_positions=None):
    fields(d,("request","authorization","signerBackendSha256","signerProfileSha256","mode"))
    need(full and not unresolved,"canonical_signer_boundary")
    a=d["authorization"];auth_raw=encode(a);auth_pin=authorization(j,state,auth_raw,sha(auth_raw),allow_expired=True,require_signer=True,_replay_source_sha=_replay_source_sha)
    need(d["request"]["requestId"] not in state["signerAttempts"],"signer_request_already_attempted")
    need(d["request"]==signing_request(c,state,record["previousDigest"],[p["id"] for p in d["request"]["packets"]],auth_pin,_replay_source_sha=_replay_source_sha,_phase_positions=_phase_positions),"signer_attempt_request_binding")
    captured_authorization(state,[p["id"] for p in d["request"]["packets"]],auth_pin)
    need(d["signerBackendSha256"]==a["signerBackendSha256"]==SIGNER_BACKEND_SHA and d["signerProfileSha256"]==a["signerProfileSha256"],"signer_attempt_profile_binding")
    need(d["mode"] in ("LOCAL_TESTNET_BACKEND","OFFLINE_INJECTED_TEST"),"signer_attempt_mode")
    if d["mode"]=="OFFLINE_INJECTED_TEST":
        need(state["transportMode"] in (None,"OFFLINE_INJECTED_TEST"),"transport_mode_changed")
        state["transportMode"]="OFFLINE_INJECTED_TEST"
    else:need(state["transportMode"]!="OFFLINE_INJECTED_TEST","injected_history_cannot_sign_live")
    state["handoff"]=d["request"];state["signerAttempt"]=d;state["signerAttempts"][d["request"]["requestId"]]=d


def replay_import(c, j, state, d, full, signed_wire_hashes, signed_signature_hexes):
    bound = type(d) is dict
    if bound:
        fields(d,("requestId","responseSha256","authorizationSha256","entries"))
        need(type(d["entries"]) is list and all(type(p) is dict and "id" in p and "wireBase64" in p for p in d["entries"]), "import_replay_batch")
        response={"schema":"piv1-startup-signing-response-v1","requestId":d["requestId"],"packets":[{k:p[k] for k in ("id","wireBase64")} for p in d["entries"]]}
        packets=signing_response(state,response,d["authorizationSha256"])
        need(sha(encode(response)) == d["responseSha256"], "signing_response_digest")
        # Rederive bytes/hashes/linkage on replay. Cryptographic checks
        # run at import and again before SEND, not on every past record.
        need(d["entries"] == [signed_entry(c,state,p) for p in packets], "signed_response_replay_binding")
        entries=d["entries"]
    else:
        need(not full, "signing_response_required")
        need(state["handoff"] is None, "signing_response_required")
        entries=d
    need(type(entries) is list and 1 <= len(entries) <= MAX_BATCH and len({p["id"] for p in entries}) == len(entries), "import_replay_batch")
    for p in entries:
        need(p["id"] in state["prepared"] and p["id"] not in state["signed"], "import_replay")
        need(p["id"] not in state["intents"] and p["preparedSha256"] == sha(encode(state["prepared"][p["id"]])), "import_prepared_binding")
        if full:
            need(p["wireSha256"] not in signed_wire_hashes and p["signatureHex"] not in signed_signature_hexes, "signed_history_replay")
        else:
            need(not any(x["wireSha256"] == p["wireSha256"] or x["signatureHex"] == p["signatureHex"] for x in state["signedHistory"]), "signed_history_replay")
        state["signed"][p["id"]] = {**p,**({"requestId":d["requestId"],"authorizationSha256":d["authorizationSha256"]} if bound else {})}
        state["signedHistory"].append(p)
        if full:
            signed_wire_hashes.add(p["wireSha256"])
            signed_signature_hexes.add(p["signatureHex"])
    state["handoff"] = None
    state["signerAttempt"] = None


def replay_attempt(state, ids, d, full, unresolved):
    need(state["cursor"] < len(ids) and d["id"] == ids[state["cursor"]] and d["stage"] in ("SEND", "RECOVER", "PRE_VOTE", "POST_VOTE"), "attempt_phase")
    need((d["stage"] == "RECOVER") == unresolved, "attempt_intent_relation")
    need(d["transportMode"] in ("FIXED_ORIGIN_TLS", "OFFLINE_INJECTED_TEST") and (state["transportMode"] is None or state["transportMode"] == d["transportMode"]), "transport_mode_changed")
    need(full or d["transportMode"] == "OFFLINE_INJECTED_TEST", "full_startup_required_for_live_transport")
    state["transportMode"] = d["transportMode"]
    state["attempt"] = d


def compound_response(j, attempt, receipt):
    """R remains an exact public-byte replay dependency, never an adopted reply."""
    fields(receipt,("responseSha256","signerAttemptSha256"))
    fd=j.directory(attempt["evidencePath"])
    try:raw=j.read_file(fd,"signer-response.json",262144)
    finally:os.close(fd)
    response=j.parse(raw)
    need(encode(response)==raw and sha(raw)==receipt["responseSha256"],"compound_response_receipt")
    return response


def replay_compound_import(c,j,state,ids,record,d,full,unresolved,compound_requests,signed_wire_hashes,signed_signature_hexes):
    fields(d,("import","attempt","response") if record["kind"]=="IMPORT_SEND_ATTEMPT_INTENT" else ("import","attempt"))
    pending=state["signerAttempt"]
    need(full and pending is not None and pending["request"]["requestId"] in compound_requests,"compound_original_attempt_required")
    need(type(d["import"]) is dict and type(d["import"].get("entries")) is list and len(d["import"]["entries"])==1,"compound_single_import")
    fields(d["attempt"],("id","stage","evidencePath","transportMode"))
    pid=ids[state["cursor"]]
    need(pending["request"]["packets"][0]["id"]==pid and type(d["import"]["entries"][0]) is dict and
        d["import"]["entries"][0].get("id")==d["attempt"]["id"]==pid,"compound_current_import_attempt")
    if "response" in d:
        root=journal_path(state["baseParentPath"]);root=root.with_name(root.name+".evidence")
        need(d["attempt"]["evidencePath"]==str(root/(record["sequence"].zfill(8)+"-send")),"compound_send_path_binding")
        fields(d["response"],("responseSha256","signerAttemptSha256"))
        need(d["response"]["signerAttemptSha256"]==sha(encode(pending)),"signer_attempt_changed")
        response=compound_response(j,d["attempt"],d["response"])
        need(response=={"schema":"piv1-startup-signing-response-v1","requestId":d["import"]["requestId"],"packets":[{k:p[k] for k in ("id","wireBase64")} for p in d["import"]["entries"]]},"compound_response_binding")
    replay_import(c,j,state,d["import"],full,signed_wire_hashes,signed_signature_hexes)
    need(d["attempt"]["stage"]=="SEND","compound_send_attempt_required")
    root=journal_path(state["baseParentPath"]);root=root.with_name(root.name+".evidence")
    need(d["attempt"]["evidencePath"]==str(root/(record["sequence"].zfill(8)+"-send")),"compound_send_path_binding")
    replay_attempt(state,ids,d["attempt"],full,unresolved)


def compound_dispatch(c,j,op,record,data_raw,attempt_raw,prepared_raw,a_raw):
    """Read only this durable C and its exact original public inputs, not a replay result."""
    op.stable()
    raw=j.read_file(op.fd,f"{int(record['sequence']):08}.json",j.MAX_INPUT)
    committed=j.parse(raw)
    need(raw==encode(record) and committed["digest"]==op.tail() and committed["kind"]=="IMPORT_SEND_ATTEMPT_INTENT" and
        encode(committed["data"])==data_raw,"compound_dispatch_record_changed")
    a=j.parse(a_raw)
    need(committed["previousDigest"]==a["digest"] and int(committed["sequence"])==int(a["sequence"])+1 and
        j.read_file(op.fd,f"{int(a['sequence']):08}.json",j.MAX_INPUT)==a_raw,"compound_dispatch_original_attempt")
    d=committed["data"];original=j.parse(attempt_raw);prepared=j.parse(prepared_raw);pid=prepared["id"]
    need(a["data"]["attempt"]==original and d["response"]["signerAttemptSha256"]==sha(attempt_raw),"signer_attempt_changed")
    response=compound_response(j,d["attempt"],d["response"])
    need(response["requestId"]==original["request"]["requestId"]==d["import"]["requestId"] and
        sha(encode(response))==d["import"]["responseSha256"] and len(response["packets"])==1,"compound_dispatch_response")
    signed=signed_entry(c,{"prepared":{pid:prepared},"signed":{},"intents":{}},response["packets"][0])
    need(d["import"]["entries"]==[signed] and original["request"]["packets"][0]["preparedSha256"]==sha(prepared_raw) and
        d["import"]["authorizationSha256"]==original["request"]["authorizationSha256"],"compound_dispatch_packet")
    create_raw=j.read_file(op.fd,"00000000.json",j.MAX_INPUT)
    need(create_raw==encode(op.chain[0]),"compound_dispatch_create")
    manifest=j.parse(create_raw)["data"]["manifest"];manifest_sha=sha(encode(manifest))
    need(manifest_sha==original["request"]["manifestSha256"],"compound_dispatch_manifest")
    phase=next(p for p in manifest["phases"] if p["id"]==pid)
    need(prepared["lifetime"]["minimumRemainingBlockHeights"]==phase["minimumRemainingBlockHeights"],"compound_dispatch_lifetime")
    return prepared,signed,d["intent"],{"manifest":manifest,"manifestSha256":manifest_sha},phase


class Journal:
    """Exclusive append-only successor; partial writes are never repaired/adopted."""
    def __init__(self, j, base, create=False, *, _candidate=False):
        need(type(_candidate) is bool, "candidate_journal_selector")
        self.candidate = _candidate
        self.schema,self.lock_bytes = (CANDIDATE_CONTROLLER_SCHEMA,CANDIDATE_CONTROLLER_LOCK) if _candidate else (SCHEMA,LOCK)
        self.j = j; self.path = Path(os.path.abspath(base)) if _candidate else journal_path(base)
        need(not _candidate or self.path.name.endswith(".candidate-controller"), "candidate_controller_namespace")
        self.fd = j.directory(self.path, create); self.lock = None; self.directory_meta = os.fstat(self.fd)
        try:
            if create: j.write_file(self.fd, "lock", self.lock_bytes)
            self.lock = os.open("lock", os.O_RDWR | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=self.fd)
            self.meta = os.fstat(self.lock); j.secure_meta(self.meta); fcntl.flock(self.lock, fcntl.LOCK_EX)
            self.identity = {"device": str(self.meta.st_dev), "inode": str(self.meta.st_ino), "ctimeNs": str(self.meta.st_ctime_ns)}
            self.stable(); names = os.listdir(self.fd)
            need(all(n == "lock" or re.fullmatch(r"[0-9]{8}\.json", n) for n in names), "operator_namespace")
            names = sorted(n for n in names if n != "lock")
            need(len(names) <= (CANDIDATE_STARTUP_RECORDS if self.candidate else MAX_RECORDS) and names == [f"{i:08}.json" for i in range(len(names))], "record_gap")
            need(create or names, "missing_create_record"); self.chain = []; self.total = 0
            for i, name in enumerate(names):
                raw = j.read_file(self.fd, name, j.MAX_INPUT); record = j.parse(raw)
                fields(record, ("schema", "sequence", "previousDigest", "lockIdentity", "kind", "data", "digest"))
                need(record["schema"] == self.schema and record["sequence"] == str(i) and record["previousDigest"] == self.tail() and record["lockIdentity"] == self.identity, "record_link")
                need(encode(record) == raw and sha(encode({k:v for k,v in record.items() if k != "digest"})) == record["digest"], "record_digest")
                self.chain.append(record); self.total += len(raw)
                need(self.total <= MAX_JOURNAL, "operator_journal_bound")
            if self.candidate: self.stable()
        except Exception:
            self.close(); raise

    def tail(self): return self.chain[-1]["digest"] if self.chain else None

    def stable(self):
        now = os.stat("lock", dir_fd=self.fd, follow_symlinks=False)
        need((now.st_dev, now.st_ino, now.st_ctime_ns) == (self.meta.st_dev, self.meta.st_ino, self.meta.st_ctime_ns) and self.j.read_file(self.fd,"lock",128) == self.lock_bytes, "stable_lock")
        if self.candidate:
            here = os.stat(self.path,follow_symlinks=False); opened = os.fstat(self.fd)
            need(stat.S_ISDIR(here.st_mode) and stat.S_IMODE(here.st_mode) == 0o700 and here.st_uid == os.getuid()
                and (here.st_dev,here.st_ino) == (opened.st_dev,opened.st_ino) == (self.directory_meta.st_dev,self.directory_meta.st_ino), "candidate_controller_directory_changed")

    def append(self, kind, data, *, expected_signer_attempt=None):
        need(not self.candidate or expected_signer_attempt is None, "candidate_signer_attempt_forbidden")
        self.stable(); need(len(self.chain) < (CANDIDATE_STARTUP_RECORDS if self.candidate else MAX_RECORDS), "operator_record_bound")
        body = {"schema": self.schema, "sequence": str(len(self.chain)), "previousDigest": self.tail(), "lockIdentity": self.identity, "kind": kind, "data": data}
        record = {**body, "digest": sha(encode(body))}; raw = encode(record)
        need(len(raw) <= self.j.MAX_INPUT and self.total + len(raw) <= MAX_JOURNAL, "operator_journal_bound")
        if self.candidate: record = self.j.parse(raw)  # Replay/chain consume the exact captured bytes.
        # Reject invalid transitions before committing an immutable record.
        prospective = _candidate_controller_replay(self.j,self.chain+[record],self.path) if self.candidate else replay(self.j, self.chain + [record])
        if self.candidate: self.stable()
        if expected_signer_attempt is not None:
            # SIGN merges its post-callback replay into this mandatory fresh
            # precommit replay. IMPORT consumes only the current exact request;
            # signerAttempts retains the complete original attempt afterwards.
            request_id = expected_signer_attempt["request"]["requestId"]
            imported_data = data["import"] if kind in ("IMPORT_SEND_ATTEMPT","IMPORT_SEND_ATTEMPT_INTENT") else data
            need(kind in ("IMPORT","IMPORT_SEND_ATTEMPT","IMPORT_SEND_ATTEMPT_INTENT") and type(imported_data) is dict and imported_data["requestId"] == request_id and
                prospective["signerAttempts"].get(request_id) == expected_signer_attempt,
                "signer_attempt_changed")
        try: self.j.write_file(self.fd, f"{len(self.chain):08}.json", raw)
        except Exception as exc: raise Stop("commit_uncertain_no_retry_or_adoption") from exc
        if self.candidate:
            self.stable()
            need(self.j.read_file(self.fd,f"{len(self.chain):08}.json",self.j.MAX_INPUT) == raw, "candidate_controller_commit_readback")
        self.chain.append(record); self.total += len(raw); return record

    def close(self):
        if self.lock is not None: fcntl.flock(self.lock, fcntl.LOCK_UN); os.close(self.lock); self.lock = None
        if self.fd is not None: os.close(self.fd); self.fd = None


def replay(j, records):
    need(records and records[0]["kind"] == "CREATE", "create_required")
    source_sha = _replay_source_hashers()
    # Pinned rule functions do not mutate their module globals. Share only
    # their code/globals within this side-effect-free replay, never its results
    # or module across a commit, callback or another replay invocation.
    fresh_rules = state_rules; rules_module = None
    send_payloads = {}  # Derived only after fresh INTENT coverage equality below.
    compound_requests = set()  # Replay-local origin of the new compound attempt.
    def replay_rules():
        nonlocal rules_module
        if rules_module is None: rules_module = fresh_rules()
        return rules_module
    api = operator_api(); api.state_rules = replay_rules
    first = records[0]["data"]; m = first["manifest"]
    full = full_startup(m)
    # Full-mode IMPORT rederives canonical string hashes before membership.
    # These indexes belong only to this replay; signedHistory stays unchanged.
    signed_wire_hashes = set() if full else None
    signed_signature_hexes = set() if full else None
    adapter = startup_manifest(_replay_source_sha=source_sha) if full else None
    c = helpers()[1] if full or any(r["kind"] in ("HANDOFF","SIGN_ATTEMPT") for r in records) else None
    if full: adapter.validate(api,j,m)
    state = {"manifest": m, "manifestSha256": sha(encode(m)), "cursor": 0, "prepared": {}, "signed": {}, "intents": {}, "terminal": {}, "seal": None, "preVote": None, "postVote": None,
        "reserved": 0, "actual": 0, "floor": j.u64(first["contextFloor"]), "lastSlot": j.u64(first["lastFinalizedSlot"]), "failed": False, "signedHistory": [], "attempt": None, "transportMode": None, "runtimeFacts":{}, "runtimeClock":None,
        "baseParentPath":first["baseParentPath"],"preparedAt":{},"handoff":None,"signerAttempt":None,"signerAttempts":{},"quoteByPhase":{},"voteGateAt":{},"voteGateClock":{}}
    need(first["historicalLedger"] == m["historicalLedger"], "historical_ledger_rebind")
    ids = [p["id"] for p in m["phases"]]
    phase_positions = {pid:index for index,pid in enumerate(ids)} if full else None
    for record in records[1:]:
        kind, d = record["kind"], record["data"]
        compound_intent=kind=="IMPORT_SEND_ATTEMPT_INTENT"
        # Terminal insertion requires an existing intent and forbids repeats;
        # neither dictionary removes entries, so terminal is a subset of intents.
        unresolved = len(state["intents"]) != len(state["terminal"])
        need(not state["failed"], "record_after_finalized_failure")
        if state["signerAttempt"] is not None: need(kind in ("IMPORT","IMPORT_SEND_ATTEMPT","IMPORT_SEND_ATTEMPT_INTENT"),"signer_attempt_unresolved")
        if kind in ("BIND_SEAL", "PREPARE", "HANDOFF", "IMPORT", "PRE_VOTE", "POST_VOTE", "PREPARE_SIGN_ATTEMPT", "IMPORT_SEND_ATTEMPT", "IMPORT_SEND_ATTEMPT_INTENT"):
            need(not unresolved, "unresolved_intent_blocks_transition")
        if compound_intent:
            fields(d,("import","attempt","response","intent"))
            replay_compound_import(c,j,state,ids,record,{k:d[k] for k in ("import","attempt","response")},full,unresolved,compound_requests,signed_wire_hashes,signed_signature_hexes)
            kind,d="INTENT",d["intent"]
        if kind == "BIND_SEAL":
            need(state["seal"] is None and not state["failed"] and state["cursor"] < len(ids) and m["phases"][state["cursor"]]["kind"] == "INITIALIZER", "seal_transition")
            need(d["baseParentTail"] == m["parentTail"], "seal_original_parent_binding")
            need(state["transportMode"] is None or state["transportMode"] == d["transportMode"], "transport_mode_changed")
            state["transportMode"] = d["transportMode"]
            if full: adapter.validate_seal(api,j,state,d)
            state["seal"] = d
            state["handoff"] = None
        elif kind == "PREPARE":
            replay_prepare(c,j,state,m,ids,record,d,full,_replay_source_sha=source_sha,_phase_positions=phase_positions)
        elif kind == "PREPARE_SIGN_ATTEMPT":
            fields(d,("entries","quote","attempt"))
            need(full and state["cursor"] < len(ids) and state["handoff"] is None,"compound_current_phase_required")
            entries=d["entries"]
            need(type(entries) is list and len(entries)==1 and type(entries[0]) is dict and entries[0].get("id")==ids[state["cursor"]],"compound_current_phase_required")
            pid=ids[state["cursor"]]
            need(pid not in state["prepared"] and pid not in state["signed"] and pid not in state["intents"],"compound_phase_already_prepared")
            commitment=compound_prepared_digest(record["previousDigest"],entries,d["quote"])
            replay_prepare(c,j,state,m,ids,record,{"entries":entries,"quote":d["quote"]},full,prepared_digest=commitment,_replay_source_sha=source_sha,_phase_positions=phase_positions)
            need(type(d["attempt"]) is dict and type(d["attempt"].get("request")) is dict and type(d["attempt"]["request"].get("packets")) is list and len(d["attempt"]["request"]["packets"])==1,"compound_single_signing_request")
            need(type(d["attempt"]["request"]["packets"][0]) is dict and d["attempt"]["request"]["packets"][0].get("id")==pid,"compound_current_signing_request")
            replay_sign_attempt(c,j,state,record,d["attempt"],full,unresolved,_replay_source_sha=source_sha,_phase_positions=phase_positions)
            compound_requests.add(d["attempt"]["request"]["requestId"])
        elif kind == "IMPORT_SEND_ATTEMPT":
            replay_compound_import(c,j,state,ids,record,d,full,unresolved,compound_requests,signed_wire_hashes,signed_signature_hexes)
        elif kind == "HANDOFF":
            fields(d,("schema","cluster","genesisHash","operatorTail","operatorSourceSha256","authorizationSha256","baseParentPath","baseParentTail","manifestSha256","sealSha256","packets","requestId"))
            j.digest(d["authorizationSha256"])
            need(type(d["packets"]) is list and all(type(p) is dict and "id" in p for p in d["packets"]), "handoff_batch")
            need(d == signing_request(c,state,record["previousDigest"],[p["id"] for p in d["packets"]],d["authorizationSha256"],_replay_source_sha=source_sha,_phase_positions=phase_positions), "handoff_replay_binding")
            captured_authorization(state,[p["id"] for p in d["packets"]],d["authorizationSha256"])
            state["handoff"] = d
        elif kind == "SIGN_ATTEMPT":
            replay_sign_attempt(c,j,state,record,d,full,unresolved,_replay_source_sha=source_sha,_phase_positions=phase_positions)
        elif kind == "IMPORT":
            replay_import(c,j,state,d,full,signed_wire_hashes,signed_signature_hexes)
        elif kind == "ATTEMPT":
            replay_attempt(state,ids,d,full,unresolved)
        elif kind == "INTENT":
            phase = d["id"]
            need(not state["failed"] and state["cursor"] < len(ids) and phase == ids[state["cursor"]] and phase in state["signed"] and phase not in state["intents"], "intent_order_or_duplicate")
            need(d["wireSha256"] == state["signed"][phase]["wireSha256"] and d["signatureHex"] == state["signed"][phase]["signatureHex"] and d["signature"] == state["signed"][phase]["signature"] and d["feeLamports"] == state["prepared"][phase]["feeQuoteLamports"], "intent_signed_binding")
            need(d["preparedSha256"] == state["signed"][phase]["preparedSha256"] and state["attempt"] is not None and state["attempt"]["stage"] == "SEND" and d["evidencePath"] == state["attempt"]["evidencePath"] and d["transportMode"] == state["attempt"]["transportMode"], "intent_attempt_binding")
            if "requestId" in state["signed"][phase]: need(d["authorizationSha256"] == state["signed"][phase]["authorizationSha256"], "signing_authorization_changed")
            captured_authorization(state,[phase],d["authorizationSha256"])
            if full: need("requestId" in state["signed"][phase], "signing_response_required")
            need(j.u64(d["prestateContextSlot"]) >= state["floor"], "intent_prestate_stale")
            p = m["phases"][state["cursor"]]
            if full:
                adapter = startup_manifest(_replay_source_sha=source_sha)
                if p["kind"] != "UPLOAD_WRITE": p = adapter.effective(api,c,state,p)
            if full:
                need(d["runtimePreFacts"] == state["runtimeFacts"], "startup_intent_facts_binding")
                replay_rules().saved_clock(d["runtimePreClock"])
                need(j.u64(d["runtimePreClock"]["slot"]) >= state["floor"] and d["runtimePreClock"]["contextSlot"] == d["prestateContextSlot"], "startup_intent_clock_binding")
                if state["runtimeClock"] is not None: need(int(d["runtimePreClock"]["unixTimestamp"]) >= int(state["runtimeClock"]["unixTimestamp"]), "startup_preclock_regression")
                raw_accounts,preflight=verify_preflight_accounts(c,j,state,p,d,d["coverage"].get("preflightReceipt")) if p["kind"]!="UPLOAD_WRITE" else (None,None)
                coverage=adapter.coverage(api,c,state,p,"PRE",state["prepared"][phase]["reviewed"],d["runtimePreFacts"],d["runtimePreClock"],raw_accounts,preflight_receipt=preflight)
                need(d["coverage"] == coverage, "startup_intent_coverage_binding")
                send_payloads[phase]=account_payload(coverage["accountDigests"])
                # Later post-INTENT/send failure files must never poison the
                # durable original intent or its read-only recovery path.
                successful_evidence(j,[Path(d["evidencePath"])],send_payloads[phase],preflight_queries=(10+len(adapter.rent_sizes(p)))//5,compound_response=compound_intent)
            if typed_phase(p):
                need(d["runtimePreFacts"] == state["runtimeFacts"], "startup_intent_facts_binding")
                if phase_clock(p):
                    replay_rules().saved_clock(d["runtimePreClock"])
                    need(j.u64(d["runtimePreClock"]["slot"]) >= state["floor"] and d["runtimePreClock"]["contextSlot"] == d["prestateContextSlot"], "startup_intent_clock_binding")
                    if state["runtimeClock"] is not None: need(int(d["runtimePreClock"]["unixTimestamp"]) >= int(state["runtimeClock"]["unixTimestamp"]), "startup_preclock_regression")
            if p["kind"] == "INITIALIZER" and 4 <= p["initializerIndex"] <= 7: need(state["preVote"] is not None, "intent_causal_pre_vote")
            if p["kind"] == "INITIALIZER" and p["initializerIndex"] == 8: need(state["postVote"] is not None, "intent_post_vote")
            if full: verify_gate_clock(state,p,d["runtimePreClock"])
            state["intents"][phase] = d; state["reserved"] += j.u64(d["feeLamports"])
            state["handoff"] = None
            state["floor"] = max(state["floor"], j.u64(d["prestateContextSlot"]))
        elif kind in ("UNKNOWN", "SUBMITTED"):
            need(d["id"] in state["intents"] and d["id"] not in state["terminal"], "unknown_without_intent")
            if kind == "SUBMITTED":
                need(d["signature"] == state["intents"][d["id"]]["signature"], "submitted_signature_changed")
                if full:successful_evidence(j,[Path(state["intents"][d["id"]]["evidencePath"])],send_payloads[d["id"]])
        elif kind in ("FINALIZED", "FAILED"):
            phase = d["id"]; need(phase in state["intents"] and phase not in state["terminal"] and phase == ids[state["cursor"]], "terminal_order")
            slot = j.u64(d["slot"]); need(slot > state["lastSlot"], "global_slot_not_later")
            need(d["feeLamports"] == state["intents"][phase]["feeLamports"], "terminal_fee_changed")
            need(d["wireSha256"] == state["intents"][phase]["wireSha256"] and d["signatureHex"] == state["intents"][phase]["signatureHex"], "terminal_packet_changed")
            need(state["attempt"] is not None and state["attempt"]["stage"] == "RECOVER" and d["evidencePath"] == str(Path(state["attempt"]["evidencePath"]) / "collection"), "terminal_recovery_binding")
            need(j.u64(d["contextSlot"]) >= max(state["floor"], slot) if kind == "FINALIZED" else j.u64(d["contextSlot"]) == slot, "terminal_context_stale")
            p = m["phases"][state["cursor"]]
            if full:
                adapter = startup_manifest(_replay_source_sha=source_sha)
                if p["kind"] != "UPLOAD_WRITE": p = adapter.effective(api,c,state,p)
            if kind == "FINALIZED" and full:
                post_clock=d["runtimePostClock"]; replay_rules().saved_clock(post_clock)
                need(post_clock["contextSlot"] == d["contextSlot"], "startup_terminal_clock_binding")
                replay_rules().window(state["intents"][phase]["runtimePreClock"],post_clock,slot)
                snapshot_accounts=verify_full_snapshot(c,j,state,p,d) if adapter.large_key() in adapter.required_keys(p) else None
                need(d.get("fullSnapshot") is None if snapshot_accounts is None else True, "startup_unexpected_full_snapshot")
                coverage=adapter.coverage(api,c,state,p,"POST",state["prepared"][phase]["reviewed"],d["runtimeFacts"],post_clock,snapshot_accounts,full_snapshot=d.get("fullSnapshot"))
                need(d["coverage"] == coverage, "startup_terminal_coverage_binding")
                paths=[Path(d["evidencePath"])];payload=3*account_payload(coverage["accountDigests"],adapter.proof_keys(p))
                if snapshot_accounts is not None:
                    paths.append(paths[0].parent/"full-state");payload+=2*account_payload(coverage["accountDigests"])
                successful_evidence(j,paths,payload)
                state["runtimeClock"] = post_clock
            if kind == "FINALIZED" and typed_phase(p):
                rules = replay_rules(); post_clock = d["runtimePostClock"]
                if phase_clock(p):
                    rules.saved_clock(post_clock)
                    need(post_clock["contextSlot"] == d["contextSlot"], "startup_terminal_clock_binding")
                rules.transition(state,p,d["runtimeFacts"],slot,state["intents"][phase]["runtimePreClock"],post_clock)
                state["runtimeFacts"] = copy.deepcopy(d["runtimeFacts"])
                if post_clock is not None: state["runtimeClock"] = post_clock
            if kind == "FINALIZED" and full and not typed_phase(p):
                need(d["runtimeFacts"] == state["runtimeFacts"], "startup_runtime_facts_changed")
            state["actual"] += j.u64(d["feeLamports"]); state["terminal"][phase] = d
            state["lastSlot"] = slot; state["floor"] = max(state["floor"], j.u64(d["contextSlot"]), slot)
            if kind == "FINALIZED": state["cursor"] += 1
            else: state["failed"] = True
        elif kind in ("PRE_VOTE", "POST_VOTE"):
            need(state["seal"] is not None, "gate_without_seal")
            p = m["phases"][state["cursor"]]
            need(p["kind"] == "INITIALIZER" and p["initializerIndex"] == (4 if kind == "PRE_VOTE" else 8) and d["id"] == p["id"] and p["id"] not in state["intents"] and d["sealDataSha256"] == state["seal"]["sealDataSha256"] and j.u64(d["contextSlot"]) >= state["floor"], "gate_causality_or_binding")
            need(state["attempt"] is not None and state["attempt"]["stage"] == kind and d["evidencePath"] == str(Path(state["attempt"]["evidencePath"]) / "readback") and d["transportMode"] == state["attempt"]["transportMode"], "gate_attempt_binding")
            if full:state["voteGateClock"][kind]=verify_gate(j,state,kind,d)
            state["preVote" if kind == "PRE_VOTE" else "postVote"] = d
            state["voteGateAt"][kind]=record["digest"]
            state["floor"] = max(state["floor"], j.u64(d["contextSlot"]))
        else: raise Stop("operator_record_kind")
        need(state["reserved"] <= j.u64(m["proposedFeeBudget"]["feeCapLamports"]) and state["actual"] <= j.u64(m["proposedFeeBudget"]["feeCapLamports"]), "global_fee_cap")
    if rules_module is not None:
        # Independently reload trusted globals and verify the rules file plus
        # all primary sources again before returning any successfully replayed
        # state. Never trust the reused module's mutable provenance dictionary.
        fresh_rules()
    return state


def authorization(j, state, raw, pinned, *, allow_expired=False, require_signer=False, _replay_source_sha=None):
    need(type(raw) is bytes and pinned is not None and sha(raw) == j.digest(pinned), "launcher_authorization_required")
    a = j.parse(raw)
    v2=a.get("schema")=="piv1-launcher-release-authorization-v2"
    fields(a, ("schema", "manifestSha256", "baseParentTail", "operatorSourceSha256", "expiresUnixTimestamp", "scope", "feeBudgetNamespace")+(("signerBackendSha256","signerProfileSha256") if v2 else ()))
    need(a["schema"] in ("piv1-launcher-release-authorization-v1","piv1-launcher-release-authorization-v2") and a["manifestSha256"] == state["manifestSha256"] and a["baseParentTail"] == state["manifest"]["parentTail"] and a["operatorSourceSha256"] == (sha if _replay_source_sha is None else _replay_source_sha[0])(Path(__file__).read_bytes()) and a["scope"] == "EXACT_BOUNDED_TESTNET_RELEASE", "launcher_authorization_binding")
    need(a["feeBudgetNamespace"] == state["manifest"]["proposedFeeBudget"]["namespace"], "launcher_fee_namespace_binding")
    expires = j.u64(a["expiresUnixTimestamp"])
    need(allow_expired or expires > int(time.time()), "launcher_authorization_expired")
    if v2:
        j.digest(a["signerProfileSha256"])
        need(encode(a)==raw and a["signerBackendSha256"]==SIGNER_BACKEND_SHA,"signer_authorization_binding")
        signer_backend_source(_replay_source_sha=_replay_source_sha)
    need(not require_signer or v2,"signer_authorization_v2_required")
    return pinned


def phase_template(state, phase):
    if phase["kind"] != "INITIALIZER": return phase["reviewed"]
    need(state["seal"] is not None, "initializer_seal_required")
    return state["seal"]["reviewedPackets"][phase["initializerIndex"]]


def check_signed(j, frozen, phase, prepared, signed, observation=None):
    deadline = time.monotonic() + 20
    packet = {"signedWireBase64": signed["wireBase64"], "reviewed": prepared["reviewed"]}
    p = {"id": phase["id"], "packet": packet, "lifetime": prepared["lifetime"], "feeQuoteLamports": prepared["feeQuoteLamports"], "feeCapLamports": phase["feeCapLamports"], "expectedPoststate": observation["poststate"]["accounts"] if observation and observation["poststate"] else {}}
    trace = []
    for event in ("PREPARED", "SUBMISSION_START_RECORDED") + (("FINALIZED" if observation["status"]["err"] is None else "FAILED",) if observation else ()):
        entry = {"previousDigest": None, "phase": phase["id"], "state": event, "packetSha256": signed["wireSha256"], "signatureHex": signed["signatureHex"], "frozen": frozen, "observation": observation if event in ("FINALIZED", "FAILED") else None}
        # entryDigest uses canonical JSON without a trailing newline.
        if trace: entry["previousDigest"] = sha(encode(trace[-1]).rstrip(b"\n"))
        trace.append(entry)
    need(time.monotonic() < deadline, "signed_checker_deadline")
    report = j._node_request([{"schema": "piv1-startup-public-check-v1", "frozen": frozen, "cumulativeFeeCapLamports": phase["feeCapLamports"], "phases": [p], "trace": trace}])[0]
    need(report.get("reason") in ("PENDING_OR_UNKNOWN_NO_REPLACEMENT", "ALL_DECLARED_PHASES_FINALIZED_WITH_EXPECTED_POSTSTATE", "FINALIZED_FAILURE"), "signed_packet_or_receipt_rejected:" + str(report.get("reason")))
    need(time.monotonic() < deadline, "signed_checker_deadline")
    return report


def frozen(state):
    seal = state["seal"]
    return {"packageSha256": state["manifest"]["packageSha256"], "payloadSha256": seal["nativeSha256"] if seal else state["manifest"]["initializerPreManifestSha256"], "anchorUnixTimestamp": seal["anchorUnixTimestamp"] if seal else None, "transactionIndex": seal["transactionIndex"] if seal else "0"}


class Rpc:
    def __init__(self, c, j, store, transport):
        self.c=c; self.j=j; self.store=store
        self.transport = bounded_transport(c, transport); self.count=0
    def query(self, calls):
        need(1 <= len(calls) <= 5 and self.count < 6, "rpc_budget")
        self.count += 1; prefix=f"rpc-{self.count:02}"
        queries=[{"jsonrpc":"2.0","id":i+1,"method":m,"params":p} for i,(m,p) in enumerate(calls)]
        raw=encode(queries); self.store.write(prefix+".request.json",raw)
        self.store.write(prefix+".started.json",encode({"startedUnix":str(int(time.time()))}))
        try: status, headers, body = self.transport(raw)
        except Exception as exc:
            if isinstance(exc, self.c.TransportFailure) and exc.partial:
                self.store.write(prefix+".partial-response.bin", exc.partial)
            self.store.write(prefix+".failure.json", encode({"reason": "transport_failure"}))
            raise
        need(type(body) is bytes and len(body) <= self.c.MAX_JSON,"rpc_response_bound")
        self.store.write(prefix+".response.bin",body); self.store.write(prefix+".http.json",encode({"status":status,"headers":headers}))
        need(type(status) is int and status == 200,"rpc_http_status")
        replies=self.c.strict_json(body); need(type(replies) is list and len(replies)==len(queries),"rpc_batch_shape")
        by={}
        for reply in replies:
            need(type(reply) is dict and reply.get("jsonrpc")=="2.0" and type(reply.get("id")) is int and reply["id"] not in by and 1<=reply["id"]<=len(queries) and "result" in reply and "error" not in reply,"rpc_reply")
            by[reply["id"]]=reply["result"]
        return [by[i+1] for i in range(len(queries))]


def public_rpc_transport(c, raw):
    """One production request; kill/reap before failure, never retry or resend."""
    deadline=time.monotonic()+RPC_SECONDS
    need(not any(os.environ.get(key) for key in c.PROXIES),"proxy_environment_unsupported")
    need(not any(os.environ.get(key) for key in RPC_TLS_ENV),"rpc_tls_environment_unsupported")
    need(c.MAX_JSON==RPC_BODY_BOUND and type(raw) is bytes and 0<len(raw)<=RPC_BODY_BOUND,"rpc_child_request_bound")
    repo=Path(os.environ.get("PIV1_PUBLIC_CHECK_REPO",Path(__file__).resolve().parent.parent)).resolve()
    path=repo/"tools/collect_startup_public_evidence.py"
    need(not path.is_symlink() and sha(path.read_bytes())==RPC_COLLECTOR_SHA,"rpc_collector_source_pin")
    process=None;selector=selectors.DefaultSelector();streams=();output=bytearray();accepted=False
    try:
        need(time.monotonic()<deadline,"rpc_child_deadline")
        process=subprocess.Popen([str(Path(sys.executable).resolve()),"-I","-S","-c",RPC_TRANSPORT_SCRIPT],cwd=repo,
            env={"PIV1_PUBLIC_CHECK_REPO":str(repo)},stdin=subprocess.PIPE,stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,close_fds=True,start_new_session=True)
        streams=(process.stdin,process.stdout);position=0
        for stream in streams:
            os.set_blocking(stream.fileno(),False);selector.register(stream,selectors.EVENT_WRITE if stream is process.stdin else selectors.EVENT_READ)
        while selector.get_map():
            remaining=deadline-time.monotonic();need(remaining>0,"rpc_child_deadline")
            for event,_ in selector.select(min(.25,remaining)):
                need(time.monotonic()<deadline,"rpc_child_deadline")
                stream=event.fileobj
                if stream is process.stdin:
                    position+=os.write(stream.fileno(),raw[position:position+65536])
                    if position==len(raw):selector.unregister(stream);stream.close()
                else:
                    chunk=os.read(stream.fileno(),65536)
                    if not chunk:selector.unregister(stream)
                    else:output.extend(chunk);need(len(output)<=RPC_OUTPUT_BOUND,"rpc_child_output_bound")
        frame=json.loads(bytes(output));need(encode(frame)==bytes(output),"rpc_child_canonical_frame")
        need(type(frame) is dict and frame.get("schema")=="piv1-startup-rpc-child-v1","rpc_child_frame")
        if frame.get("kind")=="TRANSPORT_FAILURE":
            fields(frame,("schema","kind","partialBase64"))
            partial=c.b64(frame["partialBase64"],RPC_BODY_BOUND+1,True)
        else:
            fields(frame,("schema","kind","status","headers","bodyBase64"))
            need(frame["kind"]=="RESPONSE" and type(frame["status"]) is int and 100<=frame["status"]<=599,"rpc_child_response")
            headers=frame["headers"]
            need(type(headers) is list and len(encode(headers))<=16384 and all(type(pair) is list and len(pair)==2 and all(type(value) is str for value in pair) for pair in headers),"rpc_child_headers")
            body=c.b64(frame["bodyBase64"],RPC_BODY_BOUND,True)
        remaining=deadline-time.monotonic();need(remaining>0,"rpc_child_deadline")
        need(process.wait(timeout=remaining)==0,"rpc_child_exit")
        need(not path.is_symlink() and sha(path.read_bytes())==RPC_COLLECTOR_SHA,"rpc_collector_source_pin")
        need(time.monotonic()<deadline,"rpc_child_deadline")
        if frame["kind"]=="TRANSPORT_FAILURE":raise c.TransportFailure("rpc_child_transport_failure",partial)
        accepted=True;return frame["status"],headers,body
    except c.TransportFailure:
        raise
    except Exception:
        # Partial stdout is framing, never raw HTTP evidence. Only a complete
        # validated TRANSPORT_FAILURE frame can carry genuine partial bytes.
        raise c.TransportFailure("rpc_child_deadline_or_failure") from None
    finally:
        try:
            if process is not None and not accepted:
                try:
                    if process.returncode is None:
                        try:os.killpg(process.pid,signal.SIGKILL)
                        except ProcessLookupError:pass
                        except OSError:
                            try:process.kill()
                            except OSError:pass
                finally:
                    # Even a failed group signal/fallback must not release a
                    # failure while the direct transport worker is unreaped.
                    process.wait()
        finally:
            selector.close()
            for stream in streams:
                if not stream.closed:stream.close()


def native_collection(mode, request, output, *, parent_path=None, seal_sha256=None, locked_operator_path=None):
    """Unchanged native collector in a private group; incomplete output is never adopted."""
    deadline=time.monotonic()+COLLECTION_SECONDS
    r,c,_,j=helpers()
    need(not any(os.environ.get(key) for key in c.PROXIES),"proxy_environment_unsupported")
    need(not any(os.environ.get(key) for key in RPC_TLS_ENV),"rpc_tls_environment_unsupported")
    need(mode in ("RECOVER","GOVERNANCE"),"collection_mode")
    output=Path(os.path.abspath(output));need(not output.exists() and not output.is_symlink(),"collection_output_exists_no_adoption")
    if mode=="RECOVER":
        need(parent_path is None and seal_sha256 is None and request["includeExpiry"] is False,"collection_recovery_scope")
    else:
        parent_path=Path(os.path.abspath(parent_path));locked_operator_path=Path(os.path.abspath(locked_operator_path))
        need(parent_path!=locked_operator_path and locked_operator_path not in parent_path.parents,"collection_parent_is_operator")
        j.digest(seal_sha256)
    value={"schema":"piv1-startup-collection-request-v1","mode":mode,"request":request,"outputPath":str(output),"parentPath":str(parent_path) if parent_path is not None else None,"sealSha256":seal_sha256}
    raw=encode(value);need(len(raw)<=COLLECTION_PIPE_BOUND,"collection_input_bound")
    repo=Path(os.environ.get("PIV1_PUBLIC_CHECK_REPO",Path(__file__).resolve().parent.parent)).resolve()
    process=None;selector=selectors.DefaultSelector();streams=();result=bytearray();group_cleanup_ok=True
    try:
        need(time.monotonic()<deadline,"collection_deadline")
        process=subprocess.Popen([str(Path(sys.executable).resolve()),"-I","-S","-c",COLLECTION_SCRIPT],cwd=repo,
            env={"PIV1_PUBLIC_CHECK_REPO":str(repo)},stdin=subprocess.PIPE,stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,close_fds=True,start_new_session=True)
        streams=(process.stdin,process.stdout);position=0
        for stream in streams:
            os.set_blocking(stream.fileno(),False);selector.register(stream,selectors.EVENT_WRITE if stream is process.stdin else selectors.EVENT_READ)
        while selector.get_map():
            remaining=deadline-time.monotonic();need(remaining>0,"collection_deadline")
            for event,_ in selector.select(min(.25,remaining)):
                need(time.monotonic()<deadline,"collection_deadline");stream=event.fileobj
                if stream is process.stdin:
                    position+=os.write(stream.fileno(),raw[position:position+65536])
                    if position==len(raw):selector.unregister(stream);stream.close()
                else:
                    chunk=os.read(stream.fileno(),65536)
                    if not chunk:selector.unregister(stream)
                    else:result.extend(chunk);need(len(result)<=COLLECTION_PIPE_BOUND,"collection_output_bound")
        # Retain the leader's waitable identity until the group cleanup decision:
        # a failed collector may have left a live bounded Node descendant.
        while True:
            remaining=deadline-time.monotonic();need(remaining>0,"collection_deadline")
            exited=os.waitid(os.P_PID,process.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
            if exited is not None:break
            selector.select(min(.05,remaining))
        need(exited.si_code==os.CLD_EXITED and exited.si_status==0,"collection_child_exit")
        frame=j.parse(bytes(result));need(encode(frame)==bytes(result),"collection_canonical_frame")
        fields(frame,("schema","mode","requestSha256","outputPath","manifestReceipt"))
        need(frame["schema"]=="piv1-startup-collection-result-v1" and frame["mode"]==mode and frame["requestSha256"]==sha(raw) and frame["outputPath"]==str(output),"collection_frame_binding")
        receipt=frame["manifestReceipt"];fields(receipt,("name","bytes","sha256"))
        need(receipt["name"]=="manifest.json" and type(receipt["bytes"]) is int and 0<receipt["bytes"]<=c.MAX_JSON,"collection_manifest_bound");j.digest(receipt["sha256"])
        fd=j.directory(output)
        try:
            saved=j.read_file(fd,"manifest.json",c.MAX_JSON)
            need(len(saved)==receipt["bytes"] and sha(saved)==receipt["sha256"],"collection_manifest_receipt")
            report=j.parse(saved);need(encode(report)==saved and "manifestReceipt" not in report,"collection_manifest_canonical")
            need(report["schema"]==(c.SCHEMA if mode=="RECOVER" else r.SCHEMA) and report["transportMode"]=="FIXED_ORIGIN_TLS" and report["endpoint"]==c.ENDPOINT and report["expectedGenesis"]==c.GENESIS,"collection_report_binding")
            artifacts=report["artifacts"];need(type(artifacts) is list and len(artifacts)<=64,"collection_artifact_bound")
            names=set();total=len(saved)
            for artifact in artifacts:
                fields(artifact,("name","bytes","sha256"));name=artifact["name"]
                need(type(name) is str and re.fullmatch(r"[a-z0-9.-]+",name) and name not in names and name!="manifest.json","collection_artifact_name")
                need(type(artifact["bytes"]) is int and 0<=artifact["bytes"]<=c.MAX_JSON,"collection_artifact_size")
                data=j.read_file(fd,name,c.MAX_JSON);j.digest(artifact["sha256"])
                need(len(data)==artifact["bytes"] and sha(data)==artifact["sha256"],"collection_artifact_hash")
                names.add(name);total+=len(data)
                need(total<=(ACTION_EVIDENCE if mode=="RECOVER" else r.EVIDENCE_BUDGET),"collection_evidence_bound")
            need(set(os.listdir(fd))==names|{"manifest.json"} and "input.json" in names,"collection_artifact_namespace")
            need(j.read_file(fd,"input.json",COLLECTION_PIPE_BOUND)==encode(request),"collection_saved_input_binding")
        finally:os.close(fd)
        if mode=="RECOVER":
            packet=c.packet(request["wireBase64"])
            need(report["checkerSource"]==c.checker_pin() and report["packetSha256"]==sha(packet["wire"]) and report["messageSha256"]==packet["messageSha256"] and report["signature"]==packet["signature"] and report["signatureHex"]==packet["signatureHex"],"collection_original_signature_binding")
        else:
            need(report["parentTail"]==request["expectedTail"] and report["sealDataSha256"]==seal_sha256 and report["sourcePins"]==r.PINS and j.u64(report["minimumContextSlot"])>=j.u64(request["minimumContextSlot"]),"collection_governance_binding")
            if report.get("storageReport") is not None:need(report["storageReport"]["stage"]==request["stage"] and report["storageReport"]["parentTail"]==request["expectedTail"],"collection_governance_stage")
        helpers()  # Independent fresh source pins before accepting any report.
        need(time.monotonic()<deadline,"collection_deadline")
    except Exception:
        raise Stop("native_collection_incomplete_no_adoption") from None
    finally:
        try:
            if process is not None:
                try:
                    # The leader has not been reaped, so its group identity
                    # cannot be reused while orphaned Node children are killed.
                    try:os.killpg(process.pid,signal.SIGKILL)
                    except ProcessLookupError:pass
                    except OSError:
                        group_cleanup_ok=False
                        try:process.kill()
                        except OSError:pass
                finally:process.wait()
        finally:
            selector.close()
            for stream in streams:
                if not stream.closed:stream.close()
    # A leader-only fallback cannot establish that Node descendants stopped.
    # Never accept evidence when group cleanup was only best effort.
    need(group_cleanup_ok and time.monotonic()<deadline,"native_collection_incomplete_no_adoption")
    return {**report,"manifestReceipt":receipt}


def bounded_transport(c, transport):
    selected = (lambda raw:public_rpc_transport(c,raw)) if transport is None else transport
    def request(raw):
        try: response = selected(raw)
        except c.TransportFailure as exc:
            need(not exc.partial or len(exc.partial) <= c.MAX_JSON + 1, "partial_response_bound")
            raise
        need(type(response) is tuple and len(response) == 3, "transport_response")
        status, headers, body = response
        need(type(body) is bytes and len(body) <= c.MAX_JSON, "rpc_response_bound")
        need(len(encode(headers)) <= 16384, "rpc_headers_bound")
        return status, headers, body
    return request


def evidence_plan(manifest, ref):
    """Finite selected-profile byte slots; project lengths, never future state."""
    roles=ref["roles"];baseline=ref["baseline"];large=roles["stakePoolProgramData"]
    slots={}
    def sizes(completed):
        result={key:value["dataBytes"] if value["exists"] else 0 for key,value in baseline.items()}
        result[roles["loaderBuffer"]]=894589 if 1<=completed<938 else 0
        if completed>=938:result[roles["pivProgram"]]=36;result[roles["programData"]]=894597
        result[roles["governanceBuffer1"]]=1559 if 940<=completed<942 else 0
        if completed>=942:result[roles["transaction1"]]=1548
        if completed>=943:result[roles["proposal1"]]=646
        if completed>=948:result.update(zip(ref["targetAddresses"],ref["targetSizes"]))
        return {key:4*((size+2)//3) for key,size in result.items()}
    write_keys=[roles[name] for name in ("g0","h2","pivProgram","programData","loaderBuffer","loaderProgram","computeBudget","rent","clock","instructions")]
    gate_payload=3*sum(4*((size+2)//3) for size in (baseline[roles["multisig"]]["dataBytes"],1548,646))+2*(4*((baseline[roles["clock"]]["dataBytes"]+2)//3))
    for index,phase in enumerate(manifest["phases"]):
        pid=phase["id"];keys=write_keys if phase["kind"]=="UPLOAD_WRITE" else list(baseline)
        pre,post=sizes(index),sizes(index+1)
        slots[("QUOTE",pid)]=SUCCESS_SERIALIZATION
        slots[("SEND",pid)]=sum(pre[key] for key in keys)+SUCCESS_SERIALIZATION
        slots[("RECOVER",pid)]=3*sum(post[key] for key in keys if key!=large)+(2*sum(post[key] for key in keys) if large in keys else 0)+SUCCESS_SERIALIZATION
        stage=vote_stage(phase)
        if stage:slots[(stage,pid)]=gate_payload+SUCCESS_SERIALIZATION
    return slots


def remaining_reservations(manifest, records, slots):
    """Spend baseline slots once from validated history, never latest-only state."""
    spent=set();finished=set();gates=set();sealed=False
    for record in records:
        kind,data=record["kind"],record["data"]
        if kind in ("PREPARE","PREPARE_SIGN_ATTEMPT") and type(data) is dict and "quote" in data:
            spent.update(("QUOTE",entry["id"]) for entry in data["entries"])
        elif kind in ("INTENT","IMPORT_SEND_ATTEMPT_INTENT"):
            # No future prepare or resend is legal after this original intent.
            intent=data["intent"] if kind=="IMPORT_SEND_ATTEMPT_INTENT" else data
            spent.update((("QUOTE",intent["id"]),("SEND",intent["id"])))
        elif kind=="FINALIZED":
            spent.add(("RECOVER",data["id"]));finished.add(data["id"])
        elif kind in ("PRE_VOTE","POST_VOTE"):
            spent.add((kind,data["id"]));gates.add(kind)
        elif kind=="BIND_SEAL":sealed=True
    remaining=sum(value for key,value in slots.items() if key not in spent)
    # Eight normal native records per uncompleted phase, two per outstanding
    # gate and one seal. Partial current phases are deliberately overreserved.
    records_needed=8*(len(manifest["phases"])-len(finished))+2*(2-len(gates))+int(not sealed)
    return remaining,records_needed


def evidence_store(c, j, operator, label, state, transport):
    # Full raw Buffer responses remain intact. Bound total retained evidence and
    # reserve one action envelope plus the inherited 2-GiB free-space floor.
    root=evidence_root(j,operator,state,label)
    output = root / f"{len(operator.chain):08}-{label.lower()}"
    operator.append("ATTEMPT", {"id": state["manifest"]["phases"][state["cursor"]]["id"], "stage": label, "evidencePath": str(output), "transportMode": "OFFLINE_INJECTED_TEST" if transport is not None else "FIXED_ORIGIN_TLS"})
    # Failure after ATTEMPT keeps its immutable identity; later recovery creates
    # a new attempt. Existing/partial output is never reused or adopted.
    return c.Store(output)


def evidence_root(j,operator,state,label):
    if label=="RECOVER":need(len(operator.chain)+2<=MAX_RECORDS,"recovery_record_headroom")
    j.space(operator.fd, ACTION_EVIDENCE)
    root = operator.path.with_name(operator.path.name + ".evidence")
    if not root.exists():
        fd = j.directory(root, True); os.close(fd)
    else:
        fd = j.directory(root); os.close(fd)
    total = count = 0
    for directory, dirs, files in os.walk(root, followlinks=False):
        for name in dirs:
            meta = (Path(directory) / name).lstat()
            need(stat.S_ISDIR(meta.st_mode) and meta.st_uid == os.getuid() and stat.S_IMODE(meta.st_mode) == 0o700, "evidence_directory_identity")
        for name in files:
            meta = (Path(directory) / name).lstat(); j.secure_meta(meta)
            total += meta.st_size; count += 1
            need(total <= MAX_EVIDENCE and count <= MAX_RECORDS * 64, "evidence_total_bound")
    need(total + ACTION_EVIDENCE <= MAX_EVIDENCE, "evidence_total_bound")
    if label!="RECOVER" and full_startup(state["manifest"]):
        remaining,records_needed=remaining_reservations(state["manifest"],operator.chain,evidence_plan(state["manifest"],startup_manifest().oracle()))
        # Count every actual file, including orphan/superseded/UNKNOWN output.
        # This reserves future successes, not a separate fixed failure quota.
        need(total+remaining+ACTION_EVIDENCE<=MAX_EVIDENCE,"future_evidence_reservation_hold")
        need(len(operator.chain)+records_needed<=MAX_RECORDS,"future_record_reservation_hold")
    # Original-signature recovery may spend future forward space under the
    # existing hard/physical caps. Exhaustion holds the intent; INSPECT/replay
    # never run these new admission checks or discard retained evidence.
    return root


def quote_store(c,j,operator,state):
    root=evidence_root(j,operator,state,"QUOTE");prefix=f"{len(operator.chain):08}-prepare-"
    used=[p.name for p in root.iterdir() if p.name.startswith(prefix)]
    need(all(re.fullmatch(re.escape(prefix)+r"[0-9]{2}",name) for name in used) and len(used)<MAX_BATCH,"quote_attempt_bound")
    index=next(i for i in range(MAX_BATCH) if prefix+f"{i:02}" not in used)
    return c.Store(root/(prefix+f"{index:02}"))


def latest_quote(c,state,latest):
    need(type(latest) is dict and type(latest.get("context")) is dict and c.u64(latest["context"].get("slot"))>=state["floor"],"quote_context_stale")
    fields(latest.get("value"),("blockhash","lastValidBlockHeight"));value=latest["value"]
    c.b58decode(value["blockhash"],32);c.u64(value["lastValidBlockHeight"])
    return {"recentBlockhash":value["blockhash"],"lastValidBlockHeight":str(c.u64(value["lastValidBlockHeight"])),"contextSlot":str(c.u64(latest["context"]["slot"]))}


def quote_calls(entries,floor,blockhash):
    calls=[("getFeeForMessage",[p["reviewed"]["messageBase64"],{"commitment":"finalized","minContextSlot":floor}]) for p in entries]
    calls += [("getBlockHeight",[{"commitment":"finalized","minContextSlot":floor}]),("isBlockhashValid",[blockhash,{"commitment":"finalized","minContextSlot":floor}])]
    return [calls[i:i+5] for i in range(0,len(calls),5)]


def quote_values(c,j,state,entries,initial,values,*,_phase_positions=None):
    need(len(initial)==2 and initial[0]==c.GENESIS,"genesis_mismatch")
    original=latest_quote(c,state,initial[1]);floor=max(state["floor"],j.u64(original["contextSlot"]))
    need(len(values)==len(entries)+2,"quote_response_count")
    height=c.u64(values[-2]);valid=values[-1]
    need(type(valid) is dict and valid.get("value") is True and type(valid.get("context")) is dict and c.u64(valid["context"].get("slot"))>=floor,"quote_blockhash_invalid_or_stale")
    highest=max(floor,c.u64(valid["context"]["slot"]))
    for p,result in zip(entries,values[:-2]):
        fields(p,("id","wireBase64","reviewed","feeQuoteLamports","lifetime"))
        need(type(result) is dict and result.get("value") is not None and type(result.get("context")) is dict and c.u64(result["context"].get("slot"))>=floor,"quote_fee_context")
        pid=p["id"]
        phase=state["manifest"]["phases"][_phase_positions[pid]] if _phase_positions is not None and type(pid) is str and pid in _phase_positions else next(x for x in state["manifest"]["phases"] if x["id"]==pid)
        need(str(c.u64(result["value"]))==p["feeQuoteLamports"]==phase["feeCapLamports"],"quote_fee_mismatch")
        highest=max(highest,c.u64(result["context"]["slot"]))
        need(p["lifetime"]=={"recentBlockhash":original["recentBlockhash"],"currentBlockHeight":str(height),"lastValidBlockHeight":original["lastValidBlockHeight"],"minimumRemainingBlockHeights":phase["minimumRemainingBlockHeights"]},"quote_lifetime_binding")
        last=j.u64(original["lastValidBlockHeight"])
        need(height<=last and last-height>=j.u64(phase["minimumRemainingBlockHeights"]),"quote_lifetime_margin")
        wire,review=rebind_unsigned(c,p["wireBase64"],p["reviewed"],original["recentBlockhash"])
        need(wire==p["wireBase64"] and review==p["reviewed"],"quote_message_blockhash_binding")
    return highest


def rebind_unsigned(c,wire,review,blockhash):
    message=base64.b64decode(review["messageBase64"],validate=True);position=4 if message[0]==128 else 3;count=0
    for index in range(3):
        byte=message[position];position+=1;count|=(byte&127)<<(7*index)
        if not byte&128:break
    at=position+32*count;need(at+32<=len(message),"quote_message_blockhash")
    updated=message[:at]+c.b58decode(blockhash,32)+message[at+32:]
    raw=base64.b64decode(wire,validate=True);offset=1+64*raw[0]
    need(raw[0]==len(review["requiredSigners"]) and 1<=raw[0]<=6 and offset<len(raw) and raw[offset:]==message and not any(raw[1:offset]),"quote_unsigned_message")
    reviewed={**review,"messageBase64":base64.b64encode(updated).decode(),"messageSha256":sha(updated)}
    need(semantic(reviewed)==semantic(review),"quote_message_semantics")
    return base64.b64encode(raw[:offset]+updated).decode(),reviewed


def captured_authorization(state,ids,pin):
    for pid in ids:
        quote=state["quoteByPhase"].get(pid)
        if quote is not None:need(quote["authorizationSha256"]==pin,"quote_authorization_changed")


def capture_quote(c,j,op,state,packets,reviews,authorization_raw,authorization_pin,transport):
    store=quote_store(c,j,op,state);rpc=Rpc(c,j,store,transport)
    try:
        authorization(j,state,authorization_raw,authorization_pin)
        initial=rpc.query([("getGenesisHash",[]),("getLatestBlockhash",[{"commitment":"finalized","minContextSlot":state["floor"]}])])
        need(initial[0]==c.GENESIS,"genesis_mismatch");original=latest_quote(c,state,initial[1]);entries=[]
        for p,review in zip(packets,reviews):
            wire,reviewed=rebind_unsigned(c,p["wireBase64"],review,original["recentBlockhash"])
            phase=next(x for x in state["manifest"]["phases"] if x["id"]==p["id"])
            entries.append({"id":p["id"],"wireBase64":wire,"reviewed":reviewed,"feeQuoteLamports":phase["feeCapLamports"],"lifetime":{"recentBlockhash":original["recentBlockhash"],"lastValidBlockHeight":original["lastValidBlockHeight"],"minimumRemainingBlockHeights":phase["minimumRemainingBlockHeights"]}})
        floor=max(state["floor"],j.u64(original["contextSlot"]));values=[]
        for batch in quote_calls(entries,floor,original["recentBlockhash"]):values.extend(rpc.query(batch))
        height=str(c.u64(values[-2]))
        for p in entries:p["lifetime"]["currentBlockHeight"]=height
        quote_values(c,j,state,entries,initial,values)
        authorization(j,state,authorization_raw,authorization_pin)
        binding={"schema":"piv1-startup-lifetime-quote-v1","path":str(store.path),"operatorSequence":str(len(op.chain)),"operatorTail":op.tail(),"operatorSourceSha256":sha(Path(__file__).read_bytes()),"baseParentPath":state["baseParentPath"],"baseParentTail":state["manifest"]["parentTail"],"manifestSha256":state["manifestSha256"],"authorization":j.parse(authorization_raw),"authorizationSha256":authorization_pin,"minimumContextSlot":str(state["floor"]),"transportMode":"OFFLINE_INJECTED_TEST" if transport is not None else "FIXED_ORIGIN_TLS","orderedPreparedSha256":[sha(encode(p)) for p in entries]}
        manifest=store.write("manifest.json",encode({"binding":binding,"artifacts":store.artifacts.copy()}))
        successful_evidence(j,[store.path],0)
        return entries,{**binding,"manifestSha256Receipt":manifest["sha256"]}
    finally:store.close()


def verify_quote(c,j,state,entries,quote,record,*,_replay_source_sha=None,_phase_positions=None):
    fields(quote,("schema","path","operatorSequence","operatorTail","operatorSourceSha256","baseParentPath","baseParentTail","manifestSha256","authorization","authorizationSha256","minimumContextSlot","transportMode","orderedPreparedSha256","manifestSha256Receipt"))
    root=journal_path(state["baseParentPath"]);root=root.with_name(root.name+".evidence");path=Path(quote["path"])
    need(path.parent==root and re.fullmatch(re.escape(record["sequence"].zfill(8))+r"-prepare-[0-9]{2}",path.name),"quote_path_binding")
    need(quote["schema"]=="piv1-startup-lifetime-quote-v1" and quote["operatorSequence"]==record["sequence"] and quote["operatorTail"]==record["previousDigest"] and quote["operatorSourceSha256"]==(sha if _replay_source_sha is None else _replay_source_sha[0])(Path(__file__).read_bytes()),"quote_source_or_predecessor_binding")
    for entry in entries:
        phase=state["manifest"]["phases"][_phase_positions[entry["id"]]] if _phase_positions is not None and type(entry) is dict and type(entry.get("id")) is str and entry["id"] in _phase_positions else next(p for p in state["manifest"]["phases"] if p["id"]==entry["id"])
        stage=vote_stage(phase)
        if stage:need(phase==state["manifest"]["phases"][state["cursor"]] and quote["operatorTail"]==state["voteGateAt"].get(stage),"quote_gate_predecessor")
    need(quote["baseParentPath"]==state["baseParentPath"] and quote["baseParentTail"]==state["manifest"]["parentTail"] and quote["manifestSha256"]==state["manifestSha256"] and quote["minimumContextSlot"]==str(state["floor"]),"quote_parent_binding")
    need(quote["orderedPreparedSha256"]==[sha(encode(p)) for p in entries],"quote_prepared_order_binding")
    need(quote["transportMode"] in ("FIXED_ORIGIN_TLS","OFFLINE_INJECTED_TEST") and state["transportMode"] in (None,quote["transportMode"]),"transport_mode_changed")
    raw_auth=encode(quote["authorization"]);authorization(j,state,raw_auth,quote["authorizationSha256"],allow_expired=True,_replay_source_sha=_replay_source_sha)
    fd=j.directory(path)
    try:
        successful_evidence(j,[path],0)
        raw=j.read_file(fd,"manifest.json",c.MAX_JSON);need(sha(raw)==j.digest(quote["manifestSha256Receipt"]),"quote_manifest_hash")
        manifest=j.parse(raw);fields(manifest,("binding","artifacts"));need(manifest["binding"]=={k:v for k,v in quote.items() if k!="manifestSha256Receipt"},"quote_manifest_binding")
        queries=1+(len(entries)+6)//5;names={f"rpc-{i:02}.{s}" for i in range(1,queries+1) for s in ("request.json","started.json","response.bin","http.json")}
        artifacts=manifest["artifacts"];need(type(artifacts) is list and len(artifacts)==len(names) and {a.get("name") for a in artifacts}==names,"quote_artifact_count")
        saved={}
        for artifact in artifacts:
            fields(artifact,("name","bytes","sha256"));value=j.read_file(fd,artifact["name"],c.MAX_JSON)
            need(len(value)==artifact["bytes"] and sha(value)==artifact["sha256"],"quote_artifact_hash");saved[artifact["name"]]=value
        def response(index,calls):
            prefix=f"rpc-{index:02}";expected=[{"jsonrpc":"2.0","id":i+1,"method":m,"params":p} for i,(m,p) in enumerate(calls)]
            need(saved[prefix+".request.json"]==encode(expected),"quote_raw_request_binding")
            meta=j.parse(saved[prefix+".http.json"]);fields(meta,("status","headers"));need(type(meta["status"]) is int and meta["status"]==200 and len(encode(meta["headers"]))<=16384,"quote_http_binding")
            replies=c.strict_json(saved[prefix+".response.bin"])
            need(type(replies) is list and len(replies)==len(calls),"quote_raw_response_count");by={}
            for reply in replies:
                need(type(reply) is dict and reply.get("jsonrpc")=="2.0" and type(reply.get("id")) is int and 1<=reply["id"]<=len(calls) and reply["id"] not in by and "result" in reply and "error" not in reply,"quote_raw_response_binding");by[reply["id"]]=reply["result"]
            return [by[i+1] for i in range(len(calls))]
        initial=response(1,[("getGenesisHash",[]),("getLatestBlockhash",[{"commitment":"finalized","minContextSlot":state["floor"]}])])
        original=latest_quote(c,state,initial[1]);floor=max(state["floor"],j.u64(original["contextSlot"]));values=[]
        for index,calls in enumerate(quote_calls(entries,floor,original["recentBlockhash"]),2):values.extend(response(index,calls))
        return quote_values(c,j,state,entries,initial,values,_phase_positions=_phase_positions)
    finally:os.close(fd)


def collect_full_snapshot(c,j,state,phase,signed,proof,output,transport):
    """One complete selected57-account snapshot after original signature proof."""
    adapter=startup_manifest();keys=adapter.required_keys(phase);observation=proof["observation"]
    floor=max(state["floor"],j.u64(observation["status"]["slot"]),j.u64(observation["poststate"]["contextSlot"]))
    store=c.Store(output/"full-state");rpc=Rpc(c,j,store,transport)
    try:
        need(rpc.query([("getGenesisHash",[])])[0]==c.GENESIS,"genesis_mismatch")
        response=rpc.query([("getMultipleAccounts",[keys,{"encoding":"base64","commitment":"finalized","minContextSlot":floor}])])[0]
        need(type(response) is dict and c.u64(response["context"]["slot"])>=floor and type(response.get("value")) is list and len(response["value"])==len(keys),"startup_full_snapshot_context_or_keys")
        snapshot={"contextSlot":str(c.u64(response["context"]["slot"])),"accounts":{key:adapter.normalize_account(c,key,value) for key,value in zip(keys,response["value"])}}
        saved=store.write("snapshot.json",encode(snapshot))
        binding={"schema":"piv1-startup-full-state-receipt-v1","path":str(store.path),"phase":phase["id"],"wireSha256":signed["wireSha256"],"signatureHex":signed["signatureHex"],"inclusionSlot":observation["status"]["slot"],"minimumContextSlot":str(floor),"contextSlot":snapshot["contextSlot"],"requiredKeys":keys,"proofSubsetPath":str(output/"collection"),"proofSubsetManifestSha256":proof["manifestReceipt"]["sha256"],"proofSubsetContextSlot":observation["poststate"]["contextSlot"],"transportMode":"OFFLINE_INJECTED_TEST" if transport is not None else "FIXED_ORIGIN_TLS","snapshotSha256":saved["sha256"]}
        manifest=store.write("manifest.json",encode({"binding":binding,"artifacts":store.artifacts.copy()}))
        return snapshot,{**binding,"manifestSha256":manifest["sha256"]}
    finally:store.close()


def verify_preflight_accounts(c,j,state,phase,intent,expected_receipt=None):
    """Reopen only the original full-phase SEND account batch, never later files.

    Its compact receipt pins every raw byte. Exact request/IDs/context/Clock and
    typed account semantics are rederived before journal digests are consumed.
    """
    adapter=startup_manifest();need(phase['kind']!='UPLOAD_WRITE','startup_jito_preflight_phase')
    keys=adapter.required_keys(phase);floor=state['floor']
    calls=[('getMultipleAccounts',[keys,{'encoding':'base64','commitment':'finalized','minContextSlot':floor}])]
    calls += [('getMinimumBalanceForRentExemption',[size,{'commitment':'finalized'}]) for size in adapter.rent_sizes(phase)[:4]]
    queries=[{'jsonrpc':'2.0','id':index+1,'method':method,'params':params} for index,(method,params) in enumerate(calls)]
    fd=j.directory(Path(intent['evidencePath']))
    try:
        saved={suffix:j.read_file(fd,'rpc-02.'+suffix,c.MAX_JSON) for suffix in ('request.json','http.json','response.bin')}
    finally:os.close(fd)
    receipt={name:sha(saved[suffix]) for name,suffix in (('requestSha256','request.json'),('httpSha256','http.json'),('responseSha256','response.bin'))}
    if expected_receipt is not None:need(receipt==expected_receipt,'startup_preflight_raw_hash')
    need(saved['request.json']==encode(queries),'startup_preflight_request_binding')
    meta=j.parse(saved['http.json']);fields(meta,('status','headers'))
    need(type(meta['status']) is int and meta['status']==200 and len(encode(meta['headers']))<=16384,'startup_preflight_http_binding')
    replies=c.strict_json(saved['response.bin'])
    need(type(replies) is list and len(replies)==len(queries),'startup_preflight_response_count');by={}
    for reply in replies:
        need(type(reply) is dict and reply.get('jsonrpc')=='2.0' and type(reply.get('id')) is int and 1<=reply['id']<=len(queries) and reply['id'] not in by and 'result' in reply and 'error' not in reply,'startup_preflight_response_binding')
        by[reply['id']]=reply['result']
    result=by[1]
    need(type(result) is dict and type(result.get('context')) is dict and str(c.u64(result['context']['slot']))==intent['prestateContextSlot'] and c.u64(result['context']['slot'])>=floor and type(result.get('value')) is list and len(result['value'])==len(keys),'startup_preflight_context_binding')
    accounts={key:adapter.normalize_account(c,key,value) for key,value in zip(keys,result['value'])}
    clock=state_rules().clock(c,accounts[adapter.oracle()['roles']['clock']],result['context']['slot'],floor)
    need(clock==intent['runtimePreClock'],'startup_preflight_clock_binding')
    adapter.jito_digests(c,phase,accounts,clock)
    return accounts,receipt


def verify_full_snapshot(c,j,state,phase,terminal):
    receipt=terminal.get("fullSnapshot");need(type(receipt) is dict,"startup_full_snapshot_receipt_required")
    path=Path(state["attempt"]["evidencePath"])/"full-state";proof_path=Path(state["attempt"]["evidencePath"])/"collection"
    fields(receipt,("schema","path","phase","wireSha256","signatureHex","inclusionSlot","minimumContextSlot","contextSlot","requiredKeys","proofSubsetPath","proofSubsetManifestSha256","proofSubsetContextSlot","transportMode","snapshotSha256","manifestSha256"))
    need(receipt["schema"]=="piv1-startup-full-state-receipt-v1" and receipt["path"]==str(path) and receipt["proofSubsetPath"]==str(proof_path),"startup_full_snapshot_path_binding")
    need(receipt["phase"]==phase["id"] and receipt["wireSha256"]==terminal["wireSha256"] and receipt["signatureHex"]==terminal["signatureHex"] and receipt["inclusionSlot"]==terminal["slot"],"startup_full_snapshot_packet_binding")
    need(receipt["contextSlot"]==terminal["contextSlot"] and receipt["minimumContextSlot"]==str(max(state["floor"],j.u64(terminal["slot"]),j.u64(receipt["proofSubsetContextSlot"]))) and j.u64(receipt["contextSlot"])>=j.u64(receipt["minimumContextSlot"]),"startup_full_snapshot_context_binding")
    need(receipt["requiredKeys"]==startup_manifest().required_keys(phase) and receipt["transportMode"]==state["attempt"]["transportMode"] and receipt["proofSubsetManifestSha256"]==terminal["manifestSha256"],"startup_full_snapshot_proof_binding")
    proof_fd=j.directory(proof_path)
    try:
        proof_raw=j.read_file(proof_fd,"manifest.json",c.MAX_JSON);need(sha(proof_raw)==receipt["proofSubsetManifestSha256"],"startup_full_snapshot_proof_manifest_hash")
        proof=j.parse(proof_raw);observation=proof["observation"]
        need(proof["status"]=="COLLECTED_PUBLIC_EVIDENCE_ONLY" and proof["packetSha256"]==terminal["wireSha256"] and proof["signatureHex"]==terminal["signatureHex"] and observation["status"]["slot"]==terminal["slot"] and observation["poststate"]["contextSlot"]==receipt["proofSubsetContextSlot"] and set(observation["poststate"]["accounts"])==set(startup_manifest().proof_keys(phase)),"startup_full_snapshot_original_proof")
    finally:os.close(proof_fd)
    fd=j.directory(path)
    try:
        raw=j.read_file(fd,"manifest.json",65536);need(sha(raw)==j.digest(receipt["manifestSha256"]),"startup_full_snapshot_manifest_hash")
        manifest=j.parse(raw);fields(manifest,("binding","artifacts"))
        need(manifest["binding"]=={key:value for key,value in receipt.items() if key!="manifestSha256"},"startup_full_snapshot_manifest_binding")
        names={f"rpc-{index:02}.{suffix}" for index in (1,2) for suffix in ("request.json","started.json","response.bin","http.json")}|{"snapshot.json"}
        need(type(manifest["artifacts"]) is list and len(manifest["artifacts"])==9 and {a.get("name") for a in manifest["artifacts"]}==names,"startup_full_snapshot_artifact_count")
        saved_raw={}
        for artifact in manifest["artifacts"]:
            fields(artifact,("name","bytes","sha256"));value=j.read_file(fd,artifact["name"],c.MAX_JSON)
            need(len(value)==artifact["bytes"] and sha(value)==artifact["sha256"],"startup_full_snapshot_artifact_hash")
            saved_raw[artifact['name']]=value
        saved=j.read_file(fd,"snapshot.json",c.MAX_JSON);need(sha(saved)==receipt["snapshotSha256"],"startup_full_snapshot_hash")
        snapshot=j.parse(saved);fields(snapshot,("contextSlot","accounts"))
        need(snapshot["contextSlot"]==terminal["contextSlot"] and set(snapshot["accounts"])==set(receipt["requiredKeys"]),"startup_full_snapshot_saved_context")
        # Reconnect the saved complete snapshot to its actual retained raw RPC
        # response before dynamic Jito digests or runtime facts are consumed.
        query=[{'jsonrpc':'2.0','id':1,'method':'getMultipleAccounts','params':[receipt['requiredKeys'],{'encoding':'base64','commitment':'finalized','minContextSlot':j.u64(receipt['minimumContextSlot'])}]}]
        need(saved_raw['rpc-02.request.json']==encode(query),'startup_full_snapshot_raw_request')
        http=j.parse(saved_raw['rpc-02.http.json']);fields(http,('status','headers'))
        need(type(http['status']) is int and http['status']==200 and len(encode(http['headers']))<=16384,'startup_full_snapshot_raw_http')
        replies=c.strict_json(saved_raw['rpc-02.response.bin'])
        need(type(replies) is list and len(replies)==1,'startup_full_snapshot_raw_response')
        reply=replies[0]
        need(type(reply) is dict and reply.get('jsonrpc')=='2.0' and type(reply.get('id')) is int and reply['id']==1 and 'result' in reply and 'error' not in reply,'startup_full_snapshot_raw_response')
        result=reply['result']
        need(type(result) is dict and type(result.get('context')) is dict and str(c.u64(result['context']['slot']))==snapshot['contextSlot'] and type(result.get('value')) is list and len(result['value'])==len(receipt['requiredKeys']),'startup_full_snapshot_raw_context')
        normalized={key:startup_manifest().normalize_account(c,key,value) for key,value in zip(receipt['requiredKeys'],result['value'])}
        need(normalized==snapshot['accounts'],'startup_full_snapshot_raw_account_binding')
        return snapshot["accounts"]
    finally:os.close(fd)


def vote_stage(phase):
    if phase["kind"]=="INITIALIZER":return {4:"PRE_VOTE",8:"POST_VOTE"}.get(phase["initializerIndex"])
    return None


def captured_gate(state,phase):
    stage=vote_stage(phase);quote=state["quoteByPhase"].get(phase["id"])
    if not stage or quote is None:return False
    need(quote["operatorTail"]==state["voteGateAt"].get(stage),"quote_gate_predecessor")
    return True


def verify_gate_clock(state,phase,clock):
    stage=vote_stage(phase)
    if not stage:return
    saved=state["voteGateClock"].get(stage)
    need(saved is not None and clock is not None and int(clock["unixTimestamp"])>=int(saved["unixTimestamp"]),"gate_clock_regression")
    if stage=="POST_VOTE":
        elapsed=int(clock["unixTimestamp"])-int(saved["approvedTimestamp"])
        need(-(1<<63)<=elapsed<(1<<63) and elapsed>=int(saved["configuredDelaySeconds"]),"gate_timelock_arithmetic")


def gate_seal(j,state):
    """Read the exact already-bound terminal seal without rerunning Node checks."""
    seal=state["seal"];path=Path(seal["parentPath"]);operator=journal_path(state["baseParentPath"])
    need(path!=operator and operator not in path.parents,"gate_parent_is_operator")
    fd=j.directory(path);lock=None
    try:
        lock=os.open("lock",os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK,dir_fd=fd)
        meta=os.fstat(lock);j.secure_meta(meta);fcntl.flock(lock,fcntl.LOCK_EX)
        def stable():
            current=os.stat("lock",dir_fd=fd,follow_symlinks=False)
            need((current.st_dev,current.st_ino,current.st_ctime_ns)==(meta.st_dev,meta.st_ino,meta.st_ctime_ns) and j.read_file(fd,"lock",128)==j.LOCK_BYTES,"gate_parent_lock")
        stable();chain,_=j.records(fd)
        need(chain and chain[0]["lockIdentity"]=={"device":str(meta.st_dev),"inode":str(meta.st_ino),"ctimeNs":str(meta.st_ctime_ns)} and j.tail(chain)==seal["parentTail"] and chain[-1]["kind"]=="SEAL","gate_parent_tail")
        actual=chain[-1]["data"];binding=seal["stateBinding"]
        need(sha(encode(actual))==seal["sealDataSha256"] and actual["unsignedInitializerPlan"]==binding["plan"] and actual["postManifest"]==binding["postManifest"] and actual["compactSha256"]==sha(bytes.fromhex(binding["plan"]["compact"]["hex"])) and actual["nativeSha256"]==seal["nativeSha256"],"gate_parent_seal_binding")
        stable();return actual
    finally:
        if lock is not None:os.close(lock)
        os.close(fd)


def verify_gate(j,state,stage,data):
    """Reread one source-bound gate; pure checks only, never collect or launch Node."""
    r,c,s,_=helpers();seal=state["seal"];binding=seal["stateBinding"]
    ref=startup_manifest().oracle();roles=ref["roles"];plan=binding["plan"];g=binding["postManifest"]["initializer"]["governance"]
    need(g==ref["preManifest"]["initializer"]["governance"],"gate_governance_profile")
    addresses={"multisig":roles["multisig"],"transaction":roles["transaction1"],"proposal":roles["proposal1"],"vault":roles["vault0"]}
    need(all(plan[name]==address for name,address in addresses.items()),"gate_sealed_addresses")
    # The immutable bump comes from the source-pinned baseline, never the
    # collected current account or a report's claimed PDA derivation.
    baseline=base64.b64decode(ref["multisigBase64"],validate=True)
    need(len(baseline)>95 and baseline[94] in (0,1),"gate_multisig_bump_layout")
    offset=95+32*baseline[94];need(offset<len(baseline),"gate_multisig_bump_layout")
    bumps={"multisig":baseline[offset],"transaction":ref["transactionBump"],"proposal":ref["proposalBump"],"vault":ref["vaultBump"]}
    need(bumps["proposal"]==binding["proposalBump"],"gate_sealed_proposal_bump")
    pdas={name:{"address":address,"bump":bumps[name]} for name,address in addresses.items()}
    root=journal_path(state["baseParentPath"]);root=root.with_name(root.name+".evidence")
    path=Path(data["evidencePath"])
    need(path.name=="readback" and path.parent.parent==root and re.fullmatch(r"[0-9]{8}-"+stage.lower(),path.parent.name),"gate_evidence_path")
    fd=j.directory(path)
    try:
        raw=j.read_file(fd,"manifest.json",c.MAX_JSON);need(sha(raw)==j.digest(data["manifestSha256"]),"gate_manifest_hash")
        report=j.parse(raw);need(encode(report)==raw and "manifestReceipt" not in report,"gate_manifest_canonical")
        artifacts=report["artifacts"];need(type(artifacts) is list and len(artifacts)==15,"gate_artifact_count")
        names={"input.json","parent-reference.json","storage-snapshot.json","clock-account.json","storage-request.json","storage-report.json","clock-evidence.json"}
        names|={f"http-{i:02}.{suffix}" for i in (1,2) for suffix in ("request.json","started.json","response.bin","http.json")}
        saved={};total=len(raw)
        for artifact in artifacts:
            fields(artifact,("name","bytes","sha256"));name=artifact["name"]
            need(name in names and name not in saved and type(artifact["bytes"]) is int and 0<=artifact["bytes"]<=c.MAX_JSON,"gate_artifact_identity")
            value=j.read_file(fd,name,c.MAX_JSON);need(len(value)==artifact["bytes"] and sha(value)==j.digest(artifact["sha256"]),"gate_artifact_hash")
            saved[name]=value;total+=len(value);need(total<=r.EVIDENCE_BUDGET,"gate_evidence_bound")
        need(set(saved)==names and set(os.listdir(fd))==names|{"manifest.json"},"gate_artifact_namespace")
    finally:os.close(fd)
    request={"schema":r.SCHEMA,"expectedTail":seal["parentTail"],"stage":stage,"minimumContextSlot":str(state["floor"])}
    need(saved["input.json"]==encode(request),"gate_request_binding")
    need(report["schema"]==r.SCHEMA and report["status"]=="VERIFIED_COLLECTED_STORAGE_AND_CLOCK_EVIDENCE_ONLY" and report["endpoint"]==c.ENDPOINT and report["expectedGenesis"]==c.GENESIS and report["genesisMatched"] is True,"gate_report_status")
    need(report["parentTail"]==seal["parentTail"] and report["sealDataSha256"]==data["sealDataSha256"]==seal["sealDataSha256"] and report["postManifestSha256"]==seal["postManifestSha256"] and report["inheritedLedgerReference"]==state["manifest"]["historicalLedger"] and report["sourcePins"]==r.PINS,"gate_report_binding")
    need(report["transportMode"]==data["transportMode"]==state["attempt"]["transportMode"] and report["transportMode"] in ("FIXED_ORIGIN_TLS","OFFLINE_INJECTED_TEST"),"gate_transport_binding")
    need(saved["parent-reference.json"]==encode({key:report[key] for key in ("parentTail","sealDataSha256","postManifestSha256","inheritedLedgerReference","sourcePins")}),"gate_parent_reference")
    actual_seal=gate_seal(j,state)
    floor=r.minimum(request,actual_seal);need(report["minimumContextSlot"]==str(floor),"gate_minimum_context")
    keys=[plan[name] for name in ("multisig","transaction","proposal")]+[r.CLOCK]
    need(report["accountKeys"]==keys,"gate_account_keys")
    calls=(("getGenesisHash",[]),("getMultipleAccounts",[keys,{"encoding":"base64","commitment":"finalized","minContextSlot":floor}]))
    replies=[]
    for index,(method,params) in enumerate(calls,1):
        prefix=f"http-{index:02}";need(saved[prefix+".request.json"]==encode([{"jsonrpc":"2.0","id":index,"method":method,"params":params}]),"gate_raw_request")
        meta=j.parse(saved[prefix+".http.json"]);need(type(meta.get("status")) is int and meta["status"]==200 and len(encode(meta["headers"]))<=16384,"gate_http_binding")
        replies.append(r.rpc_result(c,saved[prefix+".response.bin"],index))
    need(replies[0]==c.GENESIS,"gate_genesis")
    result=replies[1];context=c.u64(result["context"]["slot"]);values=result["value"]
    need(context>=floor and type(values) is list and len(values)==4,"gate_raw_accounts")
    for value in values:
        if value is not None:
            need(type(value) is dict and type(value.get("data")) is list and len(value["data"])==2,"gate_raw_account_data")
            c.b64(value["data"][0],4096,True)
    accounts={key:c.normalize_account(value) for key,value in zip(keys,values)}
    snapshot={"contextSlot":str(context),"accounts":{key:accounts[key] for key in keys[:3]}}
    need(saved["storage-snapshot.json"]==encode(snapshot) and report["snapshotSha256"]==data["snapshotSha256"]==sha(encode(snapshot)) and report["snapshotContextSlot"]==data["contextSlot"]==str(context),"gate_snapshot_binding")
    need(saved["clock-account.json"]==encode({"address":r.CLOCK,"contextSlot":str(context),**accounts[r.CLOCK]}),"gate_raw_clock")
    evidence=s.validate_snapshot(snapshot,actual_seal,pdas,stage)
    storage=j.parse(saved["storage-report.json"])
    need(storage==report["storageReport"] and storage["status"]=="VERIFIED_SUPPLIED_INITIALIZER_STORAGE_ONLY" and storage["stage"]==stage and storage["parentTail"]==seal["parentTail"] and storage["sealDataSha256"]==seal["sealDataSha256"] and storage["postManifestSha256"]==seal["postManifestSha256"] and storage["inheritedLedgerReference"]==state["manifest"]["historicalLedger"] and storage["pdaDerivations"]==pdas and storage["evidence"]==evidence and storage["snapshotSha256"]==sha(encode(snapshot)) and storage["snapshotContextSlot"]==str(context),"gate_storage_binding")
    need(saved["storage-request.json"]==encode({"schema":s.SCHEMA,"expectedTail":seal["parentTail"],"stage":stage,"snapshot":snapshot}),"gate_storage_request")
    timing=r.clock_evidence(accounts[r.CLOCK],context,floor,stage,storage,g["timeLockSeconds"])
    need(saved["clock-evidence.json"]==encode(timing) and report["clockEvidence"]==timing,"gate_clock_binding")
    sizes=[ref["baseline"][roles["multisig"]]["dataBytes"],plan["accountSizes"]["transaction"],plan["accountSizes"]["proposal"]]
    payload=3*sum(4*((size+2)//3) for size in sizes)+2*(4*((40+2)//3))
    successful_evidence(j,[path],payload)
    helpers()  # Fresh source pins after secure raw/pure validation as well.
    return timing


def gate(r, c, j, operator, state, stage, transport):
    seal=state["seal"]; need(seal is not None,"seal_required")
    store=evidence_store(c,j,operator,stage,state,transport); output=store.path; store.close()
    # Collector requires an exclusive new directory, so use a deterministic child.
    request={"schema":r.SCHEMA,"expectedTail":seal["parentTail"],"stage":stage,"minimumContextSlot":str(state["floor"])}
    report=(r.collect(request,seal["parentPath"],output/"readback",_transport=bounded_transport(c,transport)) if transport is not None else
        native_collection("GOVERNANCE",request,output/"readback",parent_path=seal["parentPath"],seal_sha256=seal["sealDataSha256"],locked_operator_path=operator.path))
    need(report["status"]=="VERIFIED_COLLECTED_STORAGE_AND_CLOCK_EVIDENCE_ONLY" and report["sealDataSha256"]==seal["sealDataSha256"],"storage_clock_gate_failed:"+str(report.get("reason")))
    need(j.u64(report["snapshotContextSlot"])>=state["floor"],"gate_context_stale")
    return {"id":state["manifest"]["phases"][state["cursor"]]["id"],"contextSlot":report["snapshotContextSlot"],"sealDataSha256":report["sealDataSha256"],"snapshotSha256":report["snapshotSha256"],"evidencePath":str(output/"readback"),"manifestSha256":report["manifestReceipt"]["sha256"],"transportMode":report["transportMode"]}


def operate(request, base, *, authorization_raw=None, authorization_sha256=None, live=False, live_sign=False, output_mode="REPORT", _transport=None, _signer=None):
    fields(request,("schema","action","expectedTail","data"))
    if request["schema"] == CANDIDATE_CONTROLLER_SCHEMA:
        need(__name__ != "__main__", "candidate_controller_import_only")
        need(live is False and live_sign is False and authorization_raw is None and authorization_sha256 is None
            and _transport is None and _signer is None and type(output_mode) is str and output_mode == "REPORT", "candidate_controller_offline_only")
        return _candidate_controller_operate(request,base)
    need(request["schema"]==SCHEMA,"schema")
    action=request["action"]
    need(type(output_mode) is str and output_mode in ("REPORT","COMMIT_RECEIPT"),"output_mode")
    need(output_mode == "REPORT" or action in ("PREPARE","SIGN"),"commit_receipt_action")
    if action is None: return {"status":"INCOMPLETE","authorized":False,"liveReady":False}
    need(action in ("CREATE","INSPECT","PREPARE","HANDOFF","SIGN","IMPORT","BIND_SEAL","SEND","RECOVER","CAPTURE_SIGN_SEND"),"action")
    need(_transport is None or callable(_transport),"transport_callable")
    need(_signer is None or callable(_signer),"signer_callable")
    compound = action=="CAPTURE_SIGN_SEND"
    if compound:
        need(live_sign is True,"live_sign_not_enabled")
        need(live is True,"live_transport_not_enabled")
        need((_transport is None)==(_signer is None),"compound_transport_signer_mode")
    r,c,s,j=helpers(); base=Path(os.path.abspath(base)); op=Journal(j,base,action=="CREATE")
    compound_store=None
    try:
        committed=None
        need(request["expectedTail"]==op.tail(),"tail_compare_and_swap")
        if action=="CREATE":
            m=request["data"]; parent=inspect_parent(j,base,m["parentTail"])
            manifest_shape(c,j,m,parent)
            # Read inherited cap from actual base records rather than a report label.
            fd=j.directory(base)
            try:
                chain,_=j.records(fd); need(j.tail(chain) == m["parentTail"], "base_changed_during_create")
                inherited_cap=j.reconstruct(chain)[0]["input"]["cumulativeFeeCapLamports"]
            finally: os.close(fd)
            need(m["historicalLedger"]["cumulativeFeeCapLamports"]==inherited_cap,"inherited_cap_changed")
            floor=last=0
            for record in chain:
                obs=record["data"].get("observation") if record["kind"]=="EVENT" else None
                if obs:
                    last=max(last,j.u64(obs["status"]["slot"])); floor=max(floor,last)
                    if obs["poststate"]: floor=max(floor,j.u64(obs["poststate"]["contextSlot"]))
            op.append("CREATE",{"manifest":m,"baseParentPath":str(base),"historicalLedger":copy.deepcopy(m["historicalLedger"]),"contextFloor":str(floor),"lastFinalizedSlot":str(last)})
        else:
            state=replay(j,op.chain); m=state["manifest"]; data=request["data"]
            need(op.chain[0]["data"]["baseParentPath"] == str(base), "operator_base_path_changed")
            need(not state["failed"] or action=="INSPECT","finalized_failure_halted")
            if state["signerAttempt"] is not None: need(action in ("INSPECT","IMPORT"),"signer_attempt_unresolved")
            cursor=state["cursor"]
            if action=="INSPECT": need(data is None,"inspect_data")
            elif action=="BIND_SEAL":
                fields(data,("parentPath","parentTail")); need(state["seal"] is None,"seal_already_bound")
                pending=[p for p in m["phases"][:cursor] if p["kind"]=="INITIALIZER"]
                need(not pending and all(p["kind"] not in ("INITIALIZER","HEARTBEAT") for p in m["phases"][:cursor]),"seal_boundary_order")
                need(cursor<len(m["phases"]) and m["phases"][cursor]["kind"]=="INITIALIZER","seal_at_initializer_boundary")
                parent_path=Path(os.path.abspath(data["parentPath"])); need(parent_path!=op.path and op.path not in parent_path.parents,"seal_parent_path")
                report=inspect_parent(j,parent_path,data["parentTail"]); seal=report.get("initializerSeal")
                need(seal is not None and seal["preManifestSha256"]==m["initializerPreManifestSha256"],"seal_profile_binding")
                need(report["initializerProfile"]["parentTail"] == seal["parentTail"] == m["parentTail"], "seal_original_parent_binding")
                need(report["initializerProfile"]["parentLedger"]==m["historicalLedger"],"seal_historical_ledger_rebind")
                need(j.u64(seal["clock"]["contextSlot"])>=state["floor"] and j.u64(seal["decodedClock"]["slot"])>=state["floor"],"seal_before_prerequisite_poststate")
                need(r.tls_profile(seal) or _transport is not None,"official_initializer_profile")
                plan=seal["unsignedInitializerPlan"]
                lookups=[{"address":plan["alt"]["address"],"addresses":plan["alt"]["addresses"]}]
                packets=[]
                for p in plan["packets"]:
                    raw=base64.b64decode(p["wireBase64"],validate=True); offset=1+raw[0]*64
                    packets.append({"wireBase64":p["wireBase64"],"lookupTables":lookups if raw[offset]==128 else []})
                state_binding = None
                if full_startup(m) or any(typed_phase(p) for p in m["phases"]):
                    derived = describe(j,packets,state_plan=plan,governance=seal["postManifest"]["initializer"]["governance"])
                    reviewed = derived.pop("packets")
                    state_binding = {**derived,"plan":plan,"postManifest":seal["postManifest"]}
                else: reviewed=describe(j,packets)
                op.append("BIND_SEAL",{"parentPath":str(parent_path),"parentTail":data["parentTail"],"baseParentTail":report["initializerProfile"]["parentTail"],"sealDataSha256":sha(encode(seal)),"postManifestSha256":seal["postManifestSha256"],"nativeSha256":seal["nativeSha256"],"anchorUnixTimestamp":seal["decodedClock"]["unixTimestamp"],"transactionIndex":seal["postManifest"]["initializer"]["governance"]["transactionIndex"],"reviewedPackets":reviewed,"transportMode":"OFFLINE_INJECTED_TEST" if _transport is not None else "FIXED_ORIGIN_TLS","stateBinding":state_binding})
            elif action in ("PREPARE","CAPTURE_SIGN_SEND"):
                if compound:
                    fields(data,("packet","signerProfilePath"))
                    need(full_startup(m) and cursor<len(m["phases"]) and state["handoff"] is None,"compound_current_phase_required")
                    packet=data["packet"];private_path=data["signerProfilePath"]
                    fields(packet,("id","wireBase64"));pid=m["phases"][cursor]["id"]
                    need(packet["id"]==pid,"compound_current_phase_required")
                    need(pid not in state["prepared"] and pid not in state["signed"] and pid not in state["intents"],"compound_phase_already_prepared")
                    need(type(private_path) is str and 1<=len(private_path)<=4096 and os.path.isabs(private_path) and os.path.normpath(private_path)==private_path and "\0" not in private_path,"signer_profile_path")
                    need(Path(private_path)!=Path(__file__).resolve().parent.parent and Path(__file__).resolve().parent.parent not in Path(private_path).parents,"signer_profile_in_repository")
                    authorization(j,state,authorization_raw,authorization_sha256,require_signer=True)
                    data={"captureLifetime":True,"packets":[packet]}
                capture=type(data) is dict;captured=None
                if capture:
                    fields(data,("captureLifetime","packets"));need(data["captureLifetime"] is True,"quote_capture_required")
                    need(full_startup(m),"full_startup_required_for_quote")
                    need(_transport is not None or live,"live_transport_not_enabled")
                    need(state["transportMode"] in (None,"OFFLINE_INJECTED_TEST" if _transport is not None else "FIXED_ORIGIN_TLS"),"transport_mode_changed")
                    authorization_pin=authorization(j,state,authorization_raw,authorization_sha256)
                    need(encode(j.parse(authorization_raw))==authorization_raw,"quote_authorization_canonical")
                    data=data["packets"];need(type(data) is list and 1<=len(data)<=23,"quote_batch_bound")
                need(type(data) is list and 1<=len(data)<=MAX_BATCH,"batch_bound")
                need(not any(p["id"] in state["intents"] and p["id"] not in state["terminal"] for p in m["phases"]),"unresolved_intent_blocks_prepare")
                need(type(data[0]) is dict and data[0].get("id") in [p["id"] for p in m["phases"]], "prepare_phase")
                start=next(i for i,p in enumerate(m["phases"]) if p["id"] == data[0]["id"])
                need(cursor <= start and start+len(data)<=len(m["phases"]) and all(p["id"] in state["prepared"] for p in m["phases"][cursor:start]),"batch_phase_bound_or_gap")
                if capture:
                    need(all(not vote_stage(p) or start+i==cursor for i,p in enumerate(m["phases"][start:start+len(data)])),"quote_crosses_vote_boundary")
                reviewed=describe(j,[{"wireBase64":p["wireBase64"],"lookupTables":phase_template(state,m["phases"][start+i])["lookupTables"]} for i,p in enumerate(data)])
                if capture:
                    for i,(p,review) in enumerate(zip(data,reviewed)):
                        fields(p,("id","wireBase64"));phase=m["phases"][start+i]
                        need(p["id"]==phase["id"] and semantic(review)==semantic(phase_template(state,phase)),"prepared_semantics_changed")
                    stage=vote_stage(m["phases"][start])
                    if stage:
                        evidence=gate(r,c,j,op,state,stage,_transport)
                        op.append(stage,evidence);state=replay(j,op.chain)
                    quoted,captured=capture_quote(c,j,op,state,data,reviewed,authorization_raw,authorization_pin,_transport)
                    reviewed=[p["reviewed"] for p in quoted];data=[{k:v for k,v in p.items() if k!="reviewed"} for p in quoted]
                entries=[]
                for i,(p,review) in enumerate(zip(data,reviewed)):
                    fields(p,("id","wireBase64","lifetime","feeQuoteLamports")); phase=m["phases"][start+i]
                    need(p["id"]==phase["id"] and semantic(review)==semantic(phase_template(state,phase)),"prepared_semantics_changed")
                    fields(p["lifetime"],("recentBlockhash","currentBlockHeight","lastValidBlockHeight","minimumRemainingBlockHeights"))
                    need(p["lifetime"]["minimumRemainingBlockHeights"]==phase["minimumRemainingBlockHeights"],"margin_changed")
                    msg=base64.b64decode(review["messageBase64"],validate=True); pos=4 if msg[0] == 128 else 3; count=0
                    for n in range(3):
                        b=msg[pos]; pos+=1; count|=(b&127) << (7*n)
                        if not b&128: break
                    blockhash=c.b58encode(msg[pos+count*32:pos+count*32+32])
                    need(p["lifetime"]["recentBlockhash"] == blockhash, "prepared_blockhash_binding")
                    now=j.u64(p["lifetime"]["currentBlockHeight"]); last=j.u64(p["lifetime"]["lastValidBlockHeight"])
                    need(now<=last and last-now>=j.u64(phase["minimumRemainingBlockHeights"]),"prepared_lifetime_margin")
                    need(j.u64(p["feeQuoteLamports"])<=j.u64(phase["feeCapLamports"]),"phase_fee_cap")
                    if full_startup(m): need(p["feeQuoteLamports"] == phase["feeCapLamports"], "startup_exact_fee_profile")
                    entries.append({**p,"reviewed":review})
                if capture:authorization(j,state,authorization_raw,authorization_sha256)
                if not compound:
                    committed=op.append("PREPARE",{"entries":entries,"quote":captured} if capture else entries)
                else:
                    # Build public request data locally; append still freshly
                    # replays the whole prefix and both facts before any write.
                    previous=op.tail();pid=entries[0]["id"]
                    signing_state={**state,"prepared":{**state["prepared"],pid:entries[0]},
                        "preparedAt":{**state["preparedAt"],pid:compound_prepared_digest(previous,entries,captured)},
                        "quoteByPhase":{**state["quoteByPhase"],pid:captured}}
                    signing=signing_request(c,signing_state,previous,[pid],authorization_pin)
                    del signing_state
                    a=j.parse(authorization_raw)
                    attempt={"request":signing,"authorization":a,"signerBackendSha256":SIGNER_BACKEND_SHA,"signerProfileSha256":a["signerProfileSha256"],"mode":"OFFLINE_INJECTED_TEST" if _signer is not None else "LOCAL_TESTNET_BACKEND"}
                    # Untrusted output creation precedes both fresh A barriers.
                    # Its future C sequence follows A; partial/orphan paths are never reused.
                    root=evidence_root(j,op,state,"SEND");compound_output=root/f"{len(op.chain)+1:08}-send"
                    compound_store=c.Store(compound_output)
                    op.append("PREPARE_SIGN_ATTEMPT",{"entries":entries,"quote":captured,"attempt":attempt})
                    state=replay(j,op.chain)  # Fresh after durability, before keys.
                    need(state["signerAttempt"]==attempt,"signer_attempt_changed")
                    original_attempt_raw=encode(attempt);original_prepared_raw=encode(state["prepared"][pid]);original_a_raw=encode(op.chain[-1])
                    payload={"authorizationRawBase64":base64.b64encode(authorization_raw).decode(),"authorizationSha256":authorization_pin,"request":signing,
                        "packets":[copy.deepcopy(state["prepared"][pid])],"signerProfilePath":private_path}
                    authorization(j,state,authorization_raw,authorization_sha256,require_signer=True)
                    try:response=_signer(copy.deepcopy(payload)) if _signer is not None else invoke_signer(j,payload)
                    except Exception:raise Stop("signer_attempt_unresolved") from None
                    # Retain a complete bounded public reply before any later refusal.
                    response_raw=encode(response);need(len(response_raw)<=262144,"signer_output_bound")
                    try:compound_store.write("signer-response.json",response_raw)
                    except Exception as exc:raise Stop("compound_response_uncertain_no_retry_or_adoption") from exc
                    response=j.parse(response_raw)
                    response_receipt={"responseSha256":sha(response_raw),"signerAttemptSha256":sha(original_attempt_raw)}
                    authorization(j,state,authorization_raw,authorization_sha256,require_signer=True)
                    op.stable();imported=import_response(c,j,state,response,authorization_pin)
                    # SEND's independent crypto gate still precedes durable
                    # ATTEMPT, now atomically committed with this exact IMPORT.
                    phase=active_phase(c,state,m["phases"][cursor])
                    check_signed(j,frozen(state),phase,state["prepared"][pid],imported["entries"][0])
                    if phase["kind"]=="INITIALIZER":
                        idx=phase["initializerIndex"]
                        if idx in (4,8):need(captured_gate(state,phase),"quote_gate_predecessor")
                        if 4<=idx<=7:need(state["preVote"] is not None,"causal_pre_vote_required")
                    authorization(j,state,authorization_raw,authorization_sha256,require_signer=True)
                    send_attempt={"id":pid,"stage":"SEND","evidencePath":str(compound_output),"transportMode":"OFFLINE_INJECTED_TEST" if _transport is not None else "FIXED_ORIGIN_TLS"}
                    need(compound_response(j,send_attempt,response_receipt)==response,"compound_response_binding")
                    op.stable()
                    # This B is ephemeral: one fresh full traversal after R fsync
                    # and both checker callbacks, before any public collection.
                    body={"schema":SCHEMA,"sequence":str(len(op.chain)),"previousDigest":op.tail(),"lockIdentity":op.identity,
                        "kind":"IMPORT_SEND_ATTEMPT","data":{"import":imported,"attempt":send_attempt}}
                    state=replay(j,op.chain+[{**body,"digest":sha(encode(body))}])
                    need(state["signerAttempts"].get(signing["requestId"])==j.parse(original_attempt_raw),"signer_attempt_changed")
                    data={"id":pid}

            elif action=="HANDOFF":
                authorization_pin=authorization(j,state,authorization_raw,authorization_sha256)
                need(not any(pid not in state["terminal"] for pid in state["intents"]), "unresolved_intent_blocks_handoff")
                signing=signing_request(c,state,op.tail(),data,authorization_pin)
                captured_authorization(state,data,authorization_pin)
                op.append("HANDOFF",signing)
                return {"status":"LAUNCHER_BOUND_SIGNING_HANDOFF_ONLY","tail":op.tail(),"request":signing,"packets":[copy.deepcopy(state["prepared"][pid]) for pid in data],"signerInvoked":False,"signerControlProven":False,"founderAuthorizationCryptographicallyVerified":False,"liveReady":False}
            elif action=="SIGN":
                need(full_startup(m),"full_startup_required_for_signer")
                need(live_sign is True,"live_sign_not_enabled")
                authorization_pin=authorization(j,state,authorization_raw,authorization_sha256,require_signer=True)
                need(not any(pid not in state["terminal"] for pid in state["intents"]),"unresolved_intent_blocks_signer")
                fields(data,("ids","signerProfilePath"))
                private_path=data["signerProfilePath"]
                need(type(private_path) is str and 1<=len(private_path)<=4096 and os.path.isabs(private_path) and os.path.normpath(private_path)==private_path and "\0" not in private_path,"signer_profile_path")
                need(Path(private_path)!=Path(__file__).resolve().parent.parent and Path(__file__).resolve().parent.parent not in Path(private_path).parents,"signer_profile_in_repository")
                signing=signing_request(c,state,op.tail(),data["ids"],authorization_pin);a=j.parse(authorization_raw)
                captured_authorization(state,data["ids"],authorization_pin)
                mode="OFFLINE_INJECTED_TEST" if _signer is not None else "LOCAL_TESTNET_BACKEND"
                attempt={"request":signing,"authorization":a,"signerBackendSha256":SIGNER_BACKEND_SHA,"signerProfileSha256":a["signerProfileSha256"],"mode":mode}
                # This combined request+attempt is the one durable signing
                # boundary. No retry after a crash/timeout or lost response.
                op.append("SIGN_ATTEMPT",attempt);state=replay(j,op.chain)
                payload={"authorizationRawBase64":base64.b64encode(authorization_raw).decode(),"authorizationSha256":authorization_pin,"request":signing,
                    "packets":[copy.deepcopy(state["prepared"][pid]) for pid in data["ids"]],"signerProfilePath":private_path}
                authorization(j,state,authorization_raw,authorization_sha256,require_signer=True)
                try:response=_signer(copy.deepcopy(payload)) if _signer is not None else invoke_signer(j,payload)
                except Exception:raise Stop("signer_attempt_unresolved") from None
                authorization(j,state,authorization_raw,authorization_sha256,require_signer=True)
                op.stable()
                imported=import_response(c,j,state,response,authorization_pin)
                authorization(j,state,authorization_raw,authorization_sha256,require_signer=True)
                committed=op.append("IMPORT",imported,expected_signer_attempt=attempt)
            elif action=="IMPORT":
                need(not any(pid not in state["terminal"] for pid in state["intents"]), "unresolved_intent_blocks_import")
                bound=type(data) is dict
                if bound:
                    authorization_pin=authorization(j,state,authorization_raw,authorization_sha256)
                    imported=import_response(c,j,state,data,authorization_pin)
                    authorization(j,state,authorization_raw,authorization_sha256)
                    op.append("IMPORT",imported)
                else:
                    need(not full_startup(m),"signing_response_required")
                    need(state["handoff"] is None,"signing_response_required")
                    need(type(data) is list and 1<=len(data)<=MAX_BATCH,"import_batch")
                    entries=[]
                    for p in data:
                        fields(p,("id","wireBase64")); pid=p["id"]
                        need(pid in state["prepared"] and pid not in state["signed"] and pid not in state["intents"],"signed_phase")
                        need(not any(x["id"] == pid for x in entries), "duplicate_import_phase")
                        signed=signed_entry(c,state,p)
                        need(not any(x["wireSha256"]==signed["wireSha256"] or x["signatureHex"]==signed["signatureHex"] for x in state["signedHistory"]+entries),"global_signed_replay")
                        phase=next(p for p in m["phases"] if p["id"]==pid)
                        check_signed(j,frozen(state),phase,state["prepared"][pid],signed);entries.append(signed)
                    op.append("IMPORT",entries)
            if action in ("SEND","RECOVER","CAPTURE_SIGN_SEND"):
                send_action="SEND" if compound else action
                authorization_pin=authorization(j,state,authorization_raw,authorization_sha256,allow_expired=send_action=="RECOVER")
                need(_transport is not None or live,"live_transport_not_enabled")
                need(_transport is not None or full_startup(m),"full_startup_required_for_live_transport")
                need(state["transportMode"] is None or state["transportMode"] == ("OFFLINE_INJECTED_TEST" if _transport is not None else "FIXED_ORIGIN_TLS"), "transport_mode_changed")
                need(cursor<len(m["phases"]),"workflow_complete"); phase=active_phase(c,state,m["phases"][cursor]); pid=phase["id"]
                need(data=={"id":pid} and pid in state["signed"],"current_signed_phase_required")
                prepared=state["prepared"][pid]; signed=state["signed"][pid]
                if send_action=="SEND":captured_authorization(state,[pid],authorization_pin)
                if not compound:check_signed(j,frozen(state),phase,prepared,signed)
                if send_action=="SEND":
                    need(pid not in state["intents"],"intent_exists_recovery_only")
                    if "requestId" in signed: need(signed["authorizationSha256"] == authorization_pin,"signing_authorization_changed")
                    if full_startup(m): need("requestId" in signed,"signing_response_required")
                    if phase["kind"]=="INITIALIZER":
                        idx=phase["initializerIndex"]
                        if idx==4 and not captured_gate(state,phase):
                            evidence=gate(r,c,j,op,state,"PRE_VOTE",_transport); op.append("PRE_VOTE",evidence); state=replay(j,op.chain)
                        if 4<=idx<=7: need(state["preVote"] is not None,"causal_pre_vote_required")
                        if idx==8 and not captured_gate(state,phase):
                            evidence=gate(r,c,j,op,state,"POST_VOTE",_transport); op.append("POST_VOTE",evidence); state=replay(j,op.chain)
                    store=compound_store if compound else evidence_store(c,j,op,"SEND",state,_transport); rpc=Rpc(c,j,store,_transport)
                    try:
                        life=prepared["lifetime"]
                        rule_module = state_rules() if typed_phase(phase) or full_startup(m) else None
                        clock_key = rule_module.CLOCK if phase_clock(phase) else None
                        account_keys = startup_manifest().required_keys(phase) if full_startup(m) else list(phase["expectedPrestate"])
                        if clock_key and clock_key not in account_keys: account_keys.append(clock_key)
                        need(len(account_keys) <= 64,"startup_clock_account_capacity")
                        identity_calls=[("getGenesisHash",[])] + ([("getVersion",[])] if full_startup(m) else [])
                        sizes=(startup_manifest().rent_sizes(phase) if full_startup(m) else rule_module.rent_sizes(state,phase)) if rule_module else []
                        calls=identity_calls+[
                            ("getBlockHeight",[{"commitment":"finalized","minContextSlot":state["floor"]}]),
                            ("isBlockhashValid",[life["recentBlockhash"],{"commitment":"finalized","minContextSlot":state["floor"]}]),
                            ("getFeeForMessage",[prepared["reviewed"]["messageBase64"],{"commitment":"finalized","minContextSlot":state["floor"]}]),
                            ("getMultipleAccounts",[account_keys,{"encoding":"base64","commitment":"finalized","minContextSlot":state["floor"]}])]
                        calls += [("getMinimumBalanceForRentExemption",[size,{"commitment":"finalized"}]) for size in sizes]
                        # Reserve the final two bounded calls for a post-INTENT
                        # lifetime check and the single actual submission.
                        need(len(calls)<=20,"send_preflight_rpc_budget")
                        replies=[]
                        for start in range(0,len(calls),5):
                            replies += rpc.query(calls[start:start+5])
                            if start==0:
                                need(replies[0]==c.GENESIS,"genesis_mismatch")
                                if full_startup(m): need(replies[1] == startup_manifest().oracle()["version"], "startup_runtime_version_changed")
                        height,valid,quote,accounts=replies[len(identity_calls):len(identity_calls)+4]
                        now=c.u64(height); last=j.u64(life["lastValidBlockHeight"])
                        need(now >= j.u64(life["currentBlockHeight"]), "blockheight_regression")
                        need(now<=last and last-now>=j.u64(phase["minimumRemainingBlockHeights"]),"fresh_lifetime_margin")
                        need(type(valid) is dict and valid.get("value") is True and c.u64(valid["context"]["slot"])>=state["floor"],"blockhash_invalid_or_stale")
                        need(type(quote) is dict and quote.get("value") is not None and c.u64(quote["context"]["slot"])>=state["floor"] and str(c.u64(quote["value"]))==prepared["feeQuoteLamports"],"fresh_fee_quote_mismatch")
                        need(type(accounts) is dict and c.u64(accounts["context"]["slot"])>=state["floor"] and type(accounts.get("value")) is list and len(accounts["value"])==len(account_keys),"prestate_context")
                        actual={key:startup_manifest().normalize_account(c,key,raw) if full_startup(m) else c.normalize_account(raw) for key,raw in zip(account_keys,accounts["value"])}
                        pre_clock = rule_module.clock(c,actual[clock_key],accounts["context"]["slot"],state["floor"]) if clock_key else None
                        if pre_clock and state["runtimeClock"]:
                            need(int(pre_clock["unixTimestamp"]) >= int(state["runtimeClock"]["unixTimestamp"]),"startup_preclock_regression")
                        if full_startup(m):verify_gate_clock(state,phase,pre_clock)
                        pre_facts = verify_states(c,state,phase,"PRE",actual,pre_clock=pre_clock)
                        if rule_module:
                            rents=replies[len(identity_calls)+4:]
                            floors = startup_manifest().oracle()["rents"] if full_startup(m) else {str(k):v for k,v in rule_module.RENT_FLOORS.items()}
                            need(all(str(c.u64(value)) == floors[str(size)] for size,value in zip(sizes,rents)),"startup_fresh_rent_profile_changed")
                        fee=j.u64(prepared["feeQuoteLamports"]); need(state["reserved"]+fee<=j.u64(m["proposedFeeBudget"]["feeCapLamports"]),"global_fee_cap")
                        authorization(j,state,authorization_raw,authorization_sha256)
                        preflight=None
                        if full_startup(m) and phase['kind']!='UPLOAD_WRITE':
                            _,preflight=verify_preflight_accounts(c,j,state,phase,{'evidencePath':str(store.path),'prestateContextSlot':str(accounts['context']['slot']),'runtimePreClock':pre_clock})
                        pre_coverage=startup_manifest().coverage(operator_api(),c,state,phase,"PRE",prepared["reviewed"],pre_facts,pre_clock,actual,preflight_receipt=preflight) if full_startup(m) else None
                        if pre_coverage is not None:
                            send_payload=account_payload(pre_coverage["accountDigests"])
                            successful_evidence(j,[store.path],send_payload,preflight_queries=rpc.count,compound_response=compound)
                        # No send can occur until this complete intent and its directory are fsynced.
                        intent={"id":pid,"preparedSha256":signed["preparedSha256"],"wireSha256":signed["wireSha256"],"signature":signed["signature"],"signatureHex":signed["signatureHex"],"feeLamports":prepared["feeQuoteLamports"],"authorizationSha256":authorization_pin,"prestateContextSlot":str(accounts["context"]["slot"]),"evidencePath":str(store.path),"transportMode":"OFFLINE_INJECTED_TEST" if _transport is not None else "FIXED_ORIGIN_TLS","runtimePreFacts":pre_facts,"runtimePreClock":pre_clock,**({"coverage":pre_coverage} if pre_coverage is not None else {})}
                        if compound:
                            compound_data={"import":imported,"attempt":send_attempt,"response":response_receipt,"intent":intent}
                            compound_data_raw=encode(compound_data)
                            committed=op.append("IMPORT_SEND_ATTEMPT_INTENT",compound_data,expected_signer_attempt=j.parse(original_attempt_raw))
                            prepared,signed,intent,dispatch_authorization,phase=compound_dispatch(c,j,op,committed,compound_data_raw,original_attempt_raw,original_prepared_raw,original_a_raw)
                            # Never carry the ephemeral imported state across C fsync.
                            state=None;life=prepared["lifetime"];last=j.u64(life["lastValidBlockHeight"])
                            pre_coverage=intent.get("coverage")
                            if pre_coverage is not None:send_payload=account_payload(pre_coverage["accountDigests"])
                        else:op.append("INTENT",intent)
                        # Replay/fsync can consume the old quote's remaining
                        # lifetime. A failure here holds the original durable
                        # INTENT for read-only recovery; no replacement/send.
                        post_floor=j.u64(intent["prestateContextSlot"]) if compound else max(state["floor"],c.u64(accounts["context"]["slot"]))
                        post_height,post_valid,latest=rpc.query([
                            ("getBlockHeight",[{"commitment":"finalized","minContextSlot":post_floor}]),
                            ("isBlockhashValid",[life["recentBlockhash"],{"commitment":"finalized","minContextSlot":post_floor}]),
                            ("getLatestBlockhash",[{"commitment":"finalized","minContextSlot":post_floor}])])
                        current=c.u64(post_height)
                        need(current>=max(now,j.u64(life["currentBlockHeight"])),"post_intent_blockheight_regression")
                        need(current<=last and last-current>=j.u64(phase["minimumRemainingBlockHeights"]),"post_intent_lifetime_margin")
                        need(type(post_valid) is dict and post_valid.get("value") is True and c.u64(post_valid["context"]["slot"])>=post_floor,"post_intent_blockhash_invalid_or_stale")
                        latest_quote(c,{"floor":post_floor},latest)
                        if not compound and pid not in state["quoteByPhase"]:
                            need(latest["value"]["blockhash"] == life["recentBlockhash"] and c.u64(latest["value"]["lastValidBlockHeight"]) == last, "fresh_blockhash_lifetime_binding")
                        if pre_coverage is not None:successful_evidence(j,[store.path],send_payload)
                        authorization(j,dispatch_authorization if compound else state,authorization_raw,authorization_sha256)
                        try:
                            result=rpc.query([("sendTransaction",[signed["wireBase64"],{"encoding":"base64","skipPreflight":False,"maxRetries":0,"preflightCommitment":"finalized","minContextSlot":post_floor}])])[0]
                            need(result==signed["signature"],"send_signature_mismatch")
                            if pre_coverage is not None:successful_evidence(j,[store.path],send_payload)
                            op.append("SUBMITTED",{"id":pid,"signature":signed["signature"]})
                        except Exception:
                            op.append("UNKNOWN",{"id":pid,"reason":"SEND_UNCERTAIN_NO_RESEND_OR_REPLACEMENT"})
                    finally:
                        if compound:compound_store=None
                        store.close()
                else:
                    need(pid in state["intents"],"recovery_requires_original_intent")
                    need(authorization_pin == state["intents"][pid]["authorizationSha256"], "recovery_original_authorization_required")
                    need(_transport is not None or state["intents"][pid]["transportMode"] == "FIXED_ORIGIN_TLS", "injected_history_cannot_become_live")
                    store=evidence_store(c,j,op,"RECOVER",state,_transport); output=store.path; store.close()
                    account_keys = startup_manifest().required_keys(phase) if full_startup(m) else list(phase["expectedPoststate"])
                    rule_module = state_rules() if typed_phase(phase) or full_startup(m) else None
                    clock_key = rule_module.CLOCK if phase_clock(phase) else None
                    if clock_key and clock_key not in account_keys: account_keys.append(clock_key)
                    need(len(account_keys) <= 64,"startup_clock_account_capacity")
                    collection_request={"schema":c.SCHEMA,"wireBase64":signed["wireBase64"],"expectedWireSha256":signed["wireSha256"],"expectedMessageSha256":prepared["reviewed"]["messageSha256"],"signature":signed["signature"],"expectedFeeLamports":prepared["feeQuoteLamports"],"accountKeys":startup_manifest().proof_keys(phase) if full_startup(m) else account_keys,"minimumContextSlot":str(state["floor"]),"includeExpiry":False}
                    report=(c.collect(collection_request,output/"collection",_transport=bounded_transport(c,_transport)) if _transport is not None else
                        native_collection("RECOVER",collection_request,output/"collection"))
                    obs=report.get("observation")
                    if obs is not None and obs["status"]["confirmationStatus"]=="finalized":
                        need(j.u64(obs["status"]["slot"])>state["lastSlot"],"global_slot_not_later")
                        if obs["status"]["err"] is None:
                            need(report["status"]=="COLLECTED_PUBLIC_EVIDENCE_ONLY" and obs["poststate"] is not None,"poststate_missing")
                            full_snapshot=None
                            snapshot=obs["poststate"]
                            if full_startup(m) and startup_manifest().large_key() in account_keys:
                                snapshot,full_snapshot=collect_full_snapshot(c,j,state,phase,signed,report,output,_transport)
                            actual=snapshot["accounts"]
                            post_clock=rule_module.clock(c,actual[clock_key],snapshot["contextSlot"],max(state["floor"],j.u64(obs["status"]["slot"]))) if clock_key else None
                            pre_clock=state["intents"][pid].get("runtimePreClock")
                            if clock_key: rule_module.window(pre_clock,post_clock,obs["status"]["slot"])
                            next_facts=verify_states(c,state,phase,"POST",actual,inclusion_slot=obs["status"]["slot"],pre_clock=pre_clock,post_clock=post_clock)
                        checked=check_signed(j,frozen(state),phase,prepared,signed,obs)
                        kind="FINALIZED" if obs["status"]["err"] is None else "FAILED"
                        post_coverage=startup_manifest().coverage(operator_api(),c,state,phase,"POST",prepared["reviewed"],next_facts,post_clock,actual,full_snapshot=full_snapshot) if full_startup(m) and kind == "FINALIZED" else None
                        if post_coverage is not None:
                            paths=[output/"collection"];payload=3*account_payload(post_coverage["accountDigests"],startup_manifest().proof_keys(phase))
                            if full_snapshot is not None:
                                paths.append(output/"full-state");payload+=2*account_payload(post_coverage["accountDigests"])
                            successful_evidence(j,paths,payload)
                        op.append(kind,{"id":pid,"slot":obs["status"]["slot"],"contextSlot":snapshot["contextSlot"] if kind == "FINALIZED" else obs["status"]["slot"],"feeLamports":obs["transaction"]["feeLamports"],"wireSha256":signed["wireSha256"],"signatureHex":signed["signatureHex"],"checkerReport":checked,"evidencePath":str(output/"collection"),"manifestSha256":report["manifestReceipt"]["sha256"],"runtimeFacts":next_facts if kind == "FINALIZED" else state["runtimeFacts"],"runtimePostClock":post_clock if kind == "FINALIZED" else None,**({"coverage":post_coverage,"fullSnapshot":full_snapshot} if post_coverage is not None else {})})
                    else: op.append("UNKNOWN",{"id":pid,"reason":"ORIGINAL_SIGNATURE_UNRESOLVED_NO_REPLACEMENT","evidencePath":str(output/"collection"),"manifestSha256":report["manifestReceipt"]["sha256"]})
        if output_mode == "COMMIT_RECEIPT":
            # Only acknowledge this append's fsynced bytes. No postcommit
            # dependency validation, derived state or permission is reported;
            # every subsequent action still independently replays its prefix.
            try:
                need(committed is not None and committed["kind"] == ("PREPARE" if action == "PREPARE" else "IMPORT") and
                    committed["digest"] == op.tail() and committed["sequence"] == str(len(op.chain)-1),"commit_receipt_record_required")
                op.stable()
                raw=j.read_file(op.fd,f"{int(committed['sequence']):08}.json",j.MAX_INPUT)
                need(raw == encode(committed),"commit_receipt_record_changed")
            except Exception as exc: raise Stop("commit_receipt_uncertain_no_retry_or_adoption") from exc
            return {"schema":"piv1-startup-release-commit-receipt-v1","status":"DURABLE_COMMIT_ONLY","action":action,"recordSequence":committed["sequence"],"recordKind":committed["kind"],"recordDigest":committed["digest"],"tail":op.tail(),"durableCommit":True,"postCommitReplayPerformed":False}
        state=replay(j,op.chain)
        unresolved=state["cursor"]<len(state["manifest"]["phases"]) and state["manifest"]["phases"][state["cursor"]]["id"] in state["intents"]
        possible_live_signer=any(a["mode"]=="LOCAL_TESTNET_BACKEND" for a in state["signerAttempts"].values())
        return {"status":"SIGNER_ATTEMPT_UNRESOLVED_ORIGINAL_RESPONSE_ONLY" if state["signerAttempt"] is not None else "FINALIZED_FAILURE_STOP" if state["failed"] else "ALL_PHASES_RECONCILED" if state["cursor"]==len(state["manifest"]["phases"]) else "UNKNOWN_OR_SUBMITTED_RECOVERY_ONLY" if unresolved else "DURABLE_RELEASE_PREPARATION_ONLY", "tail":op.tail(),"completedPhases":state["cursor"],"phaseCapacity":MAX_PHASES,"historicalLedger":copy.deepcopy(state["manifest"]["historicalLedger"]),"proposedFeeLedger":{"namespace":state["manifest"]["proposedFeeBudget"]["namespace"],"status":"PROPOSAL_ONLY_NOT_AUTHORIZED","feeCapLamports":state["manifest"]["proposedFeeBudget"]["feeCapLamports"],"reservedFeeLamports":str(state["reserved"]),"actualFeeLamports":str(state["actual"])},"pendingSigningRequest":copy.deepcopy(state["handoff"]),"pendingSignerAttempt":copy.deepcopy(state["signerAttempt"]),"signingPackets":[copy.deepcopy(state["prepared"][p["id"]]) for p in state["handoff"]["packets"]] if state["handoff"] else [],"liveReady":False,"founderAuthorizationCryptographicallyVerified":False,"completeStateOracle":False,"mandatoryStartupManifestConsumed":full_startup(state["manifest"]),"signerInvoked":None if possible_live_signer else False,"signerInvocationStatus":"MAY_HAVE_RUN_NO_CONTROL_PROOF" if possible_live_signer else "OFFLINE_INJECTED_ONLY" if state["signerAttempts"] else "NOT_INVOKED","signerControlProven":False}
    finally:
        try:
            if compound_store is not None:compound_store.close()
        finally:op.close()


def main(argv):
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("request"); p.add_argument("base_parent"); p.add_argument("--authorization-receipt"); p.add_argument("--authorization-sha256"); p.add_argument("--allow-live-send",action="store_true"); p.add_argument("--allow-live-sign",action="store_true")
    p.add_argument("--output-mode",choices=("REPORT","COMMIT_RECEIPT"),default="REPORT",help="PREPARE/SIGN only: acknowledge durable append without a postcommit state report")
    args=p.parse_args(argv)
    # All current preparations omit authorization and the live flag.
    r,c,s,j=helpers(); raw=Path(args.request).read_bytes(); request=j.parse(raw)
    auth=Path(args.authorization_receipt).read_bytes() if args.authorization_receipt else None
    result=operate(request,args.base_parent,authorization_raw=auth,authorization_sha256=args.authorization_sha256,live=args.allow_live_send,live_sign=args.allow_live_sign,output_mode=args.output_mode)
    print(json.dumps(result,sort_keys=True,indent=2))


if __name__=="__main__":
    try: main(__import__('sys').argv[1:])
    except Exception as exc:
        print(json.dumps({"status":"STOP","reason":str(exc) if isinstance(exc,(Stop,ValueError)) else "operator_failure","liveReady":False}),file=__import__('sys').stderr)
        raise SystemExit(1)


def candidate_verify_state_handoff(manifest, index, stage, accounts, *, prior_runtime=None,
                                   clock_binding=None, context_slot=None, inclusion_slot=None):
    """Import-only OFFLINE component handoff; operate/CREATE/live dispatch are unchanged."""
    return startup_manifest().candidate_runtime_handoff(manifest,index,stage,accounts,
        prior_runtime=prior_runtime,clock_binding=clock_binding,context_slot=context_slot,
        inclusion_slot=inclusion_slot)


# Immutable standalone HOST_PREPARATION only; never a controller/parent record.
CANDIDATE_PREPARATION_SCHEMA = "piv1-candidate-host-preparation-seal-v1"
CANDIDATE_PREPARATION_LOCK = b"piv1-candidate-host-preparation-lock-v1\n"
CANDIDATE_PREPARATION_STORAGE_SHA = "9b614bef1f1b1a11a81e65dbc9bcbcbfde0bea0df61f91d6c918be43812453a6"
CANDIDATE_PREPARATION_FLAGS = {"liveReady": False, "authorized": False, "controllerReady": False,
    "completeStateOracle": False, "fullSnapshotVerified": False, "runtimePredicateVerification": False,
    "completePrerequisitesAuthenticated": False, "executionLineageAuthenticated": False,
    "fundingAuthenticated": False, "sourceControlAuthenticated": False, "callerAnchorAuthenticated": False,
    "freshnessAuthenticated": False, "controlledSuccessor": False, "signingOrSendPerformed": False,
    "chainStateAuthenticated": False, "snapshotFreshnessAuthenticated": False, "preVoteCausalityProved": False,
    "voteSignatureProvenanceProved": False, "timelockReleased": False, "executionEligible": False,
    "newFeesReserved": False, "journalMutationPerformed": False}


def _candidate_preparation_sources():
    repo = Path(__file__).resolve().parent.parent
    paths = {repo/"tools/startup_manifest.py": MANIFEST_SHA,
        repo/"tools/startup_state_rules.py": STATE_RULES_SHA,
        repo/"tools/startup_initializer_storage.py": CANDIDATE_PREPARATION_STORAGE_SHA}
    captured = {}
    def load(name, path, expected):
        need(not path.is_symlink() and path.is_file(), "candidate_preparation_source_pin")
        raw = path.read_bytes(); need(sha(raw) == expected, "candidate_preparation_source_pin")
        captured[path] = raw
        spec = importlib.util.spec_from_file_location(name,path); module = importlib.util.module_from_spec(spec)
        exec(compile(raw,str(path),"exec",dont_inherit=True),module.__dict__)
        return module
    a = load("preparation_manifest",repo/"tools/startup_manifest.py",MANIFEST_SHA)
    j,r,c = a._candidate_helpers()
    s = load("preparation_storage",repo/"tools/startup_initializer_storage.py",CANDIDATE_PREPARATION_STORAGE_SHA)
    paths.update({repo/name:digest for name,digest in a.CANDIDATE_HELPER_PINS.items()})
    paths.update({repo/name:digest for name,digest in a.JITO_SOURCE_PINS.items()})
    paths[repo/"programs/piv1/src/genesis_allocation.rs"] = "d7878a592ad670396a9158e7cd5c1edc31f390302af373ac16d8cf69b56e968e"
    paths.update({repo/name:digest for name,digest in s.SOURCE_PINS.items() if name != "tools/startup_public_journal.py"})
    paths[repo/"tools/startup_manifest.candidate-oracle.json"] = a.CANDIDATE_ORACLE_SHA
    need(s.CANDIDATE_JOURNAL_SHA == a.CANDIDATE_HELPER_PINS["tools/startup_public_journal.py"], "candidate_preparation_storage_domain")
    for path,expected in paths.items():
        need(not path.is_symlink() and path.is_file(), "candidate_preparation_source_pin")
        raw = captured.get(path)
        if raw is None: raw = path.read_bytes(); captured[path] = raw
        need(sha(raw) == expected, "candidate_preparation_source_pin")
    own = Path(__file__).resolve(); captured[own] = own.read_bytes()
    domain = {str(path.relative_to(repo)):sha(raw) for path,raw in captured.items()}
    domain.update(node=j.NODE_SHA,sdk=j.SDK_SHA,checker=j.CHECKER_SHA,lockfile=j.LOCK_SHA,**j.CANDIDATE_PLANNER_PINS)
    def unchanged(boundary):
        for path,raw in captured.items():
            need(not path.is_symlink() and path.read_bytes() == raw, "candidate_preparation_source_changed:"+boundary)
        j.candidate_pins(); r.verify_sources(repo)
    unchanged("load")
    return a,j,s,domain,unchanged


def _candidate_preparation_binding(a,j,manifest,binding):
    m = a.candidate_validate(manifest); b = j.validate_candidate_clock_binding(binding); ref = a.candidate_oracle()
    fixed = copy.deepcopy(b["preManifest"])
    for name in ("recentBlockhash","lastValidBlockHeight","referenceSlot"):
        fixed["initializer"]["transport"][name] = ref["preManifest"]["initializer"]["transport"][name]
    fixed["existingAlt"]["contextSlot"] = ref["preManifest"]["existingAlt"]["contextSlot"]
    need(encode(fixed) == encode(ref["preManifest"]) and m["candidateScope"] == b["scope"] == a.CANDIDATE_SCOPE, "candidate_preparation_policy_relationship")
    p = b["unsignedInitializerPlan"]
    need(p["alt"]["addresses"] == ref["targetAddresses"] and p["accountSizes"] == ref["accountSizes"], "candidate_preparation_policy_relationship")
    need(all(p[name] == ref["roles"][role] for name,role in (("program","pivProgram"),("programData","programData"),("buffer","governanceBuffer1"),("transaction","transaction1"),("proposal","proposal1"),("vault","vault0"))), "candidate_preparation_policy_relationship")
    return m,b


def _candidate_preparation_access(path, expected_digest, *, manifest=None, clock_binding=None, snapshot=None, stage=None):
    create = manifest is not None or clock_binding is not None
    need((manifest is not None and clock_binding is not None and expected_digest is None and snapshot is None and stage is None) if create else (expected_digest is not None and (snapshot is None) == (stage is None)), "candidate_preparation_arguments")
    path = Path(os.path.abspath(path)); need(path.name.endswith(".candidate-preparation"), "candidate_preparation_namespace")
    a,j,s,domain,unchanged = _candidate_preparation_sources()
    if create: manifest,clock_binding = _candidate_preparation_binding(a,j,manifest,clock_binding)
    else: j.digest(expected_digest)
    unchanged("after_validation")
    try: fd = j.directory(path,create)
    except FileExistsError as exc: raise Stop("candidate_preparation_exists_no_retry_or_adoption") from exc
    lock = None
    try:
        if create: j.write_file(fd,"lock",CANDIDATE_PREPARATION_LOCK)
        lock = os.open("lock",os.O_RDWR|os.O_NOFOLLOW|os.O_NONBLOCK,dir_fd=fd)
        meta = os.fstat(lock); j.secure_meta(meta); fcntl.flock(lock,fcntl.LOCK_EX); directory_meta = os.fstat(fd)
        identity = {"device":str(meta.st_dev),"inode":str(meta.st_ino),"ctimeNs":str(meta.st_ctime_ns)}
        def stable():
            now = os.stat("lock",dir_fd=fd,follow_symlinks=False); here = os.stat(path,follow_symlinks=False)
            need((now.st_dev,now.st_ino,now.st_ctime_ns) == (meta.st_dev,meta.st_ino,meta.st_ctime_ns)
                and (here.st_dev,here.st_ino) == (directory_meta.st_dev,directory_meta.st_ino)
                and j.read_file(fd,"lock",128) == CANDIDATE_PREPARATION_LOCK, "candidate_preparation_lock_changed")
        stable()
        need(set(os.listdir(fd)) == ({"lock"} if create else {"lock","seal.json"}), "candidate_preparation_closed_namespace")
        if create:
            record = {"schema":CANDIDATE_PREPARATION_SCHEMA,"scope":"HOST_PREPARATION_ONLY",
                "preparationPath":str(path),"lockIdentity":identity,"sourceDomain":domain,
                "manifest":manifest,"manifestSha256":sha(encode(manifest)),
                "initializerBinding":clock_binding,"initializerBindingSha256":sha(encode(clock_binding)),
                "historicalUncertaintyReference":copy.deepcopy(manifest["historicalUncertainty"]),
                "packetScope":"NINE_ZERO_INITIALIZER_PREPARATION_NOT_947_LIFETIME_OR_PREREQUISITE_ADMISSION",
                "assurances":dict(CANDIDATE_PREPARATION_FLAGS)}
            raw = encode(record); need(len(raw) <= j.MAX_INPUT, "candidate_preparation_bound")
            try: j.write_file(fd,"seal.json",raw)
            except Exception as exc: raise Stop("candidate_preparation_commit_uncertain_no_retry_or_adoption") from exc
            stable(); unchanged("after_durability")
            need(j.read_file(fd,"seal.json",j.MAX_INPUT) == raw, "candidate_preparation_durable_bytes_changed")
        else:
            raw = j.read_file(fd,"seal.json",j.MAX_INPUT); need(sha(raw) == expected_digest, "candidate_preparation_exact_digest")
            record = j.parse(raw)
            fields(record,("schema","scope","preparationPath","lockIdentity","sourceDomain","manifest","manifestSha256","initializerBinding","initializerBindingSha256","historicalUncertaintyReference","packetScope","assurances"))
            need(encode(record) == raw and record["schema"] == CANDIDATE_PREPARATION_SCHEMA and record["scope"] == "HOST_PREPARATION_ONLY", "candidate_preparation_record_domain")
            need(record["preparationPath"] == str(path) and record["lockIdentity"] == identity and record["sourceDomain"] == domain, "candidate_preparation_source_or_identity")
            manifest,clock_binding = _candidate_preparation_binding(a,j,record["manifest"],record["initializerBinding"])
            need(record["manifestSha256"] == sha(encode(manifest)) and record["initializerBindingSha256"] == sha(encode(clock_binding))
                and encode(record["historicalUncertaintyReference"]) == encode(manifest["historicalUncertainty"])
                and record["packetScope"] == "NINE_ZERO_INITIALIZER_PREPARATION_NOT_947_LIFETIME_OR_PREREQUISITE_ADMISSION"
                and encode(record["assurances"]) == encode(CANDIDATE_PREPARATION_FLAGS), "candidate_preparation_record_binding")
            stable(); unchanged("after_readback")
        result = {"status":"HOST_PREPARATION_SEALED_ONLY" if create else "HOST_PREPARATION_READBACK_ONLY",
            "sealDigest":sha(raw),"manifestSha256":record["manifestSha256"],"initializerBindingSha256":record["initializerBindingSha256"],
            "sourceDomain":domain,"seal":record,**CANDIDATE_PREPARATION_FLAGS}
        if snapshot is not None:
            # Same validated parser/PDA path as candidate_check_supplied, using captured helper bytes.
            g = clock_binding["postManifest"]["initializer"]["governance"]
            pdas = s.derive_pdas(j,Path(__file__).resolve().parent.parent,g)
            evidence = s.validate_snapshot(snapshot,clock_binding,pdas,stage)
            result.update(status="HOST_PREPARATION_SUPPLIED_STORAGE_BYTES_ONLY",snapshotSha256=sha(encode(snapshot)),snapshotContextSlot=snapshot["contextSlot"],stage=stage,pdaDerivations=pdas,storageEvidence=evidence)
            stable(); unchanged("after_storage")
        need(set(os.listdir(fd)) == {"lock","seal.json"} and j.read_file(fd,"seal.json",j.MAX_INPUT) == raw, "candidate_preparation_readback_changed")
        stable(); unchanged("before_success")
        return result
    finally:
        if lock is not None: fcntl.flock(lock,fcntl.LOCK_UN); os.close(lock)
        os.close(fd)


def candidate_prepare_seal(path, manifest, clock_binding):
    """Create one immutable HOST_PREPARATION seal in a new dedicated namespace."""
    need(manifest is not None and clock_binding is not None, "candidate_preparation_arguments")
    return _candidate_preparation_access(path,None,manifest=manifest,clock_binding=clock_binding)


def candidate_read_preparation_seal(path, expected_digest):
    """Read exactly the pinned HOST_PREPARATION artifact, never an operational parent."""
    return _candidate_preparation_access(path,expected_digest)


def candidate_preparation_storage_readback(path, expected_digest, snapshot, stage):
    """Check supplied Squads bytes against the exact durable HOST_PREPARATION binding."""
    need(snapshot is not None and stage is not None, "candidate_preparation_arguments")
    return _candidate_preparation_access(path,expected_digest,snapshot=snapshot,stage=stage)


def _candidate_controller_intent(a, j, manifest, data):
    """Commit a fixed original identity, NOT a signature/submission proof."""
    fields(data,("wireBase64","feeCapLamports","originalBlockhash","preflight"))
    raw = encode(data); need(len(raw) <= 196608, "candidate_controller_intent_bound")
    data = j.parse(raw); _,_,c = a._candidate_helpers(); ref = a.candidate_oracle()
    config = {"commitment":"finalized"}; floor = int(ref["baselineContextSlot"])
    genesis,original = a._candidate_funding_frame(c,data["originalBlockhash"],[
        ("getGenesisHash",[]),("getLatestBlockhash",[{**config,"minContextSlot":floor}])])
    need(genesis == c.GENESIS and c.u64(original["context"]["slot"]) >= floor, "candidate_controller_original_identity")
    review = a._candidate_funding_review(ref,c,original["value"]["blockhash"])
    wire = c.b64(data["wireBase64"],1232)
    need(len(wire) == 215 and wire[:1] == b"\x01" and wire[65:] == c.b64(review["messageBase64"],1232), "candidate_controller_original_wire")
    quote,before,height,valid = a._candidate_funding_frame(c,data["preflight"],[
        ("getFeeForMessage",[review["messageBase64"],config]),
        ("getMultipleAccounts",[[ref["roles"][k] for k in ("source","h2","clock")],
            {"encoding":"base64",**config,"minContextSlot":c.u64(original["context"]["slot"])}]),
        ("getBlockHeight",[config]),("isBlockhashValid",[original["value"]["blockhash"],config])])
    fee = c.u64(quote["value"]); cap = c.u64(data["feeCapLamports"],True)
    need(0 < fee <= cap and valid["value"] is True and c.u64(height)+40 <= c.u64(original["value"]["lastValidBlockHeight"]), "candidate_controller_declared_preflight")
    return {"intent":data,"identity":{"manifestSha256":sha(encode(manifest)),"intentSha256":sha(raw),
        "wireSha256":sha(wire),"messageSha256":review["messageSha256"],"signatureHex":wire[1:65].hex(),
        "status":"UNVERIFIED_HOST_COMMITMENT_NOT_SUBMISSION_OR_SIGNED_PROOF"}}


def _candidate_prefix_bound(index):
    need(type(index) is int and 0 <= index < CANDIDATE_PREFIX_COUNT, "candidate_prefix_index")
    return 16*1024*1024 if index == 0 else 2*1024*1024


def _candidate_prefix_remaining(cursor):
    need(type(cursor) is int and 0 <= cursor <= CANDIDATE_PREFIX_COUNT, "candidate_prefix_cursor")
    return sum(2*_candidate_prefix_bound(i) for i in range(cursor,CANDIDATE_PREFIX_COUNT))


def _candidate_startup_bound(index):
    need(type(index) is int and 0 <= index < CANDIDATE_STARTUP_COUNT, "candidate_startup_index")
    return _candidate_prefix_bound(index) if index < CANDIDATE_PREFIX_COUNT else 16*1024*1024


def _candidate_startup_remaining(cursor):
    need(type(cursor) is int and 0 <= cursor <= CANDIDATE_STARTUP_COUNT, "candidate_startup_cursor")
    return sum(2*_candidate_startup_bound(i) for i in range(cursor,CANDIDATE_STARTUP_COUNT))


def _candidate_prefix_frame(c, frame, queries, index):
    # Same closed raw RPC contract as funding, with fixed full-Buffer bounds.
    fields(frame,("requestBase64","responseBase64","httpBase64"))
    request = c.b64(frame["requestBase64"],65536); response = c.b64(frame["responseBase64"],_candidate_startup_bound(index)); http = c.b64(frame["httpBase64"],16384)
    expected = [{"jsonrpc":"2.0","id":i+1,"method":method,"params":params} for i,(method,params) in enumerate(queries)]
    need(request == c.encode(expected), "candidate_prefix_raw_request")
    meta = c.strict_json(http)
    need(type(meta) is dict and {"status","headers"} <= set(meta) <= {"status","headers","responseCompletedUtc"}
        and type(meta["status"]) is int and meta["status"] == 200 and type(meta["headers"]) is list and len(http) <= 16384, "candidate_prefix_http")
    replies = c.strict_json(response)
    need(type(replies) is list and len(replies) == len(expected), "candidate_prefix_rpc_count")
    by = {}
    for reply in replies:
        need(type(reply) is dict and set(reply) == {"jsonrpc","id","result"} and reply["jsonrpc"] == "2.0"
            and type(reply["id"]) is int and 1 <= reply["id"] <= len(expected) and reply["id"] not in by, "candidate_prefix_rpc_envelope")
        by[reply["id"]] = reply["result"]
    return [by[i+1] for i in range(len(expected))]


def _candidate_prefix_inventory(j, path, refs):
    root = path.with_name(path.name+".evidence")
    if not root.exists():
        need(not root.is_symlink() and not refs, "candidate_prefix_evidence_missing")
        return 0
    fd = j.directory(root); total = 0
    try:
        names = os.listdir(fd)
        for name in names:
            child = j.directory(root/name)
            try:
                for filename in os.listdir(child):
                    meta = os.stat(filename,dir_fd=child,follow_symlinks=False); j.secure_meta(meta)
                    total += meta.st_size; need(total <= MAX_EVIDENCE, "candidate_prefix_evidence_cap")
            finally: os.close(child)
        need(refs and set(names) == {ref["name"] for ref in refs}, "candidate_prefix_orphan_uncertain_stop")
        return total
    finally: os.close(fd)


def _candidate_prefix_load(j, path, ref, sequence, kind, index):
    fields(ref,("name","bytes","sha256","rootIdentity"))
    name = f"{int(sequence):08}-"+("intent" if kind == "PHASE_INTENT" else "result")
    need(ref["name"] == name and type(ref["bytes"]) is int and 0 < ref["bytes"] <= _candidate_startup_bound(index), "candidate_prefix_evidence_reference")
    j.digest(ref["sha256"]); root = path.with_name(path.name+".evidence"); root_fd = j.directory(root)
    try:
        meta = os.fstat(root_fd)
        need(ref["rootIdentity"] == {"device":str(meta.st_dev),"inode":str(meta.st_ino)}, "candidate_prefix_evidence_root")
        fd = j.directory(root/name)
        try:
            child_meta = os.fstat(fd)
            def stable():
                for named,opened in ((root,meta),(root/name,child_meta)):
                    now = os.stat(named,follow_symlinks=False)
                    need(stat.S_ISDIR(now.st_mode) and stat.S_IMODE(now.st_mode) == 0o700 and now.st_uid == os.getuid()
                        and (now.st_dev,now.st_ino) == (opened.st_dev,opened.st_ino), "candidate_prefix_evidence_directory_changed")
            stable()
            need(set(os.listdir(fd)) == {"payload.json"}, "candidate_prefix_payload_namespace")
            raw = j.read_file(fd,"payload.json",_candidate_startup_bound(index))
            need(len(raw) == ref["bytes"] and sha(raw) == ref["sha256"], "candidate_prefix_payload_hash")
            value = j.parse(raw); need(encode(value) == raw, "candidate_prefix_payload_canonical")
            stable(); need(j.read_file(fd,"payload.json",_candidate_startup_bound(index)) == raw, "candidate_prefix_payload_changed")
            return value
        finally: os.close(fd)
    finally: os.close(root_fd)


def _candidate_prefix_store(c,j,op,state,kind,raw):
    index = state["cursor"]; need(len(raw) <= _candidate_startup_bound(index), "candidate_prefix_payload_bound")
    refs = [r["data"]["evidence"] for r in op.chain if r["kind"] in ("PHASE_INTENT","PHASE_RESULT")]
    total = _candidate_prefix_inventory(j,op.path,refs)
    remaining = _candidate_startup_remaining(index)
    need(total+remaining+ACTION_EVIDENCE <= MAX_EVIDENCE, "candidate_prefix_future_evidence_cap")
    j.space(op.fd,remaining+ACTION_EVIDENCE+MAX_JOURNAL-op.total); op.stable()
    root = op.path.with_name(op.path.name+".evidence")
    if not refs:
        need(not root.exists() and not root.is_symlink(), "candidate_prefix_orphan_uncertain_stop")
        fd = j.directory(root,True); os.close(fd)
    fd = j.directory(root)
    try: meta = os.fstat(fd)
    finally: os.close(fd)
    name = f"{len(op.chain):08}-"+("intent" if kind == "PHASE_INTENT" else "result")
    try:
        store = c.Store(root/name)
        try: store.write("payload.json",raw)
        finally: store.close()
        ref = {"name":name,"bytes":len(raw),"sha256":sha(raw),"rootIdentity":{"device":str(meta.st_dev),"inode":str(meta.st_ino)}}
        need(encode(_candidate_prefix_load(j,op.path,ref,str(len(op.chain)),kind,index)) == raw, "candidate_prefix_payload_readback")
        return ref
    except Exception as exc: raise Stop("candidate_prefix_commit_uncertain_no_retry_or_adoption") from exc


def _candidate_prefix_review(a,c,manifest,index,blockhash):
    _candidate_prefix_bound(index); review = copy.deepcopy(manifest["phases"][index]["reviewed"])
    raw = c.b64(review["messageBase64"],1232); n = len(review["accountRoles"])
    need(review["lookupTables"] == [] and 1 <= n < 128 and raw[3] == n and raw[0] == len(review["requiredSigners"]), "candidate_prefix_static_geometry")
    offset = 4+32*n; raw = raw[:offset]+c.b58decode(blockhash,32)+raw[offset+32:]
    actual = {**review,"messageBase64":base64.b64encode(raw).decode(),"messageSha256":sha(raw)}
    need(semantic(actual) == semantic(review), "candidate_prefix_review_policy")
    return actual


def _candidate_prefix_accounts(a,c,result,keys):
    need(type(result.get("value")) is list and len(result["value"]) == len(keys), "candidate_prefix_accounts")
    actual = {}
    for key,value in zip(keys,result["value"]):
        if key == a.large_key() and value is not None:
            need(type(value.get("data")) is list and len(value["data"]) == 2 and value["data"][1] == "base64", "candidate_prefix_large_encoding")
            actual[key] = {"exists":True,"owner":value.get("owner"),"executable":value.get("executable"),
                "lamports":str(c.u64(value.get("lamports"))),"dataBase64":value["data"][0]}
            a._candidate_account_digest(c,key,actual[key],a.candidate_oracle())
        else: actual[key] = c.normalize_account(value)
    return actual


def _candidate_prefix_intent(a,j,state,data):
    if state["cursor"] >= CANDIDATE_PREFIX_COUNT: return _candidate_runtime_intent(a,j,state,data)
    fields(data,("wireBase64","feeCapLamports","originalBlockhash","preflight"))
    i = state["cursor"]; _candidate_prefix_bound(i); _,rules,c = a._candidate_helpers()
    need(state["funding"] is not None and state["pending"] is None, "candidate_prefix_funding_or_original_hold")
    config = {"commitment":"finalized"}; floor = int(state["lastClock"]["contextSlot"])
    genesis,original = _candidate_prefix_frame(c,data["originalBlockhash"],[
        ("getGenesisHash",[]),("getLatestBlockhash",[{**config,"minContextSlot":floor}])],i)
    origin = c.u64(original["context"]["slot"]); blockhash = original["value"]["blockhash"]
    need(genesis == c.GENESIS and origin >= floor, "candidate_prefix_original_origin")
    review = _candidate_prefix_review(a,c,state["manifest"],i,blockhash)
    wire = c.b64(data["wireBase64"],1232); count = len(review["requiredSigners"])
    need(wire[:1] == bytes([count]) and wire[1+64*count:] == c.b64(review["messageBase64"],1232), "candidate_prefix_original_wire")
    keys = a.candidate_expected(state["manifest"],i,"PRE",funding_mode="CONDITIONAL_H2_CREDIT")["requiredSnapshotKeys"]
    quote,before,height,valid = _candidate_prefix_frame(c,data["preflight"],[
        ("getFeeForMessage",[review["messageBase64"],config]),
        ("getMultipleAccounts",[keys,{"encoding":"base64",**config,"minContextSlot":origin}]),
        ("getBlockHeight",[config]),("isBlockhashValid",[blockhash,config])],i)
    context = c.u64(before["context"]["slot"]); fee = c.u64(quote["value"]); cap = c.u64(data["feeCapLamports"],True)
    last = c.u64(original["value"]["lastValidBlockHeight"]); height = c.u64(height)
    need(origin <= min(context,c.u64(quote["context"]["slot"]),c.u64(valid["context"]["slot"]))
        and 0 < fee <= cap and valid["value"] is True and height+40 <= last, "candidate_prefix_declared_preflight")
    need(fee == int(state["manifest"]["phases"][i]["historicalFeeProjectionLamports"]), "candidate_prefix_fee_profile_changed")
    actual = _candidate_prefix_accounts(a,c,before,keys)
    checked = verify_states(c,{"manifest":state["manifest"],"candidateFunding":state["funding"]},state["manifest"]["phases"][i],"PRE",actual,context_slot=str(context))
    clock = checked["clock"]
    need(int(clock["slot"]) >= floor and int(clock["unixTimestamp"]) >= int(state["lastClock"]["unixTimestamp"]), "candidate_prefix_clock_regression")
    return {"input":data,"reviewed":review,"preAccounts":actual,"preClock":clock,"preContext":str(context),"quoteContexts":[str(c.u64(quote["context"]["slot"])),str(c.u64(valid["context"]["slot"]))],
        "feeQuoteLamports":str(fee),"lifetime":{"recentBlockhash":blockhash,"currentBlockHeight":str(height),"lastValidBlockHeight":str(last),"minimumRemainingBlockHeights":"40"},
        "identity":{"index":i,"wireSha256":sha(wire),"messageSha256":review["messageSha256"],"signatureHex":wire[1:65].hex()}}


def _candidate_runtime_template(a,j,manifest,index,binding,blockhash):
    need(type(index) is int and 935 <= index < CANDIDATE_STARTUP_COUNT, "candidate_runtime_index")
    if 937 <= index <= 945:
        need(binding is not None, "candidate_runtime_template_binding")
        plan = binding["unsignedInitializerPlan"]; packet = plan["packets"][index-937]
        tables = [{"address":plan["alt"]["address"],"addresses":plan["alt"]["addresses"]}] if packet["version"] == 0 else []
        unsigned = packet["wireBase64"]
    else:
        review = manifest["phases"][index]["reviewed"]; tables = review["lookupTables"]
        unsigned = base64.b64encode(bytes([len(review["requiredSigners"])])+bytes(64*len(review["requiredSigners"]))+base64.b64decode(review["messageBase64"],validate=True)).decode()
    decoded, = describe(j,[{"wireBase64":unsigned,"lookupTables":tables}],_runtime_metadata=True)
    if not 937 <= index <= 945:
        need(encode(decoded["reviewed"]) == encode(review), "candidate_runtime_static_template")
    _,_,c = a._candidate_helpers()
    _,review = rebind_unsigned(c,unsigned,decoded["reviewed"],blockhash)
    return review,decoded


def _candidate_runtime_intent(a,j,state,data):
    # Clock-dependent IN02 is sampled BEFORE the original hash/message quote.
    fields(data,("wireBase64","feeCapLamports","prestate","originalBlockhash","preflight"))
    i = state["cursor"]; _candidate_startup_bound(i); _,rules,c = a._candidate_helpers(); ref = a.candidate_oracle()
    need(i >= 935 and state["funding"] is not None and state["pending"] is None, "candidate_runtime_funding_or_original_hold")
    config = {"commitment":"finalized"}; floor = int(state["lastClock"]["contextSlot"]); keys = sorted(ref["baseline"])
    before, = _candidate_prefix_frame(c,data["prestate"],[("getMultipleAccounts",[keys,{"encoding":"base64",**config,"minContextSlot":floor}])],i)
    context = c.u64(before["context"]["slot"]); actual = _candidate_prefix_accounts(a,c,before,keys)
    clock = rules.clock(c,actual[ref["roles"]["clock"]],context,floor)
    need(context >= floor and int(clock["slot"]) >= floor and int(clock["unixTimestamp"]) >= int(state["lastClock"]["unixTimestamp"]), "candidate_runtime_prestate_order")
    genesis,original = _candidate_prefix_frame(c,data["originalBlockhash"],[
        ("getGenesisHash",[]),("getLatestBlockhash",[{**config,"minContextSlot":context}])],i)
    origin = c.u64(original["context"]["slot"]); blockhash = original["value"]["blockhash"]
    need(genesis == c.GENESIS and origin >= context, "candidate_runtime_original_origin")
    binding = state["initializerBinding"]
    if i == 937:
        need(binding is None and state["runtimeFacts"] is not None, "candidate_runtime_binding_once")
        pre = copy.deepcopy(ref["preManifest"]); transport = pre["initializer"]["transport"]
        transport.update(recentBlockhash=blockhash,lastValidBlockHeight=str(c.u64(original["value"]["lastValidBlockHeight"])),referenceSlot=str(context))
        pre["existingAlt"]["contextSlot"] = str(context)
        raw_clock = actual[ref["roles"]["clock"]]
        binding = j.candidate_clock_binding(pre,{"address":ref["roles"]["clock"],"contextSlot":str(context),
            **{k:raw_clock[k] for k in ("dataBase64","owner","executable")}})
    need((i < 937) == (binding is None), "candidate_runtime_binding_boundary")
    review,metadata = _candidate_runtime_template(a,j,state["manifest"],i,binding,blockhash)
    wire = c.b64(data["wireBase64"],1232); count = len(review["requiredSigners"])
    need(wire[:1] == bytes([count]) and wire[1+64*count:] == c.b64(review["messageBase64"],1232), "candidate_runtime_original_wire")
    quote,height,valid = _candidate_prefix_frame(c,data["preflight"],[
        ("getFeeForMessage",[review["messageBase64"],config]),("getBlockHeight",[config]),("isBlockhashValid",[blockhash,config])],i)
    fee = c.u64(quote["value"]); cap = c.u64(data["feeCapLamports"],True); last = c.u64(original["value"]["lastValidBlockHeight"]); height = c.u64(height)
    contexts = [c.u64(quote["context"]["slot"]),c.u64(valid["context"]["slot"])]
    need(origin <= min(contexts) and 0 < fee <= cap and valid["value"] is True and height+40 <= last, "candidate_runtime_declared_preflight")
    need(fee == int(state["manifest"]["phases"][i]["historicalFeeProjectionLamports"]), "candidate_prefix_fee_profile_changed")
    checked = a.candidate_verify_accounts(state["manifest"],a.candidate_phase(i),"PRE",actual,context_slot=str(context),
        prior_runtime=state["runtimeFacts"],clock_binding=binding,funding_receipt=state["funding"]["receipt"],funding_receipt_sha256=state["funding"]["expectedReceiptSha256"])
    return {"input":data,"reviewed":review,"metadata":metadata,"binding":binding,"runtimeFacts":checked["runtimeFacts"],
        "preAccounts":actual,"preClock":clock,"preContext":str(context),"quoteContexts":list(map(str,contexts)),
        "feeQuoteLamports":str(fee),"lifetime":{"recentBlockhash":blockhash,"currentBlockHeight":str(height),"lastValidBlockHeight":str(last),"minimumRemainingBlockHeights":"40"},
        "identity":{"index":i,"wireSha256":sha(wire),"messageSha256":review["messageSha256"],"signatureHex":wire[1:65].hex(),
            "initializerBindingSha256":None if binding is None else sha(j.encode(binding)),"runtimePreFactsSha256":sha(j.encode(checked["runtimeFacts"]))}}


def _candidate_runtime_cpi(a,c,index,pending):
    """Closed source-contract trace; local Loader4.2 is not live Testnet4.4 proof."""
    ref = a.candidate_oracle(); r = ref["roles"]; roles = [x["address"] for x in pending["reviewed"]["accountRoles"]]
    system = "11111111111111111111111111111111"; squads = "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf"; token = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    rent = lambda n:int(ref["rents"][str(n)]); u32 = lambda n:n.to_bytes(4,"little"); u64 = lambda n:n.to_bytes(8,"little")
    key = lambda address:c.b58decode(address,32)
    trace = []
    def add(program,accounts,data,height=2):
        need(program in roles and all(k in roles for k in accounts), "candidate_runtime_cpi_roles")
        trace.append({"programIdIndex":roles.index(program),"accounts":[roles.index(k) for k in accounts],"data":c.b58encode(data),"stackHeight":height})
    def create(destination,size,owner,extra=()):
        add(system,[r["h2"],destination,*extra],u32(0)+u64(rent(size))+u64(size)+key(owner))
    parent = None
    if index == 935:
        parent = r["loaderProgram"]; create(r["programData"],892397,r["loaderProgram"],[r["loaderBuffer"]])
    elif index == 937:
        parent = squads; create(r["governanceBuffer1"],1557,squads)
    elif index == 939:
        # Anchor's empty VaultTransaction is102B; realloc topup is a distinct CPI.
        parent = squads; create(r["transaction1"],102,squads)
        add(system,[r["h2"],r["transaction1"]],u32(2)+u64(rent(1546)-rent(102)))
    elif index == 940:
        parent = squads; create(r["proposal1"],646,squads)
    elif index == 945:
        parent = squads; plan = pending["binding"]["unsignedInitializerPlan"]
        add(r["pivProgram"],[x["address"] for x in plan["native"]["roles"]],bytes.fromhex(plan["native"]["hex"]))
        for position,(address,size) in enumerate(zip(ref["targetAddresses"],ref["accountSizes"]["pivTargets"])):
            add(system,[r["h2"],address],u32(2)+u64(rent(size)),3)
            if size:  # Five native zero-data vaults require no allocate/assign.
                add(system,[address],u32(8)+u64(size),3)
                add(system,[address],u32(1)+key(r["pivProgram"] if position < 14 else token),3)
        mint, = [x["address"] for x in plan["native"]["roles"] if x["role"] == "mint"]
        for address in ref["targetAddresses"][14:]: add(token,[address,mint],bytes([18])+key(plan["pivAuthority"]),3)
    if parent is None: return []
    # Select the source-derived outer instruction, never a caller-supplied index.
    outer = [n for n,ix in enumerate(pending["metadata"]["instructions"]) if roles[ix["programIdIndex"]] == parent]
    if index == 935:
        outer = [n for n in outer if c.b64(pending["metadata"]["instructions"][n]["dataBase64"],1232)[:4] == u32(2)]
    need(len(outer) == 1, "candidate_runtime_cpi_outer")
    return [{"index":outer[0],"instructions":trace}]


def _candidate_runtime_tokens(a,c,items,accounts,roles):
    # Compare raw integer amounts/decimals; UI floats are presentation only.
    token = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"; expected = []
    ref = a.candidate_oracle(); mint_key = "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn"
    token_keys = {ref["preManifest"]["initializer"]["protocol"]["managerFee"],*ref["targetAddresses"][14:]}
    for index,address in enumerate(roles):
        account = accounts[address]
        if account["exists"] and account["owner"] == token:
            raw = c.b64(account["dataBase64"],165)
            if address == mint_key:
                need(len(raw) == 82 and raw[45] == 1, "candidate_runtime_token_mint"); continue
            need(address in token_keys and len(raw) == 165 and raw[108] == 1, "candidate_runtime_token_layout")
            mint = c.b58encode(raw[:32]); need(mint == mint_key, "candidate_runtime_token_mint_identity")
            mint_account = accounts[mint]; mint_data = c.b64(mint_account["dataBase64"],82)
            need(mint_account["exists"] is True and mint_account["owner"] == token and mint_account["executable"] is False
                and len(mint_data) == 82 and mint_data[45] == 1, "candidate_runtime_token_mint")
            expected.append({"accountIndex":index,"mint":mint,"owner":c.b58encode(raw[32:64]),"programId":token,
                "amount":str(int.from_bytes(raw[64:72],"little")),"decimals":mint_data[44]})
    need(type(items) is list and len(items) == len(expected), "candidate_runtime_token_metadata")
    for item,want in zip(items,expected):
        fields(item,("accountIndex","mint","owner","programId","uiTokenAmount")); ui = item["uiTokenAmount"]
        need(type(ui) is dict and {"amount","decimals"} <= set(ui) <= {"amount","decimals","uiAmount","uiAmountString"}
            and type(item["accountIndex"]) is int and type(ui["decimals"]) is int
            and encode({**{k:item[k] for k in ("accountIndex","mint","owner","programId")},"amount":ui["amount"],"decimals":ui["decimals"]}) == encode(want), "candidate_runtime_token_effects")
        need(("uiAmount" not in ui or ui["uiAmount"] is None or type(ui["uiAmount"]) in (int,float,c.Decimal) and c.Decimal(str(ui["uiAmount"])).is_finite())
            and ("uiAmountString" not in ui or type(ui["uiAmountString"]) is str and len(ui["uiAmountString"]) <= 80), "candidate_runtime_token_presentation")


def _candidate_runtime_metadata(a,c,index,pending,meta,actual):
    roles = [role["address"] for role in pending["reviewed"]["accountRoles"]]; lookup = pending["metadata"]["lookups"]
    loaded = {"writable":[],"readonly":[]}
    if index == 945:
        plan = pending["binding"]["unsignedInitializerPlan"]; alt = plan["alt"]
        need(encode(lookup) == encode([{"address":alt["address"],"writable":list(range(16)),"readonly":[]}]), "candidate_runtime_alt_indices")
        loaded["writable"] = list(alt["addresses"])
    else: need(lookup == [], "candidate_runtime_unexpected_lookup")
    need(encode(meta.get("loadedAddresses",{"writable":[],"readonly":[]})) == encode(loaded), "candidate_runtime_loaded_addresses")
    inner = meta.get("innerInstructions")
    need(encode([] if inner is None else inner) == encode(_candidate_runtime_cpi(a,c,index,pending)), "candidate_runtime_cpi_trace")
    if index == 945:
        _candidate_runtime_tokens(a,c,meta.get("preTokenBalances"),pending["preAccounts"],roles)
        _candidate_runtime_tokens(a,c,meta.get("postTokenBalances"),actual,roles)
    else: need(all(meta.get(k) in (None,[]) for k in ("preTokenBalances","postTokenBalances")), "candidate_runtime_unexpected_token_effects")


def _candidate_prefix_result(a,j,state,data):
    fields(data,("finalized","poststate")); pending = state["pending"]
    need(pending is not None, "candidate_prefix_result_without_intent")
    _,rules,c = a._candidate_helpers(); i = state["cursor"]; original = pending["input"]; config = {"commitment":"finalized"}
    signature = c.b58encode(bytes.fromhex(pending["identity"]["signatureHex"]))
    statuses,transaction = _candidate_prefix_frame(c,data["finalized"],[
        ("getSignatureStatuses",[[signature],{"searchTransactionHistory":True}]),
        ("getTransaction",[signature,{"encoding":"base64",**config,"maxSupportedTransactionVersion":0}])],i)
    need(type(statuses.get("value")) is list and len(statuses["value"]) == 1 and type(statuses["value"][0]) is dict
        and type(transaction) is dict, "candidate_prefix_original_unknown_hold")
    status = statuses["value"][0]; slot = c.u64(status["slot"])
    need(slot > int(state["lastSlot"]) and max(int(pending["preContext"]),*map(int,pending["quoteContexts"])) <= slot <= c.u64(statuses["context"]["slot"])
        and c.u64(transaction["slot"]) == slot and status.get("confirmationStatus") == "finalized"
        and "err" in status and status["err"] is None
        and (type(transaction.get("version")) is int and transaction["version"] == 0 if i == 945 else transaction.get("version") == "legacy")
        and transaction.get("transaction") == [original["wireBase64"],"base64"], "candidate_prefix_original_result")
    meta = transaction.get("meta"); need(type(meta) is dict and "err" in meta and meta["err"] is None
        and str(c.u64(meta["fee"])) == pending["feeQuoteLamports"], "candidate_prefix_fee_or_failure")
    keys = (a.candidate_expected(state["manifest"],i,"POST",funding_mode="CONDITIONAL_H2_CREDIT")["requiredSnapshotKeys"]
        if i < CANDIDATE_PREFIX_COUNT else sorted(a.candidate_oracle()["baseline"]))
    after, = _candidate_prefix_frame(c,data["poststate"],[("getMultipleAccounts",[keys,{"encoding":"base64",**config,"minContextSlot":slot}])],i)
    context = c.u64(after["context"]["slot"]); actual = _candidate_prefix_accounts(a,c,after,keys)
    role_keys = [role["address"] for role in pending["reviewed"]["accountRoles"]]
    balances = lambda values:[int(values[k]["lamports"]) if values[k]["exists"] else 0 for k in role_keys]
    need(type(meta.get("preBalances")) is list and type(meta.get("postBalances")) is list
        and [c.u64(n) for n in meta["preBalances"]] == balances(pending["preAccounts"])
        and [c.u64(n) for n in meta["postBalances"]] == balances(actual), "candidate_prefix_atomic_accounting")
    if i < CANDIDATE_PREFIX_COUNT:
        need(meta.get("loadedAddresses",{"writable":[],"readonly":[]}) == {"writable":[],"readonly":[]}
            and all(meta.get(k) in (None,[]) for k in ("innerInstructions","preTokenBalances","postTokenBalances")), "candidate_prefix_atomic_accounting")
        checked = verify_states(c,{"manifest":state["manifest"],"candidateFunding":state["funding"]},state["manifest"]["phases"][i],"POST",actual,
            context_slot=str(context),inclusion_slot=str(slot),pre_clock=pending["preClock"])
    else:
        _candidate_runtime_metadata(a,c,i,pending,meta,actual)
        checked = a.candidate_verify_accounts(state["manifest"],a.candidate_phase(i),"POST",actual,context_slot=str(context),
            prior_runtime=pending["runtimeFacts"],clock_binding=pending["binding"],inclusion_slot=str(slot),
            funding_receipt=state["funding"]["receipt"],funding_receipt_sha256=state["funding"]["expectedReceiptSha256"])
    observation = {"status":{"signatureHex":pending["identity"]["signatureHex"],"slot":str(slot),"confirmationStatus":"finalized","err":None},
        "transaction":{"slot":str(slot),"wireBase64":original["wireBase64"],"feeLamports":pending["feeQuoteLamports"],"err":None},
        "poststate":{"contextSlot":str(context),"accounts":{a.candidate_oracle()["roles"]["clock"]:actual[a.candidate_oracle()["roles"]["clock"]]}}}
    phase = {"id":f"candidate_{i:04}","feeCapLamports":original["feeCapLamports"]}
    signed = {"wireBase64":original["wireBase64"],**pending["identity"]}
    frozen = {"packageSha256":sha(encode(state["manifest"])),"payloadSha256":pending["reviewed"]["messageSha256"],"anchorUnixTimestamp":None,"transactionIndex":"0"}
    report = check_signed(j,frozen,phase,pending,signed,observation)
    need(report.get("reason") == "ALL_DECLARED_PHASES_FINALIZED_WITH_EXPECTED_POSTSTATE", "candidate_prefix_crypto_receipt_refused")
    summary = {"index":i,"slot":str(slot),"clock":checked["clock"],"wireSha256":pending["identity"]["wireSha256"]}
    if i >= CANDIDATE_PREFIX_COUNT:
        summary.update(runtimeFacts=checked["runtimeFacts"],initializerBindingSha256=pending["identity"]["initializerBindingSha256"])
    return summary


def _candidate_controller_replay(j, records, path):
    """Only this new host chain; no inherited parent, grant or runtime facts."""
    a,loaded_j,s,domain,unchanged = _candidate_preparation_sources()
    need(records and len(records) <= CANDIDATE_STARTUP_RECORDS and records[0]["kind"] == "CREATE", "candidate_controller_create_required")
    first = records[0]["data"]
    fields(first,("path","manifest","manifestSha256","sourceDomain","historicalUncertaintyReference"))
    manifest = a.candidate_validate(first["manifest"])
    need(first["path"] == str(path) and first["sourceDomain"] == domain
        and first["manifestSha256"] == sha(encode(manifest))
        and encode(first["historicalUncertaintyReference"]) == encode(manifest["historicalUncertainty"]), "candidate_controller_create_binding")
    state = {"manifest":manifest,"intent":None,"funding":None,"fundingEvidence":None,
        "cursor":0,"pending":None,"lastClock":None,"lastSlot":"0","runtimeFacts":None,"initializerBinding":None}
    refs = [r["data"]["evidence"] for r in records if r["kind"] in ("PHASE_INTENT","PHASE_RESULT")]
    _candidate_prefix_inventory(j,path,refs)
    for record in records[1:]:
        if record["kind"] == "FUNDING_INTENT":
            need(state["intent"] is None and record["sequence"] == "1", "candidate_controller_original_intent_only")
            derived = _candidate_controller_intent(a,j,manifest,record["data"]["intent"])
            need(encode(derived) == encode(record["data"]), "candidate_controller_intent_identity")
            state["intent"] = derived
        elif record["kind"] == "FUNDING_RESULT":
            need(record["kind"] == "FUNDING_RESULT" and state["intent"] is not None
                and state["funding"] is None, "candidate_controller_result_requires_original_intent")
            data = record["data"]; fields(data,("receipt","expectedReceiptSha256"))
            receipt = data["receipt"]
            need(type(receipt) is dict and all(k in receipt and encode(receipt[k]) == encode(v)
                for k,v in state["intent"]["intent"].items()), "candidate_controller_original_result_only")
            evidence = a.candidate_verify_funding(manifest,receipt,data["expectedReceiptSha256"])
            need(evidence["originalWireSha256"] == state["intent"]["identity"]["wireSha256"]
                and evidence["originalSignatureHex"] == state["intent"]["identity"]["signatureHex"], "candidate_controller_original_signature")
            state["funding"] = copy.deepcopy(data); state["fundingEvidence"] = evidence
            state["lastClock"] = evidence["postClock"]; state["lastSlot"] = evidence["inclusionSlot"]
        else:
            kind = record["kind"]; need(kind in ("PHASE_INTENT","PHASE_RESULT") and state["funding"] is not None, "candidate_prefix_funding_required")
            data = record["data"]; fields(data,("evidence","derived")); index = state["cursor"]
            payload = _candidate_prefix_load(j,path,data["evidence"],record["sequence"],kind,index)
            if kind == "PHASE_INTENT":
                derived = _candidate_prefix_intent(a,j,state,payload)
                need(encode(data["derived"]) == encode(derived["identity"]), "candidate_prefix_intent_binding")
                state["pending"] = derived
                if index == 937: state["initializerBinding"] = derived["binding"]
            else:
                derived = _candidate_prefix_result(a,j,state,payload)
                need(encode(data["derived"]) == encode(derived), "candidate_prefix_result_binding")
                state.update(cursor=index+1,pending=None,lastClock=derived["clock"],lastSlot=derived["slot"])
                if index >= 935: state["runtimeFacts"] = derived["runtimeFacts"]
    for record in records:
        if record["kind"] in ("PHASE_INTENT","PHASE_RESULT"):
            index = record["data"]["derived"]["index"]
            _candidate_prefix_load(j,path,record["data"]["evidence"],record["sequence"],record["kind"],index)
    unchanged("controller_replay")
    return state


def _candidate_controller_operate(request, base):
    """Durable offline startup receipt replay through operate; no live dispatcher."""
    action = request["action"]
    need(type(action) is str and action in ("CREATE","INSPECT","FUNDING_INTENT","FUNDING_RESULT","PHASE_INTENT","PHASE_RESULT","VERIFY"), "candidate_controller_action")
    a,j,s,domain,unchanged = _candidate_preparation_sources(); path = Path(os.path.abspath(base))
    op = Journal(j,path,action == "CREATE",_candidate=True)
    try:
        need(request["expectedTail"] == op.tail(), "tail_compare_and_swap")
        data = request["data"]
        if action == "CREATE":
            manifest = a.candidate_validate(data)
            op.append("CREATE",{"path":str(path),"manifest":manifest,"manifestSha256":sha(encode(manifest)),
                "sourceDomain":domain,"historicalUncertaintyReference":copy.deepcopy(manifest["historicalUncertainty"])})
        else:
            op.stable(); state = _candidate_controller_replay(j,op.chain,path); op.stable()
            if action == "INSPECT": need(data is None, "candidate_controller_inspect_data")
            elif action == "FUNDING_INTENT":
                need(state["intent"] is None, "candidate_controller_original_intent_only")
                op.append("FUNDING_INTENT",_candidate_controller_intent(a,j,state["manifest"],data))
            elif action == "FUNDING_RESULT":
                need(state["intent"] is not None and state["funding"] is None, "candidate_controller_result_requires_original_intent")
                # Prospective replay runs the real cryptographic checker BEFORE write.
                # Invalid/unknown declarations leave the original INTENT/HOLD intact.
                op.append("FUNDING_RESULT",data)
            elif action in ("PHASE_INTENT","PHASE_RESULT"):
                need(state["funding"] is not None, "candidate_prefix_funding_required")
                index = state["cursor"]; _candidate_startup_bound(index)
                raw = encode(data); need(len(raw) <= _candidate_startup_bound(index), "candidate_prefix_payload_bound")
                captured = j.parse(raw)
                derived = _candidate_prefix_intent(a,j,state,captured) if action == "PHASE_INTENT" else _candidate_prefix_result(a,j,state,captured)
                ref = _candidate_prefix_store(a._candidate_helpers()[2],j,op,state,action,raw)
                try: op.append(action,{"evidence":ref,"derived":derived["identity"] if action == "PHASE_INTENT" else derived})
                except Exception as exc: raise Stop("candidate_prefix_commit_uncertain_no_retry_or_adoption") from exc
            else:
                fields(data,("accounts","contextSlot"))
                need(state["funding"] is not None, "candidate_controller_funding_unknown_hold")
                need(state["cursor"] == 0 and state["pending"] is None, "candidate_verify_initial_only")
                bound = {"manifest":state["manifest"],"candidateFunding":state["funding"]}
                verified = verify_states(None,bound,state["manifest"]["phases"][0],"PRE",data["accounts"],context_slot=data["contextSlot"])
        op.stable(); state = _candidate_controller_replay(j,op.chain,path); op.stable()
        need(set(os.listdir(op.fd)) == {"lock"}|{f"{i:08}.json" for i in range(len(op.chain))}
            and all(j.read_file(op.fd,f"{i:08}.json",j.MAX_INPUT) == encode(record) for i,record in enumerate(op.chain)), "candidate_controller_retained_bytes_changed")
        unchanged("controller_before_success"); op.stable()
        return {"schema":CANDIDATE_CONTROLLER_SCHEMA,"path":str(path),"tail":op.tail(),
            "status":"UNKNOWN_HOLD_ORIGINAL_PHASE_ONLY" if state["pending"] else "STARTUP_SUPPLIED_RECEIPTS_REPLAYED_HOST_ONLY" if state["cursor"] == CANDIDATE_STARTUP_COUNT else "VERIFIED_SUPPLIED_FUNDING_RETAINED_HOST_ONLY" if state["funding"] else "UNKNOWN_HOLD_ORIGINAL_FUNDING_ONLY" if state["intent"] else "DURABLE_CANDIDATE_CREATED_HOST_ONLY",
            "originalFundingIdentity":None if state["intent"] is None else state["intent"]["identity"],
            "fundingReceiptSha256":None if state["funding"] is None else state["funding"]["expectedReceiptSha256"],
            "accountVerification":verified if action == "VERIFY" else None,
            "historicalUncertaintyReference":state["manifest"]["historicalUncertainty"],
            "startupPhaseProgressionImplemented":False,"originalPacketSignatureVerified":False,
            "loaderPrefixProgressionImplemented":True,"loaderPrefixCursor":min(state["cursor"],CANDIDATE_PREFIX_COUNT),
            "runtimeSuffixProgressionImplemented":True,"startupCursor":state["cursor"],
            "initializerBindingSha256":None if state["initializerBinding"] is None else sha(j.encode(state["initializerBinding"])),
            "pendingPhaseIdentity":None if state["pending"] is None else state["pending"]["identity"],
            "wholePrefixCostAndOriginalLifetimeAdmission":"OPEN_UNMEASURED",
            "durableReceiptLineageAuthenticated":False,**CANDIDATE_PREPARATION_FLAGS,
            "journalMutationPerformed":action in ("CREATE","FUNDING_INTENT","FUNDING_RESULT","PHASE_INTENT","PHASE_RESULT"),
            "hostControllerJournalMutationPerformed":action in ("CREATE","FUNDING_INTENT","FUNDING_RESULT","PHASE_INTENT","PHASE_RESULT"),
            "operationalJournalAccessed":False}
    finally: op.close()
