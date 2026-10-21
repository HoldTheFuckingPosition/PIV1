#!/usr/bin/env python3
"""Durable Testnet release workflow. No wallet reader or signer.

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
import selectors
import stat
import subprocess
import time

SCHEMA = "piv1-startup-release-operator-v1"
ARTIFACT = {"bytes": 894552, "sha256": "e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99"}
READBACK_SHA = "fbc28f1bc5618459e1178bbea45f3db54fedd21363d7abdcfe4794ef0b0e9adb"
MAX_PHASES, MAX_BATCH, MAX_RECORDS = 949, 64, 8192
MAX_JOURNAL = 128 * 1024 * 1024
ACTION_EVIDENCE = 256 * 1024 * 1024
MAX_EVIDENCE = 8 * 1024 * 1024 * 1024
LOCK = b"piv1-startup-release-operator-lock-v1\n"
KINDS = {"BUFFER_CREATE", "UPLOAD_WRITE", "BOOTSTRAP", "RECIPIENTS", "INITIALIZER", "HEARTBEAT"}
DESCRIBE_SCRIPT = r'''
const fs=require('node:fs'),c=require(process.env.PIV1_PUBLIC_CHECK_REPO+'/clients/cli/startup-public-check.cjs');
c.verifySources();const raw=fs.readFileSync(0);if(raw.length>1048576)throw Error('input_bound');
const out=JSON.parse(raw).map(x=>{const d=c.decodeWire(x.wireBase64,x.lookupTables);
 if(!d.fullyResolved||d.signatures.some(s=>s.some(v=>v!==0)))throw Error('unsigned_resolved_required');
 return {messageBase64:d.message.toString('base64'),messageSha256:d.messageSha256,feePayer:d.feePayer,
 requiredSigners:d.requiredSigners,accountRoles:d.roles,lookupTables:x.lookupTables};});
const text=JSON.stringify(out);if(Buffer.byteLength(text)>524288)throw Error('output_bound');process.stdout.write(text);
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


def describe(j, packets):
    """Same pinned Node/SDK and strict decoder, used only on zero-signature wires."""
    repo = j.pins(); raw = encode(packets)
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


def states_shape(c, j, states):
    need(type(states) is dict and 1 <= len(states) <= 64, "account_subset_bound")
    for key, a in states.items():
        c.b58decode(key, 32)
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
    fields(m, ("schema", "packageSha256", "artifact", "sourcePins", "initializerPreManifestSha256", "parentTail", "historicalLedger", "proposedFeeBudget", "phases"))
    need(m["schema"] == SCHEMA and m["artifact"] == ARTIFACT and m["parentTail"] == parent["tail"], "manifest_identity")
    j.digest(m["packageSha256"]); j.digest(m["initializerPreManifestSha256"])
    need(m["sourcePins"] == {"readback": READBACK_SHA, "checker": j.CHECKER_SHA, "sdk": j.SDK_SHA, "journal": "bfea079a5740fa66e437930d50793a18289beeae1760af4164e986295c8a6d37"}, "manifest_source_pins")
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


def journal_path(base): return Path(os.path.abspath(base)).with_name(Path(base).name + ".release-operator")


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
    state = {"manifest": m, "manifestSha256": sha(encode(m)), "cursor": 0, "prepared": {}, "signed": {}, "intents": {}, "terminal": {}, "seal": None, "preVote": None, "postVote": None,
        "reserved": 0, "actual": 0, "floor": j.u64(first["contextFloor"]), "lastSlot": j.u64(first["lastFinalizedSlot"]), "failed": False, "signedHistory": [], "attempt": None, "transportMode": None}
    need(first["historicalLedger"] == m["historicalLedger"], "historical_ledger_rebind")
    ids = [p["id"] for p in m["phases"]]
    for record in records[1:]:
        kind, d = record["kind"], record["data"]
        unresolved = any(pid not in state["terminal"] for pid in state["intents"])
        need(not state["failed"], "record_after_finalized_failure")
        if kind in ("BIND_SEAL", "PREPARE", "IMPORT", "PRE_VOTE", "POST_VOTE"):
            need(not unresolved, "unresolved_intent_blocks_transition")
        if kind == "BIND_SEAL":
            need(state["seal"] is None and not state["failed"] and state["cursor"] < len(ids) and m["phases"][state["cursor"]]["kind"] == "INITIALIZER", "seal_transition")
            need(d["baseParentTail"] == m["parentTail"], "seal_original_parent_binding")
            need(state["transportMode"] is None or state["transportMode"] == d["transportMode"], "transport_mode_changed")
            state["transportMode"] = d["transportMode"]
            state["seal"] = d
        elif kind == "PREPARE":
            need(type(d) is list and 1 <= len(d) <= MAX_BATCH and len({p["id"] for p in d}) == len(d), "prepare_replay_batch")
            for p in d:
                need(p["id"] in ids and p["id"] not in state["intents"] and ids.index(p["id"]) >= state["cursor"], "prepare_after_intent")
                phase = m["phases"][ids.index(p["id"])]
                need(semantic(p["reviewed"]) == semantic(phase_template(state, phase)), "prepared_semantics_changed")
                need(p["lifetime"]["minimumRemainingBlockHeights"] == phase["minimumRemainingBlockHeights"] and j.u64(p["feeQuoteLamports"]) <= j.u64(phase["feeCapLamports"]), "prepared_limits_changed")
                prior = state["prepared"].get(p["id"])
                if prior and prior["lifetime"]["recentBlockhash"] == p["lifetime"]["recentBlockhash"]:
                    need(prior["lifetime"]["lastValidBlockHeight"] == p["lifetime"]["lastValidBlockHeight"] and j.u64(p["lifetime"]["currentBlockHeight"]) >= j.u64(prior["lifetime"]["currentBlockHeight"]), "same_blockhash_lifetime_changed")
                state["prepared"][p["id"]] = p
                state["signed"].pop(p["id"], None)
        elif kind == "IMPORT":
            need(type(d) is list and 1 <= len(d) <= MAX_BATCH and len({p["id"] for p in d}) == len(d), "import_replay_batch")
            for p in d:
                need(p["id"] in state["prepared"] and p["id"] not in state["signed"], "import_replay")
                need(p["id"] not in state["intents"] and p["preparedSha256"] == sha(encode(state["prepared"][p["id"]])), "import_prepared_binding")
                need(not any(x["wireSha256"] == p["wireSha256"] or x["signatureHex"] == p["signatureHex"] for x in state["signedHistory"]), "signed_history_replay")
                state["signed"][p["id"]] = p
                state["signedHistory"].append(p)
        elif kind == "ATTEMPT":
            need(state["cursor"] < len(ids) and d["id"] == ids[state["cursor"]] and d["stage"] in ("SEND", "RECOVER", "PRE_VOTE", "POST_VOTE"), "attempt_phase")
            need((d["stage"] == "RECOVER") == unresolved, "attempt_intent_relation")
            need(d["transportMode"] in ("FIXED_ORIGIN_TLS", "OFFLINE_INJECTED_TEST") and (state["transportMode"] is None or state["transportMode"] == d["transportMode"]), "transport_mode_changed")
            state["transportMode"] = d["transportMode"]
            state["attempt"] = d
        elif kind == "INTENT":
            phase = d["id"]
            need(not state["failed"] and state["cursor"] < len(ids) and phase == ids[state["cursor"]] and phase in state["signed"] and phase not in state["intents"], "intent_order_or_duplicate")
            need(d["wireSha256"] == state["signed"][phase]["wireSha256"] and d["signatureHex"] == state["signed"][phase]["signatureHex"] and d["signature"] == state["signed"][phase]["signature"] and d["feeLamports"] == state["prepared"][phase]["feeQuoteLamports"], "intent_signed_binding")
            need(d["preparedSha256"] == state["signed"][phase]["preparedSha256"] and state["attempt"] is not None and state["attempt"]["stage"] == "SEND" and d["evidencePath"] == state["attempt"]["evidencePath"] and d["transportMode"] == state["attempt"]["transportMode"], "intent_attempt_binding")
            need(j.u64(d["prestateContextSlot"]) >= state["floor"], "intent_prestate_stale")
            p = m["phases"][state["cursor"]]
            if p["kind"] == "INITIALIZER" and 4 <= p["initializerIndex"] <= 7: need(state["preVote"] is not None, "intent_causal_pre_vote")
            if p["kind"] == "INITIALIZER" and p["initializerIndex"] == 8: need(state["postVote"] is not None, "intent_post_vote")
            state["intents"][phase] = d; state["reserved"] += j.u64(d["feeLamports"])
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


def authorization(j, state, raw, pinned, *, allow_expired=False):
    need(type(raw) is bytes and pinned is not None and sha(raw) == j.digest(pinned), "launcher_authorization_required")
    a = j.parse(raw)
    fields(a, ("schema", "manifestSha256", "baseParentTail", "operatorSourceSha256", "expiresUnixTimestamp", "scope", "feeBudgetNamespace"))
    need(a["schema"] == "piv1-launcher-release-authorization-v1" and a["manifestSha256"] == state["manifestSha256"] and a["baseParentTail"] == state["manifest"]["parentTail"] and a["operatorSourceSha256"] == sha(Path(__file__).read_bytes()) and a["scope"] == "EXACT_BOUNDED_TESTNET_RELEASE", "launcher_authorization_binding")
    need(a["feeBudgetNamespace"] == state["manifest"]["proposedFeeBudget"]["namespace"], "launcher_fee_namespace_binding")
    expires = j.u64(a["expiresUnixTimestamp"])
    need(allow_expired or expires > int(time.time()), "launcher_authorization_expired")
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
        need(1 <= len(calls) <= 5 and self.count < 4, "rpc_budget")
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
    output = root / f"{len(operator.chain):08}-{label.lower()}"
    operator.append("ATTEMPT", {"id": state["manifest"]["phases"][state["cursor"]]["id"], "stage": label, "evidencePath": str(output), "transportMode": "OFFLINE_INJECTED_TEST" if transport is not None else "FIXED_ORIGIN_TLS"})
    # Failure after ATTEMPT keeps its immutable identity; later recovery creates
    # a new attempt. Existing/partial output is never reused or adopted.
    return c.Store(output)


def gate(r, c, j, operator, state, stage, transport):
    seal=state["seal"]; need(seal is not None,"seal_required")
    store=evidence_store(c,j,operator,stage,state,transport); output=store.path; store.close()
    # Collector requires an exclusive new directory, so use a deterministic child.
    report=r.collect({"schema":r.SCHEMA,"expectedTail":seal["parentTail"],"stage":stage,"minimumContextSlot":str(state["floor"])},seal["parentPath"],output/"readback",_transport=bounded_transport(c,transport) if transport is not None else None)
    need(report["status"]=="VERIFIED_COLLECTED_STORAGE_AND_CLOCK_EVIDENCE_ONLY" and report["sealDataSha256"]==seal["sealDataSha256"],"storage_clock_gate_failed:"+str(report.get("reason")))
    need(j.u64(report["snapshotContextSlot"])>=state["floor"],"gate_context_stale")
    return {"id":state["manifest"]["phases"][state["cursor"]]["id"],"contextSlot":report["snapshotContextSlot"],"sealDataSha256":report["sealDataSha256"],"snapshotSha256":report["snapshotSha256"],"evidencePath":str(output/"readback"),"manifestSha256":report["manifestReceipt"]["sha256"],"transportMode":report["transportMode"]}


def operate(request, base, *, authorization_raw=None, authorization_sha256=None, live=False, _transport=None):
    fields(request,("schema","action","expectedTail","data")); need(request["schema"]==SCHEMA,"schema")
    action=request["action"]
    if action is None: return {"status":"INCOMPLETE","authorized":False,"liveReady":False}
    need(action in ("CREATE","INSPECT","PREPARE","HANDOFF","IMPORT","BIND_SEAL","SEND","RECOVER"),"action")
    need(_transport is None or callable(_transport),"transport_callable")
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
                reviewed=describe(j,packets)
                op.append("BIND_SEAL",{"parentPath":str(parent_path),"parentTail":data["parentTail"],"baseParentTail":report["initializerProfile"]["parentTail"],"sealDataSha256":sha(encode(seal)),"postManifestSha256":seal["postManifestSha256"],"nativeSha256":seal["nativeSha256"],"anchorUnixTimestamp":seal["decodedClock"]["unixTimestamp"],"transactionIndex":seal["postManifest"]["initializer"]["governance"]["transactionIndex"],"reviewedPackets":reviewed,"transportMode":"OFFLINE_INJECTED_TEST" if _transport is not None else "FIXED_ORIGIN_TLS"})
            elif action=="PREPARE":
                need(type(data) is list and 1<=len(data)<=MAX_BATCH,"batch_bound")
                need(not any(p["id"] in state["intents"] and p["id"] not in state["terminal"] for p in m["phases"]),"unresolved_intent_blocks_prepare")
                need(type(data[0]) is dict and data[0].get("id") in [p["id"] for p in m["phases"]], "prepare_phase")
                start=next(i for i,p in enumerate(m["phases"]) if p["id"] == data[0]["id"])
                need(cursor <= start and start+len(data)<=len(m["phases"]) and all(p["id"] in state["prepared"] for p in m["phases"][cursor:start]),"batch_phase_bound_or_gap")
                reviewed=describe(j,[{"wireBase64":p["wireBase64"],"lookupTables":phase_template(state,m["phases"][start+i])["lookupTables"]} for i,p in enumerate(data)])
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
                    entries.append({**p,"reviewed":review})
                op.append("PREPARE",entries)
            elif action=="HANDOFF":
                authorization(j,state,authorization_raw,authorization_sha256)
                need(not any(pid not in state["terminal"] for pid in state["intents"]), "unresolved_intent_blocks_handoff")
                need(type(data) is list and 1<=len(data)<=MAX_BATCH and len(set(data))==len(data),"handoff_batch")
                packets=[]
                for pid in data:
                    need(pid in state["prepared"] and pid not in state["intents"],"handoff_phase")
                    packets.append(state["prepared"][pid])
                return {"status":"LAUNCHER_BOUND_SIGNING_HANDOFF_ONLY","tail":op.tail(),"packets":packets,"signerInvoked":False,"founderAuthorizationCryptographicallyVerified":False,"liveReady":False}
            elif action=="IMPORT":
                need(type(data) is list and 1<=len(data)<=MAX_BATCH,"import_batch")
                need(not any(pid not in state["terminal"] for pid in state["intents"]), "unresolved_intent_blocks_import")
                entries=[]
                for p in data:
                    fields(p,("id","wireBase64")); pid=p["id"]
                    need(pid in state["prepared"] and pid not in state["signed"] and pid not in state["intents"],"signed_phase")
                    need(not any(x["id"] == pid for x in entries), "duplicate_import_phase")
                    wire=c.packet(p["wireBase64"]); signed={**p,"preparedSha256":sha(encode(state["prepared"][pid])),"wireSha256":sha(wire["wire"]),"signatureHex":wire["signatureHex"],"signature":wire["signature"]}
                    need(not any(x["wireSha256"]==signed["wireSha256"] or x["signatureHex"]==signed["signatureHex"] for x in state["signedHistory"]+entries),"global_signed_replay")
                    phase=next(p for p in m["phases"] if p["id"]==pid)
                    check_signed(j,frozen(state),phase,state["prepared"][pid],signed); entries.append(signed)
                op.append("IMPORT",entries)
            elif action in ("SEND","RECOVER"):
                authorization_pin=authorization(j,state,authorization_raw,authorization_sha256,allow_expired=action=="RECOVER")
                need(_transport is not None or live,"live_transport_not_enabled")
                need(state["transportMode"] is None or state["transportMode"] == ("OFFLINE_INJECTED_TEST" if _transport is not None else "FIXED_ORIGIN_TLS"), "transport_mode_changed")
                need(cursor<len(m["phases"]),"workflow_complete"); phase=m["phases"][cursor]; pid=phase["id"]
                need(data=={"id":pid} and pid in state["signed"],"current_signed_phase_required")
                prepared=state["prepared"][pid]; signed=state["signed"][pid]
                check_signed(j,frozen(state),phase,prepared,signed)
                if action=="SEND":
                    need(pid not in state["intents"],"intent_exists_recovery_only")
                    if phase["kind"]=="INITIALIZER":
                        idx=phase["initializerIndex"]
                        if idx==4:
                            evidence=gate(r,c,j,op,state,"PRE_VOTE",_transport); op.append("PRE_VOTE",evidence); state=replay(j,op.chain)
                        if 4<=idx<=7: need(state["preVote"] is not None,"causal_pre_vote_required")
                        if idx==8:
                            evidence=gate(r,c,j,op,state,"POST_VOTE",_transport); op.append("POST_VOTE",evidence); state=replay(j,op.chain)
                    store=evidence_store(c,j,op,"SEND",state,_transport); rpc=Rpc(c,j,store,_transport)
                    try:
                        need(rpc.query([("getGenesisHash",[])])[0]==c.GENESIS,"genesis_mismatch")
                        life=prepared["lifetime"]
                        height,valid,quote,accounts,latest=rpc.query([
                            ("getBlockHeight",[{"commitment":"finalized"}]),
                            ("isBlockhashValid",[life["recentBlockhash"],{"commitment":"finalized","minContextSlot":state["floor"]}]),
                            ("getFeeForMessage",[prepared["reviewed"]["messageBase64"],{"commitment":"finalized","minContextSlot":state["floor"]}]),
                            ("getMultipleAccounts",[list(phase["expectedPrestate"]),{"encoding":"base64","commitment":"finalized","minContextSlot":state["floor"]}]),
                            ("getLatestBlockhash",[{"commitment":"finalized","minContextSlot":state["floor"]}])])
                        now=c.u64(height); last=j.u64(life["lastValidBlockHeight"])
                        need(type(latest) is dict and c.u64(latest["context"]["slot"]) >= state["floor"] and latest["value"]["blockhash"] == life["recentBlockhash"] and c.u64(latest["value"]["lastValidBlockHeight"]) == last, "fresh_blockhash_lifetime_binding")
                        need(now >= j.u64(life["currentBlockHeight"]), "blockheight_regression")
                        need(now<=last and last-now>=j.u64(phase["minimumRemainingBlockHeights"]),"fresh_lifetime_margin")
                        need(type(valid) is dict and valid.get("value") is True and c.u64(valid["context"]["slot"])>=state["floor"],"blockhash_invalid_or_stale")
                        need(type(quote) is dict and quote.get("value") is not None and c.u64(quote["context"]["slot"])>=state["floor"] and str(c.u64(quote["value"]))==prepared["feeQuoteLamports"],"fresh_fee_quote_mismatch")
                        need(type(accounts) is dict and c.u64(accounts["context"]["slot"])>=state["floor"] and type(accounts.get("value")) is list and len(accounts["value"])==len(phase["expectedPrestate"]),"prestate_context")
                        actual={key:state_digest(c,c.normalize_account(raw)) for key,raw in zip(phase["expectedPrestate"],accounts["value"])}
                        need(actual==phase["expectedPrestate"],"prestate_mismatch")
                        fee=j.u64(prepared["feeQuoteLamports"]); need(state["reserved"]+fee<=j.u64(m["proposedFeeBudget"]["feeCapLamports"]),"global_fee_cap")
                        authorization(j,state,authorization_raw,authorization_sha256)
                        # No send can occur until this complete intent and its directory are fsynced.
                        op.append("INTENT",{"id":pid,"preparedSha256":signed["preparedSha256"],"wireSha256":signed["wireSha256"],"signature":signed["signature"],"signatureHex":signed["signatureHex"],"feeLamports":prepared["feeQuoteLamports"],"authorizationSha256":authorization_pin,"prestateContextSlot":str(accounts["context"]["slot"]),"evidencePath":str(store.path),"transportMode":"OFFLINE_INJECTED_TEST" if _transport is not None else "FIXED_ORIGIN_TLS"})
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
                    report=c.collect({"schema":c.SCHEMA,"wireBase64":signed["wireBase64"],"expectedWireSha256":signed["wireSha256"],"expectedMessageSha256":prepared["reviewed"]["messageSha256"],"signature":signed["signature"],"expectedFeeLamports":prepared["feeQuoteLamports"],"accountKeys":list(phase["expectedPoststate"]),"minimumContextSlot":str(state["floor"]),"includeExpiry":False},output/"collection",_transport=bounded_transport(c,_transport) if _transport is not None else None)
                    obs=report.get("observation")
                    if obs is not None and obs["status"]["confirmationStatus"]=="finalized":
                        need(j.u64(obs["status"]["slot"])>state["lastSlot"],"global_slot_not_later")
                        if obs["status"]["err"] is None:
                            need(report["status"]=="COLLECTED_PUBLIC_EVIDENCE_ONLY" and obs["poststate"] is not None,"poststate_missing")
                            actual={key:state_digest(c,a) for key,a in obs["poststate"]["accounts"].items()}
                            need(actual==phase["expectedPoststate"],"poststate_mismatch")
                        checked=check_signed(j,frozen(state),phase,prepared,signed,obs)
                        kind="FINALIZED" if obs["status"]["err"] is None else "FAILED"
                        op.append(kind,{"id":pid,"slot":obs["status"]["slot"],"contextSlot":obs["poststate"]["contextSlot"] if obs["poststate"] else obs["status"]["slot"],"feeLamports":obs["transaction"]["feeLamports"],"wireSha256":signed["wireSha256"],"signatureHex":signed["signatureHex"],"checkerReport":checked,"evidencePath":str(output/"collection"),"manifestSha256":report["manifestReceipt"]["sha256"]})
                    else: op.append("UNKNOWN",{"id":pid,"reason":"ORIGINAL_SIGNATURE_UNRESOLVED_NO_REPLACEMENT","evidencePath":str(output/"collection"),"manifestSha256":report["manifestReceipt"]["sha256"]})
        state=replay(j,op.chain)
        unresolved=state["cursor"]<len(state["manifest"]["phases"]) and state["manifest"]["phases"][state["cursor"]]["id"] in state["intents"]
        return {"status":"FINALIZED_FAILURE_STOP" if state["failed"] else "ALL_PHASES_RECONCILED" if state["cursor"]==len(state["manifest"]["phases"]) else "UNKNOWN_OR_SUBMITTED_RECOVERY_ONLY" if unresolved else "DURABLE_RELEASE_PREPARATION_ONLY", "tail":op.tail(),"completedPhases":state["cursor"],"phaseCapacity":MAX_PHASES,"historicalLedger":copy.deepcopy(state["manifest"]["historicalLedger"]),"proposedFeeLedger":{"namespace":state["manifest"]["proposedFeeBudget"]["namespace"],"status":"PROPOSAL_ONLY_NOT_AUTHORIZED","feeCapLamports":state["manifest"]["proposedFeeBudget"]["feeCapLamports"],"reservedFeeLamports":str(state["reserved"]),"actualFeeLamports":str(state["actual"])},"liveReady":False,"founderAuthorizationCryptographicallyVerified":False,"completeStateOracle":False,"signerInvoked":False}
    finally: op.close()


def main(argv):
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("request"); p.add_argument("base_parent"); p.add_argument("--authorization-receipt"); p.add_argument("--authorization-sha256"); p.add_argument("--allow-live-send",action="store_true")
    args=p.parse_args(argv)
    # All current preparations omit authorization and the live flag.
    r,c,s,j=helpers(); raw=Path(args.request).read_bytes(); request=j.parse(raw)
    auth=Path(args.authorization_receipt).read_bytes() if args.authorization_receipt else None
    result=operate(request,args.base_parent,authorization_raw=auth,authorization_sha256=args.authorization_sha256,live=args.allow_live_send)
    print(json.dumps(result,sort_keys=True,indent=2))


if __name__=="__main__":
    try: main(__import__('sys').argv[1:])
    except Exception as exc:
        print(json.dumps({"status":"STOP","reason":str(exc) if isinstance(exc,(Stop,ValueError)) else "operator_failure","liveReady":False}),file=__import__('sys').stderr)
        raise SystemExit(1)
