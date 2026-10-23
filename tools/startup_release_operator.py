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
import stat
import subprocess
import time

SCHEMA = "piv1-startup-release-operator-v1"
ARTIFACT = {"bytes": 894552, "sha256": "e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99"}
READBACK_SHA = "fbc28f1bc5618459e1178bbea45f3db54fedd21363d7abdcfe4794ef0b0e9adb"
STATE_RULES_SHA = "9546454bac5228c33be1f65ebdd4412fe82a0a5c595479f272aeaa53fdee596e"
MANIFEST_SHA = "c0ff2d36d3cca13ab1acf7c51b2f60050a5da07ad1333c8c3b7d57a4d1918d3c"
SIGNER_BACKEND_SHA = "c4654fcc9928510c1a1d895b1749699692e1d1be5eaa39402a9e74c51450e573"
MAX_PHASES, MAX_BATCH, MAX_RECORDS = 949, 64, 8192
MAX_JOURNAL = 128 * 1024 * 1024
ACTION_EVIDENCE = 256 * 1024 * 1024
MAX_EVIDENCE = 8 * 1024 * 1024 * 1024
LOCK = b"piv1-startup-release-operator-lock-v1\n"
KINDS = {"BUFFER_CREATE", "UPLOAD_WRITE", "BOOTSTRAP", "RECIPIENTS", "INITIALIZER", "HEARTBEAT"}
DESCRIBE_SCRIPT = r'''
const fs=require('node:fs'),c=require(process.env.PIV1_PUBLIC_CHECK_REPO+'/clients/cli/startup-public-check.cjs');
c.verifySources();const raw=fs.readFileSync(0);if(raw.length>1048576)throw Error('input_bound');
const input=JSON.parse(raw),packets=Array.isArray(input)?input:input.packets;
const out=packets.map(x=>{const d=c.decodeWire(x.wireBase64,x.lookupTables);
 if(!d.fullyResolved||d.signatures.some(s=>s.some(v=>v!==0)))throw Error('unsigned_resolved_required');
 return {messageBase64:d.message.toString('base64'),messageSha256:d.messageSha256,feePayer:d.feePayer,
 requiredSigners:d.requiredSigners,accountRoles:d.roles,lookupTables:x.lookupTables};});
let result=out;
if(!Array.isArray(input)){
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


class Stop(ValueError): pass


def need(ok, reason):
    if not ok: raise Stop(reason)


def encode(x):
    return (json.dumps(x, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False) + "\n").encode()


def sha(x): return hashlib.sha256(x).hexdigest()


def fields(x, names): need(type(x) is dict and set(x) == set(names), "object_fields")


def helpers():
    repo = Path(os.environ.get("PIV1_PUBLIC_CHECK_REPO", Path(__file__).resolve().parent.parent))
    path = repo / "tools/collect_startup_initializer_storage.py"
    need(not path.is_symlink() and sha(path.read_bytes()) == READBACK_SHA, "readback_source_pin")
    spec = importlib.util.spec_from_file_location("release_pinned_readback", path)
    r = importlib.util.module_from_spec(spec); spec.loader.exec_module(r)
    c, s, j = r.helpers()
    return r, c, s, j


def describe(j, packets, *, state_plan=None, governance=None):
    """Same pinned Node/SDK and strict decoder, used only on zero-signature wires."""
    repo = j.pins(); raw = encode(packets if state_plan is None else {"packets":packets,"plan":state_plan,"governance":governance})
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


def state_rules():
    path = Path(__file__).with_name("startup_state_rules.py")
    need(not path.is_symlink() and sha(path.read_bytes()) == STATE_RULES_SHA, "startup_rules_source_pin")
    spec = importlib.util.spec_from_file_location("release_startup_rules",path)
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    module.verify_sources(Path(os.environ.get("PIV1_PUBLIC_CHECK_REPO",Path(__file__).resolve().parent.parent)))
    return module


_MANIFEST_MODULE = None


def startup_manifest():
    global _MANIFEST_MODULE
    path = Path(__file__).with_name("startup_manifest.py")
    need(not path.is_symlink() and sha(path.read_bytes()) == MANIFEST_SHA, "startup_manifest_source_pin")
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
    expected = phase["expectedPrestate" if stage == "PRE" else "expectedPoststate"]
    need(set(expected) <= set(accounts), "state_account_missing")
    for address, rule in expected.items():
        if "startupRule" not in rule:
            actual_digest = startup_manifest().digest(operator_api(),c,address,accounts[address]) if full_startup(state["manifest"]) else state_digest(c,accounts[address])
            need(actual_digest == rule, "prestate_mismatch" if stage == "PRE" else "poststate_mismatch")
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


def signing_request(c, state, previous, ids, authorization_sha256):
    """Public request only; no signer backend or proof of key control."""
    need(type(ids) is list and 1 <= len(ids) <= MAX_BATCH and all(type(x) is str for x in ids) and len(set(ids)) == len(ids), "handoff_batch")
    ordered = [p["id"] for p in state["manifest"]["phases"]]
    need(all(pid in state["prepared"] and pid not in state["signed"] and pid not in state["intents"] for pid in ids), "handoff_phase")
    need(ids == sorted(ids, key=ordered.index), "handoff_order")
    packets = []
    for pid in ids:
        prepared = state["prepared"][pid]
        packets.append({"id":pid,"preparedRecordDigest":state["preparedAt"][pid],"preparedSha256":sha(encode(prepared)),
            "unsignedWireSha256":sha(base64.b64decode(prepared["wireBase64"],validate=True)),
            "messageSha256":prepared["reviewed"]["messageSha256"],"requiredSigners":copy.deepcopy(prepared["reviewed"]["requiredSigners"])})
    body = {"schema":"piv1-startup-signing-request-v1","cluster":"testnet","genesisHash":c.GENESIS,"operatorTail":previous,
        "operatorSourceSha256":sha(Path(__file__).read_bytes()),"authorizationSha256":authorization_sha256,
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


def signer_backend_source():
    path=Path(__file__).with_name("startup_signer_backend.cjs")
    need(not path.is_symlink() and sha(path.read_bytes()) == SIGNER_BACKEND_SHA,"signer_backend_source_pin")
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
    """Fixed executable/argv, bounded pipes/deadline; no raw child diagnostics."""
    repo=j.pins();backend=signer_backend_source();raw=encode(payload)
    need(len(raw)<=1048576,"signer_input_bound")
    process=None;selector=selectors.DefaultSelector();streams=();output=bytearray()
    try:
        process=subprocess.Popen([str(j.NODE),str(backend),"--allow-live-sign"],cwd=repo,
            env={"PIV1_PUBLIC_CHECK_REPO":str(repo)},stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,close_fds=True,preexec_fn=disable_signer_core_dumps)
        streams=(process.stdin,process.stdout);position=0;deadline=time.monotonic()+20
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
        need(process.wait(timeout=max(.001,deadline-time.monotonic()))==0,"signer_backend_refused")
        return j.parse(bytes(output))
    except Exception as exc:
        raise Stop("signer_attempt_unresolved") from None
    finally:
        if process is not None and process.poll() is None:process.kill();process.wait()
        selector.close()
        for stream in streams:
            if not stream.closed:stream.close()


def disable_signer_core_dumps():
    resource.setrlimit(resource.RLIMIT_CORE,(0,0))


class Journal:
    """Exclusive append-only successor; partial writes are never repaired/adopted."""
    def __init__(self, j, base, create=False):
        self.j = j; self.path = journal_path(base); self.fd = j.directory(self.path, create); self.lock = None
        try:
            if create: j.write_file(self.fd, "lock", LOCK)
            self.lock = os.open("lock", os.O_RDWR | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=self.fd)
            self.meta = os.fstat(self.lock); j.secure_meta(self.meta); fcntl.flock(self.lock, fcntl.LOCK_EX)
            self.identity = {"device": str(self.meta.st_dev), "inode": str(self.meta.st_ino), "ctimeNs": str(self.meta.st_ctime_ns)}
            self.stable(); names = os.listdir(self.fd)
            need(all(n == "lock" or re.fullmatch(r"[0-9]{8}\.json", n) for n in names), "operator_namespace")
            names = sorted(n for n in names if n != "lock")
            need(len(names) <= MAX_RECORDS and names == [f"{i:08}.json" for i in range(len(names))], "record_gap")
            need(create or names, "missing_create_record"); self.chain = []; self.total = 0
            for i, name in enumerate(names):
                raw = j.read_file(self.fd, name, j.MAX_INPUT); record = j.parse(raw)
                fields(record, ("schema", "sequence", "previousDigest", "lockIdentity", "kind", "data", "digest"))
                need(record["schema"] == SCHEMA and record["sequence"] == str(i) and record["previousDigest"] == self.tail() and record["lockIdentity"] == self.identity, "record_link")
                need(encode(record) == raw and sha(encode({k:v for k,v in record.items() if k != "digest"})) == record["digest"], "record_digest")
                self.chain.append(record); self.total += len(raw)
                need(self.total <= MAX_JOURNAL, "operator_journal_bound")
        except Exception:
            self.close(); raise

    def tail(self): return self.chain[-1]["digest"] if self.chain else None

    def stable(self):
        now = os.stat("lock", dir_fd=self.fd, follow_symlinks=False)
        need((now.st_dev, now.st_ino, now.st_ctime_ns) == (self.meta.st_dev, self.meta.st_ino, self.meta.st_ctime_ns) and self.j.read_file(self.fd,"lock",128) == LOCK, "stable_lock")

    def append(self, kind, data):
        self.stable(); need(len(self.chain) < MAX_RECORDS, "operator_record_bound")
        body = {"schema": SCHEMA, "sequence": str(len(self.chain)), "previousDigest": self.tail(), "lockIdentity": self.identity, "kind": kind, "data": data}
        record = {**body, "digest": sha(encode(body))}; raw = encode(record)
        need(len(raw) <= self.j.MAX_INPUT and self.total + len(raw) <= MAX_JOURNAL, "operator_journal_bound")
        # Reject invalid transitions before committing an immutable record.
        replay(self.j, self.chain + [record])
        try: self.j.write_file(self.fd, f"{len(self.chain):08}.json", raw)
        except Exception as exc: raise Stop("commit_uncertain_no_retry_or_adoption") from exc
        self.chain.append(record); self.total += len(raw); return record

    def close(self):
        if self.lock is not None: fcntl.flock(self.lock, fcntl.LOCK_UN); os.close(self.lock); self.lock = None
        if self.fd is not None: os.close(self.fd); self.fd = None


def replay(j, records):
    need(records and records[0]["kind"] == "CREATE", "create_required")
    first = records[0]["data"]; m = first["manifest"]
    full = full_startup(m)
    adapter = startup_manifest() if full else None
    c = helpers()[1] if full or any(r["kind"] in ("HANDOFF","SIGN_ATTEMPT") for r in records) else None
    if full: adapter.validate(operator_api(),j,m)
    state = {"manifest": m, "manifestSha256": sha(encode(m)), "cursor": 0, "prepared": {}, "signed": {}, "intents": {}, "terminal": {}, "seal": None, "preVote": None, "postVote": None,
        "reserved": 0, "actual": 0, "floor": j.u64(first["contextFloor"]), "lastSlot": j.u64(first["lastFinalizedSlot"]), "failed": False, "signedHistory": [], "attempt": None, "transportMode": None, "runtimeFacts":{}, "runtimeClock":None,
        "baseParentPath":first["baseParentPath"],"preparedAt":{},"handoff":None,"signerAttempt":None,"signerAttempts":{},"quoteByPhase":{}}
    need(first["historicalLedger"] == m["historicalLedger"], "historical_ledger_rebind")
    ids = [p["id"] for p in m["phases"]]
    for record in records[1:]:
        kind, d = record["kind"], record["data"]
        unresolved = any(pid not in state["terminal"] for pid in state["intents"])
        need(not state["failed"], "record_after_finalized_failure")
        if state["signerAttempt"] is not None: need(kind=="IMPORT","signer_attempt_unresolved")
        if kind in ("BIND_SEAL", "PREPARE", "HANDOFF", "IMPORT", "PRE_VOTE", "POST_VOTE"):
            need(not unresolved, "unresolved_intent_blocks_transition")
        if kind == "BIND_SEAL":
            need(state["seal"] is None and not state["failed"] and state["cursor"] < len(ids) and m["phases"][state["cursor"]]["kind"] == "INITIALIZER", "seal_transition")
            need(d["baseParentTail"] == m["parentTail"], "seal_original_parent_binding")
            need(state["transportMode"] is None or state["transportMode"] == d["transportMode"], "transport_mode_changed")
            state["transportMode"] = d["transportMode"]
            if full: adapter.validate_seal(operator_api(),j,state,d)
            state["seal"] = d
            state["handoff"] = None
        elif kind == "PREPARE":
            capture=type(d) is dict
            if capture:
                fields(d,("entries","quote"));need(full,"full_startup_required_for_quote")
                entries=d["entries"];need(type(entries) is list and 1<=len(entries)<=23,"quote_batch_bound")
                next_floor=verify_quote(c,j,state,entries,d["quote"],record)
            else:entries=d
            need(type(entries) is list and 1 <= len(entries) <= MAX_BATCH and len({p["id"] for p in entries}) == len(entries), "prepare_replay_batch")
            for p in entries:
                need(p["id"] in ids and p["id"] not in state["intents"] and ids.index(p["id"]) >= state["cursor"], "prepare_after_intent")
                phase = m["phases"][ids.index(p["id"])]
                need(semantic(p["reviewed"]) == semantic(phase_template(state, phase)), "prepared_semantics_changed")
                need(p["lifetime"]["minimumRemainingBlockHeights"] == phase["minimumRemainingBlockHeights"] and j.u64(p["feeQuoteLamports"]) <= j.u64(phase["feeCapLamports"]), "prepared_limits_changed")
                if full: need(p["feeQuoteLamports"] == phase["feeCapLamports"], "startup_exact_fee_profile")
                prior = state["prepared"].get(p["id"])
                if prior and prior["lifetime"]["recentBlockhash"] == p["lifetime"]["recentBlockhash"]:
                    need(prior["lifetime"]["lastValidBlockHeight"] == p["lifetime"]["lastValidBlockHeight"] and j.u64(p["lifetime"]["currentBlockHeight"]) >= j.u64(prior["lifetime"]["currentBlockHeight"]), "same_blockhash_lifetime_changed")
                state["prepared"][p["id"]] = p
                state["preparedAt"][p["id"]] = record["digest"]
                state["signed"].pop(p["id"], None)
                if capture:state["quoteByPhase"][p["id"]]=d["quote"]
                else:state["quoteByPhase"].pop(p["id"],None)
            if capture:
                state["floor"]=max(state["floor"],next_floor)
                state["transportMode"]=d["quote"]["transportMode"]
            state["handoff"] = None
        elif kind == "HANDOFF":
            fields(d,("schema","cluster","genesisHash","operatorTail","operatorSourceSha256","authorizationSha256","baseParentPath","baseParentTail","manifestSha256","sealSha256","packets","requestId"))
            j.digest(d["authorizationSha256"])
            need(type(d["packets"]) is list and all(type(p) is dict and "id" in p for p in d["packets"]), "handoff_batch")
            need(d == signing_request(c,state,record["previousDigest"],[p["id"] for p in d["packets"]],d["authorizationSha256"]), "handoff_replay_binding")
            captured_authorization(state,[p["id"] for p in d["packets"]],d["authorizationSha256"])
            state["handoff"] = d
        elif kind == "SIGN_ATTEMPT":
            fields(d,("request","authorization","signerBackendSha256","signerProfileSha256","mode"))
            need(full and not unresolved,"canonical_signer_boundary")
            a=d["authorization"];auth_raw=encode(a);auth_pin=authorization(j,state,auth_raw,sha(auth_raw),allow_expired=True,require_signer=True)
            need(d["request"]["requestId"] not in state["signerAttempts"],"signer_request_already_attempted")
            need(d["request"]==signing_request(c,state,record["previousDigest"],[p["id"] for p in d["request"]["packets"]],auth_pin),"signer_attempt_request_binding")
            captured_authorization(state,[p["id"] for p in d["request"]["packets"]],auth_pin)
            need(d["signerBackendSha256"]==a["signerBackendSha256"]==SIGNER_BACKEND_SHA and d["signerProfileSha256"]==a["signerProfileSha256"],"signer_attempt_profile_binding")
            need(d["mode"] in ("LOCAL_TESTNET_BACKEND","OFFLINE_INJECTED_TEST"),"signer_attempt_mode")
            if d["mode"]=="OFFLINE_INJECTED_TEST":
                need(state["transportMode"] in (None,"OFFLINE_INJECTED_TEST"),"transport_mode_changed")
                state["transportMode"]="OFFLINE_INJECTED_TEST"
            else:need(state["transportMode"]!="OFFLINE_INJECTED_TEST","injected_history_cannot_sign_live")
            state["handoff"]=d["request"];state["signerAttempt"]=d;state["signerAttempts"][d["request"]["requestId"]]=d
        elif kind == "IMPORT":
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
                need(not any(x["wireSha256"] == p["wireSha256"] or x["signatureHex"] == p["signatureHex"] for x in state["signedHistory"]), "signed_history_replay")
                state["signed"][p["id"]] = {**p,**({"requestId":d["requestId"],"authorizationSha256":d["authorizationSha256"]} if bound else {})}
                state["signedHistory"].append(p)
            state["handoff"] = None
            state["signerAttempt"] = None
        elif kind == "ATTEMPT":
            need(state["cursor"] < len(ids) and d["id"] == ids[state["cursor"]] and d["stage"] in ("SEND", "RECOVER", "PRE_VOTE", "POST_VOTE"), "attempt_phase")
            need((d["stage"] == "RECOVER") == unresolved, "attempt_intent_relation")
            need(d["transportMode"] in ("FIXED_ORIGIN_TLS", "OFFLINE_INJECTED_TEST") and (state["transportMode"] is None or state["transportMode"] == d["transportMode"]), "transport_mode_changed")
            need(full or d["transportMode"] == "OFFLINE_INJECTED_TEST", "full_startup_required_for_live_transport")
            state["transportMode"] = d["transportMode"]
            state["attempt"] = d
        elif kind == "INTENT":
            phase = d["id"]
            need(not state["failed"] and state["cursor"] < len(ids) and phase == ids[state["cursor"]] and phase in state["signed"] and phase not in state["intents"], "intent_order_or_duplicate")
            need(d["wireSha256"] == state["signed"][phase]["wireSha256"] and d["signatureHex"] == state["signed"][phase]["signatureHex"] and d["signature"] == state["signed"][phase]["signature"] and d["feeLamports"] == state["prepared"][phase]["feeQuoteLamports"], "intent_signed_binding")
            need(d["preparedSha256"] == state["signed"][phase]["preparedSha256"] and state["attempt"] is not None and state["attempt"]["stage"] == "SEND" and d["evidencePath"] == state["attempt"]["evidencePath"] and d["transportMode"] == state["attempt"]["transportMode"], "intent_attempt_binding")
            if "requestId" in state["signed"][phase]: need(d["authorizationSha256"] == state["signed"][phase]["authorizationSha256"], "signing_authorization_changed")
            captured_authorization(state,[phase],d["authorizationSha256"])
            if full: need("requestId" in state["signed"][phase], "signing_response_required")
            need(j.u64(d["prestateContextSlot"]) >= state["floor"], "intent_prestate_stale")
            p = active_phase(c,state,m["phases"][state["cursor"]]) if full else m["phases"][state["cursor"]]
            if full:
                need(d["runtimePreFacts"] == state["runtimeFacts"], "startup_intent_facts_binding")
                state_rules().saved_clock(d["runtimePreClock"])
                need(j.u64(d["runtimePreClock"]["slot"]) >= state["floor"] and d["runtimePreClock"]["contextSlot"] == d["prestateContextSlot"], "startup_intent_clock_binding")
                if state["runtimeClock"] is not None: need(int(d["runtimePreClock"]["unixTimestamp"]) >= int(state["runtimeClock"]["unixTimestamp"]), "startup_preclock_regression")
                need(d["coverage"] == adapter.coverage(operator_api(),c,state,p,"PRE",state["prepared"][phase]["reviewed"],d["runtimePreFacts"],d["runtimePreClock"]), "startup_intent_coverage_binding")
            if typed_phase(p):
                need(d["runtimePreFacts"] == state["runtimeFacts"], "startup_intent_facts_binding")
                if phase_clock(p):
                    state_rules().saved_clock(d["runtimePreClock"])
                    need(j.u64(d["runtimePreClock"]["slot"]) >= state["floor"] and d["runtimePreClock"]["contextSlot"] == d["prestateContextSlot"], "startup_intent_clock_binding")
                    if state["runtimeClock"] is not None: need(int(d["runtimePreClock"]["unixTimestamp"]) >= int(state["runtimeClock"]["unixTimestamp"]), "startup_preclock_regression")
            if p["kind"] == "INITIALIZER" and 4 <= p["initializerIndex"] <= 7: need(state["preVote"] is not None, "intent_causal_pre_vote")
            if p["kind"] == "INITIALIZER" and p["initializerIndex"] == 8: need(state["postVote"] is not None, "intent_post_vote")
            state["intents"][phase] = d; state["reserved"] += j.u64(d["feeLamports"])
            state["handoff"] = None
            state["floor"] = max(state["floor"], j.u64(d["prestateContextSlot"]))
        elif kind in ("UNKNOWN", "SUBMITTED"):
            need(d["id"] in state["intents"] and d["id"] not in state["terminal"], "unknown_without_intent")
            if kind == "SUBMITTED": need(d["signature"] == state["intents"][d["id"]]["signature"], "submitted_signature_changed")
        elif kind in ("FINALIZED", "FAILED"):
            phase = d["id"]; need(phase in state["intents"] and phase not in state["terminal"] and phase == ids[state["cursor"]], "terminal_order")
            slot = j.u64(d["slot"]); need(slot > state["lastSlot"], "global_slot_not_later")
            need(d["feeLamports"] == state["intents"][phase]["feeLamports"], "terminal_fee_changed")
            need(d["wireSha256"] == state["intents"][phase]["wireSha256"] and d["signatureHex"] == state["intents"][phase]["signatureHex"], "terminal_packet_changed")
            need(state["attempt"] is not None and state["attempt"]["stage"] == "RECOVER" and d["evidencePath"] == str(Path(state["attempt"]["evidencePath"]) / "collection"), "terminal_recovery_binding")
            need(j.u64(d["contextSlot"]) >= max(state["floor"], slot) if kind == "FINALIZED" else j.u64(d["contextSlot"]) == slot, "terminal_context_stale")
            p = active_phase(c,state,m["phases"][state["cursor"]]) if full else m["phases"][state["cursor"]]
            if kind == "FINALIZED" and full:
                post_clock=d["runtimePostClock"]; state_rules().saved_clock(post_clock)
                need(post_clock["contextSlot"] == d["contextSlot"], "startup_terminal_clock_binding")
                state_rules().window(state["intents"][phase]["runtimePreClock"],post_clock,slot)
                snapshot_accounts=verify_full_snapshot(c,j,state,p,d) if adapter.large_key() in adapter.required_keys(p) else None
                need(d.get("fullSnapshot") is None if snapshot_accounts is None else True, "startup_unexpected_full_snapshot")
                need(d["coverage"] == adapter.coverage(operator_api(),c,state,p,"POST",state["prepared"][phase]["reviewed"],d["runtimeFacts"],post_clock,snapshot_accounts,full_snapshot=d.get("fullSnapshot")), "startup_terminal_coverage_binding")
                state["runtimeClock"] = post_clock
            if kind == "FINALIZED" and typed_phase(p):
                rules = state_rules(); post_clock = d["runtimePostClock"]
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
            state["preVote" if kind == "PRE_VOTE" else "postVote"] = d
            state["floor"] = max(state["floor"], j.u64(d["contextSlot"]))
        else: raise Stop("operator_record_kind")
        need(state["reserved"] <= j.u64(m["proposedFeeBudget"]["feeCapLamports"]) and state["actual"] <= j.u64(m["proposedFeeBudget"]["feeCapLamports"]), "global_fee_cap")
    return state


def authorization(j, state, raw, pinned, *, allow_expired=False, require_signer=False):
    need(type(raw) is bytes and pinned is not None and sha(raw) == j.digest(pinned), "launcher_authorization_required")
    a = j.parse(raw)
    v2=a.get("schema")=="piv1-launcher-release-authorization-v2"
    fields(a, ("schema", "manifestSha256", "baseParentTail", "operatorSourceSha256", "expiresUnixTimestamp", "scope", "feeBudgetNamespace")+(("signerBackendSha256","signerProfileSha256") if v2 else ()))
    need(a["schema"] in ("piv1-launcher-release-authorization-v1","piv1-launcher-release-authorization-v2") and a["manifestSha256"] == state["manifestSha256"] and a["baseParentTail"] == state["manifest"]["parentTail"] and a["operatorSourceSha256"] == sha(Path(__file__).read_bytes()) and a["scope"] == "EXACT_BOUNDED_TESTNET_RELEASE", "launcher_authorization_binding")
    need(a["feeBudgetNamespace"] == state["manifest"]["proposedFeeBudget"]["namespace"], "launcher_fee_namespace_binding")
    expires = j.u64(a["expiresUnixTimestamp"])
    need(allow_expired or expires > int(time.time()), "launcher_authorization_expired")
    if v2:
        j.digest(a["signerProfileSha256"])
        need(encode(a)==raw and a["signerBackendSha256"]==SIGNER_BACKEND_SHA,"signer_authorization_binding")
        signer_backend_source()
    need(not require_signer or v2,"signer_authorization_v2_required")
    return pinned


def phase_template(state, phase):
    if phase["kind"] != "INITIALIZER": return phase["reviewed"]
    need(state["seal"] is not None, "initializer_seal_required")
    return state["seal"]["reviewedPackets"][phase["initializerIndex"]]


def check_signed(j, frozen, phase, prepared, signed, observation=None):
    packet = {"signedWireBase64": signed["wireBase64"], "reviewed": prepared["reviewed"]}
    p = {"id": phase["id"], "packet": packet, "lifetime": prepared["lifetime"], "feeQuoteLamports": prepared["feeQuoteLamports"], "feeCapLamports": phase["feeCapLamports"], "expectedPoststate": observation["poststate"]["accounts"] if observation and observation["poststate"] else {}}
    trace = []
    for event in ("PREPARED", "SUBMISSION_START_RECORDED") + (("FINALIZED" if observation["status"]["err"] is None else "FAILED",) if observation else ()):
        entry = {"previousDigest": None, "phase": phase["id"], "state": event, "packetSha256": signed["wireSha256"], "signatureHex": signed["signatureHex"], "frozen": frozen, "observation": observation if event in ("FINALIZED", "FAILED") else None}
        # entryDigest uses canonical JSON without a trailing newline.
        if trace: entry["previousDigest"] = sha(encode(trace[-1]).rstrip(b"\n"))
        trace.append(entry)
    report = j._node_request([{"schema": "piv1-startup-public-check-v1", "frozen": frozen, "cumulativeFeeCapLamports": phase["feeCapLamports"], "phases": [p], "trace": trace}])[0]
    need(report.get("reason") in ("PENDING_OR_UNKNOWN_NO_REPLACEMENT", "ALL_DECLARED_PHASES_FINALIZED_WITH_EXPECTED_POSTSTATE", "FINALIZED_FAILURE"), "signed_packet_or_receipt_rejected:" + str(report.get("reason")))
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


def bounded_transport(c, transport):
    selected = c.tls_post if transport is None else transport
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


def evidence_store(c, j, operator, label, state, transport):
    # Full raw Buffer responses remain intact. Bound total retained evidence and
    # reserve one action envelope plus the inherited 2-GiB free-space floor.
    root=evidence_root(j,operator)
    output = root / f"{len(operator.chain):08}-{label.lower()}"
    operator.append("ATTEMPT", {"id": state["manifest"]["phases"][state["cursor"]]["id"], "stage": label, "evidencePath": str(output), "transportMode": "OFFLINE_INJECTED_TEST" if transport is not None else "FIXED_ORIGIN_TLS"})
    # Failure after ATTEMPT keeps its immutable identity; later recovery creates
    # a new attempt. Existing/partial output is never reused or adopted.
    return c.Store(output)


def evidence_root(j,operator):
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
    return root


def quote_store(c,j,operator):
    root=evidence_root(j,operator);prefix=f"{len(operator.chain):08}-prepare-"
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


def quote_values(c,j,state,entries,initial,values):
    need(len(initial)==2 and initial[0]==c.GENESIS,"genesis_mismatch")
    original=latest_quote(c,state,initial[1]);floor=max(state["floor"],j.u64(original["contextSlot"]))
    need(len(values)==len(entries)+2,"quote_response_count")
    height=c.u64(values[-2]);valid=values[-1]
    need(type(valid) is dict and valid.get("value") is True and type(valid.get("context")) is dict and c.u64(valid["context"].get("slot"))>=floor,"quote_blockhash_invalid_or_stale")
    highest=max(floor,c.u64(valid["context"]["slot"]))
    for p,result in zip(entries,values[:-2]):
        fields(p,("id","wireBase64","reviewed","feeQuoteLamports","lifetime"))
        need(type(result) is dict and result.get("value") is not None and type(result.get("context")) is dict and c.u64(result["context"].get("slot"))>=floor,"quote_fee_context")
        phase=next(x for x in state["manifest"]["phases"] if x["id"]==p["id"])
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
    store=quote_store(c,j,op);rpc=Rpc(c,j,store,transport)
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
        return entries,{**binding,"manifestSha256Receipt":manifest["sha256"]}
    finally:store.close()


def verify_quote(c,j,state,entries,quote,record):
    fields(quote,("schema","path","operatorSequence","operatorTail","operatorSourceSha256","baseParentPath","baseParentTail","manifestSha256","authorization","authorizationSha256","minimumContextSlot","transportMode","orderedPreparedSha256","manifestSha256Receipt"))
    root=journal_path(state["baseParentPath"]);root=root.with_name(root.name+".evidence");path=Path(quote["path"])
    need(path.parent==root and re.fullmatch(re.escape(record["sequence"].zfill(8))+r"-prepare-[0-9]{2}",path.name),"quote_path_binding")
    need(quote["schema"]=="piv1-startup-lifetime-quote-v1" and quote["operatorSequence"]==record["sequence"] and quote["operatorTail"]==record["previousDigest"] and quote["operatorSourceSha256"]==sha(Path(__file__).read_bytes()),"quote_source_or_predecessor_binding")
    need(quote["baseParentPath"]==state["baseParentPath"] and quote["baseParentTail"]==state["manifest"]["parentTail"] and quote["manifestSha256"]==state["manifestSha256"] and quote["minimumContextSlot"]==str(state["floor"]),"quote_parent_binding")
    need(quote["orderedPreparedSha256"]==[sha(encode(p)) for p in entries],"quote_prepared_order_binding")
    need(quote["transportMode"] in ("FIXED_ORIGIN_TLS","OFFLINE_INJECTED_TEST") and state["transportMode"] in (None,quote["transportMode"]),"transport_mode_changed")
    raw_auth=encode(quote["authorization"]);authorization(j,state,raw_auth,quote["authorizationSha256"],allow_expired=True)
    fd=j.directory(path)
    try:
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
        return quote_values(c,j,state,entries,initial,values)
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
        for artifact in manifest["artifacts"]:
            fields(artifact,("name","bytes","sha256"));value=j.read_file(fd,artifact["name"],c.MAX_JSON)
            need(len(value)==artifact["bytes"] and sha(value)==artifact["sha256"],"startup_full_snapshot_artifact_hash")
        saved=j.read_file(fd,"snapshot.json",c.MAX_JSON);need(sha(saved)==receipt["snapshotSha256"],"startup_full_snapshot_hash")
        snapshot=j.parse(saved);fields(snapshot,("contextSlot","accounts"))
        need(snapshot["contextSlot"]==terminal["contextSlot"] and set(snapshot["accounts"])==set(receipt["requiredKeys"]),"startup_full_snapshot_saved_context")
        # This exact snapshot already passed the four typed rules before commit;
        # coverage rederives every complete account digest and causal fact below.
        return snapshot["accounts"]
    finally:os.close(fd)


def gate(r, c, j, operator, state, stage, transport):
    seal=state["seal"]; need(seal is not None,"seal_required")
    store=evidence_store(c,j,operator,stage,state,transport); output=store.path; store.close()
    # Collector requires an exclusive new directory, so use a deterministic child.
    report=r.collect({"schema":r.SCHEMA,"expectedTail":seal["parentTail"],"stage":stage,"minimumContextSlot":str(state["floor"])},seal["parentPath"],output/"readback",_transport=bounded_transport(c,transport) if transport is not None else None)
    need(report["status"]=="VERIFIED_COLLECTED_STORAGE_AND_CLOCK_EVIDENCE_ONLY" and report["sealDataSha256"]==seal["sealDataSha256"],"storage_clock_gate_failed:"+str(report.get("reason")))
    need(j.u64(report["snapshotContextSlot"])>=state["floor"],"gate_context_stale")
    return {"id":state["manifest"]["phases"][state["cursor"]]["id"],"contextSlot":report["snapshotContextSlot"],"sealDataSha256":report["sealDataSha256"],"snapshotSha256":report["snapshotSha256"],"evidencePath":str(output/"readback"),"manifestSha256":report["manifestReceipt"]["sha256"],"transportMode":report["transportMode"]}


def operate(request, base, *, authorization_raw=None, authorization_sha256=None, live=False, live_sign=False, _transport=None, _signer=None):
    fields(request,("schema","action","expectedTail","data")); need(request["schema"]==SCHEMA,"schema")
    action=request["action"]
    if action is None: return {"status":"INCOMPLETE","authorized":False,"liveReady":False}
    need(action in ("CREATE","INSPECT","PREPARE","HANDOFF","SIGN","IMPORT","BIND_SEAL","SEND","RECOVER"),"action")
    need(_transport is None or callable(_transport),"transport_callable")
    need(_signer is None or callable(_signer),"signer_callable")
    r,c,s,j=helpers(); base=Path(os.path.abspath(base)); op=Journal(j,base,action=="CREATE")
    try:
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
            elif action=="PREPARE":
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
                reviewed=describe(j,[{"wireBase64":p["wireBase64"],"lookupTables":phase_template(state,m["phases"][start+i])["lookupTables"]} for i,p in enumerate(data)])
                if capture:
                    for i,(p,review) in enumerate(zip(data,reviewed)):
                        fields(p,("id","wireBase64"));phase=m["phases"][start+i]
                        need(p["id"]==phase["id"] and semantic(review)==semantic(phase_template(state,phase)),"prepared_semantics_changed")
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
                op.append("PREPARE",{"entries":entries,"quote":captured} if capture else entries)
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
                op.stable();need(replay(j,op.chain)["signerAttempt"]==attempt,"signer_attempt_changed")
                imported=import_response(c,j,state,response,authorization_pin)
                authorization(j,state,authorization_raw,authorization_sha256,require_signer=True)
                op.append("IMPORT",imported)
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
            elif action in ("SEND","RECOVER"):
                authorization_pin=authorization(j,state,authorization_raw,authorization_sha256,allow_expired=action=="RECOVER")
                need(_transport is not None or live,"live_transport_not_enabled")
                need(_transport is not None or full_startup(m),"full_startup_required_for_live_transport")
                need(state["transportMode"] is None or state["transportMode"] == ("OFFLINE_INJECTED_TEST" if _transport is not None else "FIXED_ORIGIN_TLS"), "transport_mode_changed")
                need(cursor<len(m["phases"]),"workflow_complete"); phase=active_phase(c,state,m["phases"][cursor]); pid=phase["id"]
                need(data=={"id":pid} and pid in state["signed"],"current_signed_phase_required")
                prepared=state["prepared"][pid]; signed=state["signed"][pid]
                if action=="SEND":captured_authorization(state,[pid],authorization_pin)
                check_signed(j,frozen(state),phase,prepared,signed)
                if action=="SEND":
                    need(pid not in state["intents"],"intent_exists_recovery_only")
                    if "requestId" in signed: need(signed["authorizationSha256"] == authorization_pin,"signing_authorization_changed")
                    if full_startup(m): need("requestId" in signed,"signing_response_required")
                    if phase["kind"]=="INITIALIZER":
                        idx=phase["initializerIndex"]
                        if idx==4:
                            evidence=gate(r,c,j,op,state,"PRE_VOTE",_transport); op.append("PRE_VOTE",evidence); state=replay(j,op.chain)
                        if 4<=idx<=7: need(state["preVote"] is not None,"causal_pre_vote_required")
                        if idx==8:
                            evidence=gate(r,c,j,op,state,"POST_VOTE",_transport); op.append("POST_VOTE",evidence); state=replay(j,op.chain)
                    store=evidence_store(c,j,op,"SEND",state,_transport); rpc=Rpc(c,j,store,_transport)
                    try:
                        identity=rpc.query([("getGenesisHash",[])] + ([("getVersion",[])] if full_startup(m) else []))
                        need(identity[0]==c.GENESIS,"genesis_mismatch")
                        if full_startup(m): need(identity[1] == startup_manifest().oracle()["version"], "startup_runtime_version_changed")
                        life=prepared["lifetime"]
                        rule_module = state_rules() if typed_phase(phase) or full_startup(m) else None
                        clock_key = rule_module.CLOCK if phase_clock(phase) else None
                        account_keys = startup_manifest().required_keys(phase) if full_startup(m) else list(phase["expectedPrestate"])
                        if clock_key and clock_key not in account_keys: account_keys.append(clock_key)
                        need(len(account_keys) <= 64,"startup_clock_account_capacity")
                        height,valid,quote,accounts,latest=rpc.query([
                            ("getBlockHeight",[{"commitment":"finalized","minContextSlot":state["floor"]}]),
                            ("isBlockhashValid",[life["recentBlockhash"],{"commitment":"finalized","minContextSlot":state["floor"]}]),
                            ("getFeeForMessage",[prepared["reviewed"]["messageBase64"],{"commitment":"finalized","minContextSlot":state["floor"]}]),
                            ("getMultipleAccounts",[account_keys,{"encoding":"base64","commitment":"finalized","minContextSlot":state["floor"]}]),
                            ("getLatestBlockhash",[{"commitment":"finalized","minContextSlot":state["floor"]}])])
                        now=c.u64(height); last=j.u64(life["lastValidBlockHeight"])
                        need(type(latest) is dict and c.u64(latest["context"]["slot"]) >= state["floor"],"fresh_blockhash_context")
                        if pid not in state["quoteByPhase"]:
                            need(latest["value"]["blockhash"] == life["recentBlockhash"] and c.u64(latest["value"]["lastValidBlockHeight"]) == last, "fresh_blockhash_lifetime_binding")
                        else:latest_quote(c,state,latest)
                        need(now >= j.u64(life["currentBlockHeight"]), "blockheight_regression")
                        need(now<=last and last-now>=j.u64(phase["minimumRemainingBlockHeights"]),"fresh_lifetime_margin")
                        need(type(valid) is dict and valid.get("value") is True and c.u64(valid["context"]["slot"])>=state["floor"],"blockhash_invalid_or_stale")
                        need(type(quote) is dict and quote.get("value") is not None and c.u64(quote["context"]["slot"])>=state["floor"] and str(c.u64(quote["value"]))==prepared["feeQuoteLamports"],"fresh_fee_quote_mismatch")
                        need(type(accounts) is dict and c.u64(accounts["context"]["slot"])>=state["floor"] and type(accounts.get("value")) is list and len(accounts["value"])==len(account_keys),"prestate_context")
                        actual={key:startup_manifest().normalize_account(c,key,raw) if full_startup(m) else c.normalize_account(raw) for key,raw in zip(account_keys,accounts["value"])}
                        pre_clock = rule_module.clock(c,actual[clock_key],accounts["context"]["slot"],state["floor"]) if clock_key else None
                        if pre_clock and state["runtimeClock"]:
                            need(int(pre_clock["unixTimestamp"]) >= int(state["runtimeClock"]["unixTimestamp"]),"startup_preclock_regression")
                        pre_facts = verify_states(c,state,phase,"PRE",actual,pre_clock=pre_clock)
                        if rule_module:
                            sizes = startup_manifest().rent_sizes(phase) if full_startup(m) else rule_module.rent_sizes(state,phase)
                            for start in range(0,len(sizes),5):
                                batch=sizes[start:start+5]
                                rents=rpc.query([("getMinimumBalanceForRentExemption",[size,{"commitment":"finalized"}]) for size in batch])
                                floors = startup_manifest().oracle()["rents"] if full_startup(m) else {str(k):v for k,v in rule_module.RENT_FLOORS.items()}
                                need(all(str(c.u64(value)) == floors[str(size)] for size,value in zip(batch,rents)),"startup_fresh_rent_profile_changed")
                        fee=j.u64(prepared["feeQuoteLamports"]); need(state["reserved"]+fee<=j.u64(m["proposedFeeBudget"]["feeCapLamports"]),"global_fee_cap")
                        authorization(j,state,authorization_raw,authorization_sha256)
                        # No send can occur until this complete intent and its directory are fsynced.
                        op.append("INTENT",{"id":pid,"preparedSha256":signed["preparedSha256"],"wireSha256":signed["wireSha256"],"signature":signed["signature"],"signatureHex":signed["signatureHex"],"feeLamports":prepared["feeQuoteLamports"],"authorizationSha256":authorization_pin,"prestateContextSlot":str(accounts["context"]["slot"]),"evidencePath":str(store.path),"transportMode":"OFFLINE_INJECTED_TEST" if _transport is not None else "FIXED_ORIGIN_TLS","runtimePreFacts":pre_facts,"runtimePreClock":pre_clock,**({"coverage":startup_manifest().coverage(operator_api(),c,state,phase,"PRE",prepared["reviewed"],pre_facts,pre_clock,actual)} if full_startup(m) else {})})
                        try:
                            result=rpc.query([("sendTransaction",[signed["wireBase64"],{"encoding":"base64","skipPreflight":False,"maxRetries":0,"preflightCommitment":"finalized","minContextSlot":state["floor"]}])])[0]
                            need(result==signed["signature"],"send_signature_mismatch")
                            op.append("SUBMITTED",{"id":pid,"signature":signed["signature"]})
                        except Exception:
                            op.append("UNKNOWN",{"id":pid,"reason":"SEND_UNCERTAIN_NO_RESEND_OR_REPLACEMENT"})
                    finally: store.close()
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
                    report=c.collect({"schema":c.SCHEMA,"wireBase64":signed["wireBase64"],"expectedWireSha256":signed["wireSha256"],"expectedMessageSha256":prepared["reviewed"]["messageSha256"],"signature":signed["signature"],"expectedFeeLamports":prepared["feeQuoteLamports"],"accountKeys":startup_manifest().proof_keys(phase) if full_startup(m) else account_keys,"minimumContextSlot":str(state["floor"]),"includeExpiry":False},output/"collection",_transport=bounded_transport(c,_transport) if _transport is not None else None)
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
                        op.append(kind,{"id":pid,"slot":obs["status"]["slot"],"contextSlot":snapshot["contextSlot"] if kind == "FINALIZED" else obs["status"]["slot"],"feeLamports":obs["transaction"]["feeLamports"],"wireSha256":signed["wireSha256"],"signatureHex":signed["signatureHex"],"checkerReport":checked,"evidencePath":str(output/"collection"),"manifestSha256":report["manifestReceipt"]["sha256"],"runtimeFacts":next_facts if kind == "FINALIZED" else state["runtimeFacts"],"runtimePostClock":post_clock if kind == "FINALIZED" else None,**({"coverage":startup_manifest().coverage(operator_api(),c,state,phase,"POST",prepared["reviewed"],next_facts,post_clock,actual,full_snapshot=full_snapshot),"fullSnapshot":full_snapshot} if full_startup(m) and kind == "FINALIZED" else {})})
                    else: op.append("UNKNOWN",{"id":pid,"reason":"ORIGINAL_SIGNATURE_UNRESOLVED_NO_REPLACEMENT","evidencePath":str(output/"collection"),"manifestSha256":report["manifestReceipt"]["sha256"]})
        state=replay(j,op.chain)
        unresolved=state["cursor"]<len(state["manifest"]["phases"]) and state["manifest"]["phases"][state["cursor"]]["id"] in state["intents"]
        possible_live_signer=any(a["mode"]=="LOCAL_TESTNET_BACKEND" for a in state["signerAttempts"].values())
        return {"status":"SIGNER_ATTEMPT_UNRESOLVED_ORIGINAL_RESPONSE_ONLY" if state["signerAttempt"] is not None else "FINALIZED_FAILURE_STOP" if state["failed"] else "ALL_PHASES_RECONCILED" if state["cursor"]==len(state["manifest"]["phases"]) else "UNKNOWN_OR_SUBMITTED_RECOVERY_ONLY" if unresolved else "DURABLE_RELEASE_PREPARATION_ONLY", "tail":op.tail(),"completedPhases":state["cursor"],"phaseCapacity":MAX_PHASES,"historicalLedger":copy.deepcopy(state["manifest"]["historicalLedger"]),"proposedFeeLedger":{"namespace":state["manifest"]["proposedFeeBudget"]["namespace"],"status":"PROPOSAL_ONLY_NOT_AUTHORIZED","feeCapLamports":state["manifest"]["proposedFeeBudget"]["feeCapLamports"],"reservedFeeLamports":str(state["reserved"]),"actualFeeLamports":str(state["actual"])},"pendingSigningRequest":copy.deepcopy(state["handoff"]),"pendingSignerAttempt":copy.deepcopy(state["signerAttempt"]),"signingPackets":[copy.deepcopy(state["prepared"][p["id"]]) for p in state["handoff"]["packets"]] if state["handoff"] else [],"liveReady":False,"founderAuthorizationCryptographicallyVerified":False,"completeStateOracle":False,"mandatoryStartupManifestConsumed":full_startup(state["manifest"]),"signerInvoked":None if possible_live_signer else False,"signerInvocationStatus":"MAY_HAVE_RUN_NO_CONTROL_PROOF" if possible_live_signer else "OFFLINE_INJECTED_ONLY" if state["signerAttempts"] else "NOT_INVOKED","signerControlProven":False}
    finally: op.close()


def main(argv):
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("request"); p.add_argument("base_parent"); p.add_argument("--authorization-receipt"); p.add_argument("--authorization-sha256"); p.add_argument("--allow-live-send",action="store_true"); p.add_argument("--allow-live-sign",action="store_true")
    args=p.parse_args(argv)
    # All current preparations omit authorization and the live flag.
    r,c,s,j=helpers(); raw=Path(args.request).read_bytes(); request=j.parse(raw)
    auth=Path(args.authorization_receipt).read_bytes() if args.authorization_receipt else None
    result=operate(request,args.base_parent,authorization_raw=auth,authorization_sha256=args.authorization_sha256,live=args.allow_live_send,live_sign=args.allow_live_sign)
    print(json.dumps(result,sort_keys=True,indent=2))


if __name__=="__main__":
    try: main(__import__('sys').argv[1:])
    except Exception as exc:
        print(json.dumps({"status":"STOP","reason":str(exc) if isinstance(exc,(Stop,ValueError)) else "operator_failure","liveReady":False}),file=__import__('sys').stderr)
        raise SystemExit(1)
