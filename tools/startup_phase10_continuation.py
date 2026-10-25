#!/usr/bin/env python3
"""Explicit phase11..948 extension of the immutable startup controller library.

The original AU journal remains UNKNOWN. A separately finalized new phase10
journal can seed this distinct namespace; no old terminal is fabricated. Only
its necessary record dispatcher is copied from e356, with explicit seed and
source/authorization adapters. Original operate, IO, signing, Clock, recovery,
SEAL and vote controls run from their genuine unchanged source paths.

No CLI activates this extension. Preparation and execution require an actual
new FINALIZED proof and a fresh exact outer grant; tests use explicit seams.
"""
import base64
import copy
import fcntl
import importlib.util
import hashlib
import os
from pathlib import Path
import re
import time

_here = Path(__file__).parent
_spec = importlib.util.spec_from_file_location("continuation_phase10", _here/"startup_phase10_exception.py")
_phase10_path = _here/"startup_phase10_exception.py"
_phase10_raw = _phase10_path.read_bytes() if not _phase10_path.is_symlink() else b""
if hashlib.sha256(_phase10_raw).hexdigest() != "6ea355a520e05aa3db00531e3456279b124fa80a719040ff59ae4ce3ade8e361":
    raise ValueError("suffix_phase10_source_pin")
e = importlib.util.module_from_spec(_spec)
exec(compile(_phase10_raw,str(_phase10_path),"exec",dont_inherit=True),e.__dict__)
Stop, need, encode, sha = e.Stop, e.need, e.encode, e.sha
SCHEMA = "piv1-au-phase10-continuation-controller-v1"
AUTH_SCHEMA = "piv1-au-phase10-continuation-authorization-v1"
BRIDGE = "startup_phase10_continuation_bridge.cjs"
SOURCE = Path(__file__).name
LOCK = b"piv1-au-phase10-continuation-lock-v1\n"


def source_pins():
    names = {**e.PINS, "collect_startup_public_evidence.py": "96309ac61a07dc23f204cd7c799b25ec67624af4d1103b9057b25728db85b7f6"}
    need(all(sha(e.read_public(_here/name, e.MAX_RECORD)) == digest for name,digest in names.items()), "suffix_immutable_source_pin")
    return {**names, **{name:sha(e.read_public(_here/name,e.MAX_RECORD)) for name in ("startup_phase10_exception.py",e.BRIDGE,SOURCE,BRIDGE)}}


def parent_files(path, manifest):
    """Pin the original physical public parent, never a backup/resume input."""
    core = e.successor_core(); j = core.helpers()[3]
    fd = j.directory(path); directory_meta = os.fstat(fd); lock = None
    try:
        lock = os.open("lock",os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK,dir_fd=fd)
        meta = os.fstat(lock); j.secure_meta(meta); fcntl.flock(lock,fcntl.LOCK_EX)
        identity = {"device":str(meta.st_dev),"inode":str(meta.st_ino),"ctimeNs":str(meta.st_ctime_ns)}
        def stable():
            current_directory = os.stat(path,follow_symlinks=False)
            need((current_directory.st_dev,current_directory.st_ino) == (directory_meta.st_dev,directory_meta.st_ino),"suffix_original_parent_directory")
            current = os.stat("lock",dir_fd=fd,follow_symlinks=False)
            need((current.st_dev,current.st_ino,current.st_ctime_ns) == (meta.st_dev,meta.st_ino,meta.st_ctime_ns) and
                 e.read_public("lock",128,dir_fd=fd) == j.LOCK_BYTES,"suffix_original_parent_lock")
        stable()
        names = set(os.listdir(fd))
        need(names == {"lock"}|{f"{i:08}.json" for i in range(12)}, "suffix_exact_original_parent_namespace")
        chain,_ = j.records(fd)
        need(chain[0]["lockIdentity"] == identity,"suffix_original_parent_lock_identity")
        need(j.tail(chain) == manifest["parentTail"], "suffix_original_parent_tail")
        segments = j.reconstruct(chain)
        need(segments[0]["input"]["cumulativeFeeCapLamports"] == manifest["historicalLedger"]["cumulativeFeeCapLamports"], "suffix_original_parent_cap")
        pins = {name:sha(e.read_public(name,e.MAX_RECORD,dir_fd=fd)) for name in sorted(names)}
        need(set(os.listdir(fd)) == names, "suffix_original_parent_changed")
        stable()
        return pins
    finally:
        if lock is not None:fcntl.flock(lock,fcntl.LOCK_UN);os.close(lock)
        os.close(fd)


def finalized_phase10(value, journal, tail):
    """Read-lock and replay the exact real new proof; no old operator invocation."""
    p = e.checked_package(value); core = e.successor_core(); _,c,_,j = core.helpers()
    core.replay = lambda api, records:e.successor_replay(core,c,api,records,journal)
    op = core.Journal(j,journal)
    try:
        need(op.tail() == tail, "suffix_phase10_tail_changed")
        state = core.replay(j,op.chain)
        need(state["stage"] == "FINALIZED" and op.chain[-1]["kind"] == "FINALIZED", "suffix_phase10_finalized_required")
        terminal = op.chain[-1]
        directory = Path(journal).with_name(Path(journal).name+".evidence")/f"{int(terminal['sequence'])-1:08}-recovery"/"collection"
        report = e.retained_recovery(core,c,j,state,directory,terminal["data"]["manifestSha256"])
        obs = report["observation"]
        accounts = obs["poststate"]["accounts"]; context = int(obs["poststate"]["contextSlot"])
        post = e.verify_snapshot(core,c,p,accounts,context,state["floor"],debit=int(state["intent"]["guardResult"]["physicalOldDebitLamports"]),post=True)
        need(obs["status"]["err"] is None and obs["transaction"]["feeLamports"] == "5000", "suffix_phase10_success_fee")
        op.stable()
        need(set(os.listdir(op.fd)) == {"lock"}|{f"{i:08}.json" for i in range(len(op.chain))} and
             all(j.read_file(op.fd,f"{i:08}.json",e.MAX_RECORD) == encode(r) for i,r in enumerate(op.chain)), "suffix_phase10_chain_changed")
        return state,obs,post
    finally: op.close()


def package(value, phase10_journal, phase10_tail):
    """Assemble only after the exact new terminal exists, with no live authority."""
    state,obs,post = finalized_phase10(value,phase10_journal,phase10_tail)
    old = e.records_at(value["evidence"][0]); first = old[0]["data"]
    parent = first["baseParentPath"]
    need(os.path.isabs(parent) and os.path.normpath(parent) == parent and not Path(parent).is_symlink(), "suffix_original_parent_path")
    return {"schema":"piv1-au-phase10-continuation-package-v1", "phase10Package":value,
            "phase10JournalPath":str(Path(phase10_journal).absolute()), "phase10Tail":phase10_tail,
            "baseParentPath":parent, "parentFiles":parent_files(parent,first["manifest"]), "sourcePins":source_pins(),
            "originalManifestSha256":sha(encode(first["manifest"])),
            "transportMode":state["attempt"]["mode"], "firstPhase":"startup_0011", "lastPhase":"startup_0948"}


def checked(value):
    need(type(value) is dict and set(value) == {"schema","phase10Package","phase10JournalPath","phase10Tail","baseParentPath","parentFiles","sourcePins","originalManifestSha256","transportMode","firstPhase","lastPhase"}, "suffix_package_fields")
    need(package(value["phase10Package"],value["phase10JournalPath"],value["phase10Tail"]) == value, "suffix_package_changed")
    old = e.records_at(value["phase10Package"]["evidence"][0])
    m = copy.deepcopy(old[0]["data"]["manifest"])
    m["packageSha256"] = sha(encode(value))
    m["proposedFeeBudget"]["namespace"] = "piv1-testnet-startup-release-"+m["packageSha256"]
    return m,old


def grant(value, raw, pin, journal, *, expired=False):
    need(type(raw) is bytes and sha(raw) == pin, "suffix_authorization_hash")
    a = e.canonical(raw)
    need(set(a) == {"schema","scope","packageSha256","journalPath","baseParentPath","sourcePins","phase10Tail","originalTail","expiresUnixTimestamp","transportMode","backendAuthorizationRawBase64","backendAuthorizationSha256","manifestSha256","carriedPossibleFeeLamports","combinedFeeCapLamports"}, "suffix_authorization_fields")
    path = Path(journal)
    need(str(path) == os.path.abspath(journal) and not path.is_symlink() and path.name == "au-phase10-"+e.TAIL+".continuation" and _here.resolve().parent not in path.parents, "suffix_exact_namespace")
    need(a["schema"] == AUTH_SCHEMA and a["scope"] == "EXACT_PHASE11_948_AFTER_NEW_PHASE10_FINALIZED" and
         a["packageSha256"] == sha(encode(value)) and a["journalPath"] == str(path) and a["baseParentPath"] == value["baseParentPath"] and
         a["sourcePins"] == value["sourcePins"] == source_pins() and a["phase10Tail"] == value["phase10Tail"] and a["originalTail"] == e.TAIL and
         a["transportMode"] == value["transportMode"] and a["carriedPossibleFeeLamports"] == "5000" and a["combinedFeeCapLamports"] == "4820000", "suffix_authorization_binding")
    expiry = a["expiresUnixTimestamp"]
    need(type(expiry) is str and re.fullmatch(r"0|[1-9][0-9]{0,19}",expiry) is not None and int(expiry)<2**64 and (expired or int(expiry)>int(time.time())), "suffix_authorization_expired")
    inner_raw = base64.b64decode(a["backendAuthorizationRawBase64"],validate=True)
    need(base64.b64encode(inner_raw).decode() == a["backendAuthorizationRawBase64"] and sha(inner_raw) == a["backendAuthorizationSha256"] != e.AUTH_SHA, "suffix_fresh_nested_authorization")
    inner = e.canonical(inner_raw)
    need(set(inner) == {"schema","manifestSha256","baseParentTail","operatorSourceSha256","expiresUnixTimestamp","scope","feeBudgetNamespace","signerBackendSha256","signerProfileSha256"} and
         inner["scope"] == "EXACT_BOUNDED_TESTNET_RELEASE" and
         inner["baseParentTail"] == value["phase10Package"]["proposal"]["lineage"]["originalParentTail"] and
         type(inner["signerProfileSha256"]) is str and re.fullmatch(r"[0-9a-f]{64}",inner["signerProfileSha256"]) is not None, "suffix_nested_fields")
    need(inner["schema"] == "piv1-launcher-release-authorization-v2" and inner["operatorSourceSha256"] == e.PINS["startup_release_operator.py"] and
         inner["manifestSha256"] == a["manifestSha256"] and inner["expiresUnixTimestamp"] == expiry and
         inner["feeBudgetNamespace"] == "piv1-testnet-startup-release-"+a["packageSha256"] and
         inner["signerBackendSha256"] == e.PINS["startup_signer_backend.cjs"], "suffix_nested_compatibility_binding")
    return a,inner_raw


def seed(core,c,j,value,m,old):
    state,obs,post = finalized_phase10(value["phase10Package"],value["phase10JournalPath"],value["phase10Tail"])
    core.startup_manifest().validate(core.operator_api(),j,m)
    need(sha(encode(old[0]["data"]["manifest"])) == e.MANIFEST_SHA == value["originalManifestSha256"] and
         old[0]["data"]["baseParentPath"] == value["baseParentPath"] and parent_files(value["baseParentPath"],old[0]["data"]["manifest"]) == value["parentFiles"], "suffix_original_lineage_changed")
    inventory = copy.deepcopy(value["phase10Package"]["proposal"]["lineage"]["originalSignedPacketInventory"])
    signed = state["intent"]["signed"]
    need(all(x["wireSha256"] != signed["wireSha256"] and x["signatureHex"] != signed["signatureHex"] for x in inventory), "suffix_new_packet_replayed")
    inventory.append(copy.deepcopy(signed))
    slot = int(obs["status"]["slot"])
    return {"manifest":m,"manifestSha256":sha(encode(m)),"cursor":11,"prepared":{},"signed":{},"intents":{},"terminal":{},"seal":None,"preVote":None,"postVote":None,
            "reserved":60000,"actual":60000,"floor":max(slot,int(post["contextSlot"]),state["floor"]),"lastSlot":slot,"failed":False,"signedHistory":inventory,
            "attempt":None,"transportMode":value["transportMode"],"runtimeFacts":{},"runtimeClock":post["clock"],"baseParentPath":value["baseParentPath"],"preparedAt":{},
            "handoff":None,"signerAttempt":None,"signerAttempts":{},"quoteByPhase":{},"voteGateAt":{},"voteGateClock":{},
            "carriedOriginalUnknown":{"tail":e.TAIL,"actualFeeLamports":"55000","reservedFeeLamports":"60000","possibleAdditionalFeeLamports":"5000","reservationReleased":False},
            "g0AdjustmentLamports":e.downstream_balance_adjustment(int(post["physicalOldDebitLamports"]),5000)}


# Static copy of the necessary e356 replay dispatcher only. All IO, validation,
# controller actions and recovery primitives below remain immutable library calls.
def replay_suffix(core, j, records, seed):
    need(records and records[0]["kind"] == "CREATE", "create_required")
    source_sha = core._replay_source_hashers()
    # Pinned rule functions do not mutate their module globals. Share only
    # their code/globals within this side-effect-free replay, never its results
    # or module across a commit, callback or another replay invocation.
    fresh_rules = core.state_rules; rules_module = None
    send_payloads = {}  # Derived only after fresh INTENT coverage equality below.
    compound_requests = set()  # Replay-local origin of the new compound attempt.
    def replay_rules():
        nonlocal rules_module
        if rules_module is None: rules_module = fresh_rules()
        return rules_module
    api = core.operator_api(); api.state_rules = replay_rules
    first = records[0]["data"]; m = first["manifest"]
    full = core.full_startup(m)
    # Full-mode IMPORT rederives canonical string hashes before membership.
    # These indexes belong only to this replay; signedHistory stays unchanged.
    signed_wire_hashes = set() if full else None
    signed_signature_hexes = set() if full else None
    adapter = core.startup_manifest(_replay_source_sha=source_sha) if full else None
    c = core.helpers()[1] if full or any(r["kind"] in ("HANDOFF","SIGN_ATTEMPT") for r in records) else None
    if full: adapter.validate(api,j,m)
    state = seed()
    need(state["manifest"] == m, "suffix_seed_manifest")
    signed_wire_hashes = {x["wireSha256"] for x in state["signedHistory"]}
    signed_signature_hexes = {x["signatureHex"] for x in state["signedHistory"]}
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
            core.fields(d,("import","attempt","response","intent"))
            core.replay_compound_import(c,j,state,ids,record,{k:d[k] for k in ("import","attempt","response")},full,unresolved,compound_requests,signed_wire_hashes,signed_signature_hexes)
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
            core.replay_prepare(c,j,state,m,ids,record,d,full,_replay_source_sha=source_sha,_phase_positions=phase_positions)
        elif kind == "PREPARE_SIGN_ATTEMPT":
            core.fields(d,("entries","quote","attempt"))
            need(full and state["cursor"] < len(ids) and state["handoff"] is None,"compound_current_phase_required")
            entries=d["entries"]
            need(type(entries) is list and len(entries)==1 and type(entries[0]) is dict and entries[0].get("id")==ids[state["cursor"]],"compound_current_phase_required")
            pid=ids[state["cursor"]]
            need(pid not in state["prepared"] and pid not in state["signed"] and pid not in state["intents"],"compound_phase_already_prepared")
            commitment=core.compound_prepared_digest(record["previousDigest"],entries,d["quote"])
            core.replay_prepare(c,j,state,m,ids,record,{"entries":entries,"quote":d["quote"]},full,prepared_digest=commitment,_replay_source_sha=source_sha,_phase_positions=phase_positions)
            need(type(d["attempt"]) is dict and type(d["attempt"].get("request")) is dict and type(d["attempt"]["request"].get("packets")) is list and len(d["attempt"]["request"]["packets"])==1,"compound_single_signing_request")
            need(type(d["attempt"]["request"]["packets"][0]) is dict and d["attempt"]["request"]["packets"][0].get("id")==pid,"compound_current_signing_request")
            core.replay_sign_attempt(c,j,state,record,d["attempt"],full,unresolved,_replay_source_sha=source_sha,_phase_positions=phase_positions)
            compound_requests.add(d["attempt"]["request"]["requestId"])
        elif kind == "IMPORT_SEND_ATTEMPT":
            core.replay_compound_import(c,j,state,ids,record,d,full,unresolved,compound_requests,signed_wire_hashes,signed_signature_hexes)
        elif kind == "HANDOFF":
            core.fields(d,("schema","cluster","genesisHash","operatorTail","operatorSourceSha256","authorizationSha256","baseParentPath","baseParentTail","manifestSha256","sealSha256","packets","requestId"))
            j.digest(d["authorizationSha256"])
            need(type(d["packets"]) is list and all(type(p) is dict and "id" in p for p in d["packets"]), "handoff_batch")
            need(d == core.signing_request(c,state,record["previousDigest"],[p["id"] for p in d["packets"]],d["authorizationSha256"],_replay_source_sha=source_sha,_phase_positions=phase_positions), "handoff_replay_binding")
            core.captured_authorization(state,[p["id"] for p in d["packets"]],d["authorizationSha256"])
            state["handoff"] = d
        elif kind == "SIGN_ATTEMPT":
            core.replay_sign_attempt(c,j,state,record,d,full,unresolved,_replay_source_sha=source_sha,_phase_positions=phase_positions)
        elif kind == "IMPORT":
            core.replay_import(c,j,state,d,full,signed_wire_hashes,signed_signature_hexes)
        elif kind == "ATTEMPT":
            core.replay_attempt(state,ids,d,full,unresolved)
        elif kind == "INTENT":
            phase = d["id"]
            need(not state["failed"] and state["cursor"] < len(ids) and phase == ids[state["cursor"]] and phase in state["signed"] and phase not in state["intents"], "intent_order_or_duplicate")
            need(d["wireSha256"] == state["signed"][phase]["wireSha256"] and d["signatureHex"] == state["signed"][phase]["signatureHex"] and d["signature"] == state["signed"][phase]["signature"] and d["feeLamports"] == state["prepared"][phase]["feeQuoteLamports"], "intent_signed_binding")
            need(d["preparedSha256"] == state["signed"][phase]["preparedSha256"] and state["attempt"] is not None and state["attempt"]["stage"] == "SEND" and d["evidencePath"] == state["attempt"]["evidencePath"] and d["transportMode"] == state["attempt"]["transportMode"], "intent_attempt_binding")
            if "requestId" in state["signed"][phase]: need(d["authorizationSha256"] == state["signed"][phase]["authorizationSha256"], "signing_authorization_changed")
            core.captured_authorization(state,[phase],d["authorizationSha256"])
            if full: need("requestId" in state["signed"][phase], "signing_response_required")
            need(j.u64(d["prestateContextSlot"]) >= state["floor"], "intent_prestate_stale")
            p = m["phases"][state["cursor"]]
            if full:
                adapter = core.startup_manifest(_replay_source_sha=source_sha)
                if p["kind"] != "UPLOAD_WRITE": p = adapter.effective(api,c,state,p)
            if full:
                need(d["runtimePreFacts"] == state["runtimeFacts"], "startup_intent_facts_binding")
                replay_rules().saved_clock(d["runtimePreClock"])
                need(j.u64(d["runtimePreClock"]["slot"]) >= state["floor"] and d["runtimePreClock"]["contextSlot"] == d["prestateContextSlot"], "startup_intent_clock_binding")
                if state["runtimeClock"] is not None: need(int(d["runtimePreClock"]["unixTimestamp"]) >= int(state["runtimeClock"]["unixTimestamp"]), "startup_preclock_regression")
                raw_accounts,preflight=core.verify_preflight_accounts(c,j,state,p,d,d["coverage"].get("preflightReceipt")) if p["kind"]!="UPLOAD_WRITE" else (None,None)
                coverage=adapter.coverage(api,c,state,p,"PRE",state["prepared"][phase]["reviewed"],d["runtimePreFacts"],d["runtimePreClock"],raw_accounts,preflight_receipt=preflight)
                need(d["coverage"] == coverage, "startup_intent_coverage_binding")
                send_payloads[phase]=core.account_payload(coverage["accountDigests"])
                # Later post-INTENT/send failure files must never poison the
                # durable original intent or its read-only recovery path.
                core.successful_evidence(j,[Path(d["evidencePath"])],send_payloads[phase],preflight_queries=(10+len(adapter.rent_sizes(p)))//5,compound_response=compound_intent)
            if core.typed_phase(p):
                need(d["runtimePreFacts"] == state["runtimeFacts"], "startup_intent_facts_binding")
                if core.phase_clock(p):
                    replay_rules().saved_clock(d["runtimePreClock"])
                    need(j.u64(d["runtimePreClock"]["slot"]) >= state["floor"] and d["runtimePreClock"]["contextSlot"] == d["prestateContextSlot"], "startup_intent_clock_binding")
                    if state["runtimeClock"] is not None: need(int(d["runtimePreClock"]["unixTimestamp"]) >= int(state["runtimeClock"]["unixTimestamp"]), "startup_preclock_regression")
            if p["kind"] == "INITIALIZER" and 4 <= p["initializerIndex"] <= 7: need(state["preVote"] is not None, "intent_causal_pre_vote")
            if p["kind"] == "INITIALIZER" and p["initializerIndex"] == 8: need(state["postVote"] is not None, "intent_post_vote")
            if full: core.verify_gate_clock(state,p,d["runtimePreClock"])
            state["intents"][phase] = d; state["reserved"] += j.u64(d["feeLamports"])
            state["handoff"] = None
            state["floor"] = max(state["floor"], j.u64(d["prestateContextSlot"]))
        elif kind in ("UNKNOWN", "SUBMITTED"):
            need(d["id"] in state["intents"] and d["id"] not in state["terminal"], "unknown_without_intent")
            if kind == "SUBMITTED":
                need(d["signature"] == state["intents"][d["id"]]["signature"], "submitted_signature_changed")
                if full:core.successful_evidence(j,[Path(state["intents"][d["id"]]["evidencePath"])],send_payloads[d["id"]])
        elif kind in ("FINALIZED", "FAILED"):
            phase = d["id"]; need(phase in state["intents"] and phase not in state["terminal"] and phase == ids[state["cursor"]], "terminal_order")
            slot = j.u64(d["slot"]); need(slot > state["lastSlot"], "global_slot_not_later")
            need(d["feeLamports"] == state["intents"][phase]["feeLamports"], "terminal_fee_changed")
            need(d["wireSha256"] == state["intents"][phase]["wireSha256"] and d["signatureHex"] == state["intents"][phase]["signatureHex"], "terminal_packet_changed")
            need(state["attempt"] is not None and state["attempt"]["stage"] == "RECOVER" and d["evidencePath"] == str(Path(state["attempt"]["evidencePath"]) / "collection"), "terminal_recovery_binding")
            need(j.u64(d["contextSlot"]) >= max(state["floor"], slot) if kind == "FINALIZED" else j.u64(d["contextSlot"]) == slot, "terminal_context_stale")
            p = m["phases"][state["cursor"]]
            if full:
                adapter = core.startup_manifest(_replay_source_sha=source_sha)
                if p["kind"] != "UPLOAD_WRITE": p = adapter.effective(api,c,state,p)
            if kind == "FINALIZED" and full:
                post_clock=d["runtimePostClock"]; replay_rules().saved_clock(post_clock)
                need(post_clock["contextSlot"] == d["contextSlot"], "startup_terminal_clock_binding")
                replay_rules().window(state["intents"][phase]["runtimePreClock"],post_clock,slot)
                snapshot_accounts=core.verify_full_snapshot(c,j,state,p,d) if adapter.large_key() in adapter.required_keys(p) else None
                need(d.get("fullSnapshot") is None if snapshot_accounts is None else True, "startup_unexpected_full_snapshot")
                coverage=adapter.coverage(api,c,state,p,"POST",state["prepared"][phase]["reviewed"],d["runtimeFacts"],post_clock,snapshot_accounts,full_snapshot=d.get("fullSnapshot"))
                need(d["coverage"] == coverage, "startup_terminal_coverage_binding")
                paths=[Path(d["evidencePath"])];payload=3*core.account_payload(coverage["accountDigests"],adapter.proof_keys(p))
                if snapshot_accounts is not None:
                    paths.append(paths[0].parent/"full-state");payload+=2*core.account_payload(coverage["accountDigests"])
                core.successful_evidence(j,paths,payload)
                state["runtimeClock"] = post_clock
            if kind == "FINALIZED" and core.typed_phase(p):
                rules = replay_rules(); post_clock = d["runtimePostClock"]
                if core.phase_clock(p):
                    rules.saved_clock(post_clock)
                    need(post_clock["contextSlot"] == d["contextSlot"], "startup_terminal_clock_binding")
                rules.transition(state,p,d["runtimeFacts"],slot,state["intents"][phase]["runtimePreClock"],post_clock)
                state["runtimeFacts"] = copy.deepcopy(d["runtimeFacts"])
                if post_clock is not None: state["runtimeClock"] = post_clock
            if kind == "FINALIZED" and full and not core.typed_phase(p):
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
            if full:state["voteGateClock"][kind]=core.verify_gate(j,state,kind,d)
            state["preVote" if kind == "PRE_VOTE" else "postVote"] = d
            state["voteGateAt"][kind]=record["digest"]
            state["floor"] = max(state["floor"], j.u64(d["contextSlot"]))
        else: raise Stop("suffix_record_kind")
        need(state["reserved"] <= j.u64(m["proposedFeeBudget"]["feeCapLamports"]) and state["actual"] <= j.u64(m["proposedFeeBudget"]["feeCapLamports"]), "global_fee_cap")
    if rules_module is not None:
        # Independently reload trusted globals and verify the rules file plus
        # all primary sources again before returning any successfully replayed
        # state. Never trust the reused module's mutable provenance dictionary.
        fresh_rules()
    return state


def controller(request, value, journal, *, authorization_raw, authorization_sha256, live=False, live_sign=False, _transport=None, _signer=None):
    """Run the actual original controller against this separately seeded journal."""
    need(type(request) is dict and set(request) == {"schema","action","expectedTail","data"} and request["schema"] == SCHEMA, "suffix_request")
    action = request["action"]
    need(action in ("CREATE","INSPECT","PREPARE","HANDOFF","SIGN","IMPORT","BIND_SEAL","SEND","RECOVER","CAPTURE_SIGN_SEND"), "suffix_action")
    need((_transport is None) == (_signer is None), "suffix_test_mode_pair")
    a,inner_raw = grant(value,authorization_raw,authorization_sha256,journal,expired=action in ("INSPECT","RECOVER"))
    need(action == "INSPECT" or value["transportMode"] == ("OFFLINE_INJECTED_TEST" if _transport is not None else "FIXED_ORIGIN_TLS"), "suffix_transport_mode_changed")
    m,old = checked(value)
    need(sha(encode(m)) == a["manifestSha256"], "suffix_manifest_grant")
    core = e.successor_core(); _,c,_,j = core.helpers()
    core.SCHEMA = SCHEMA; core.LOCK = LOCK; core.MAX_RECORDS = 8192
    core.journal_path = lambda base:Path(journal)
    # The canonical manifest module retains its genuine source identity. Its
    # source-native expected adapter applies exactly the independently derived
    # physical debit; every caller, including coverage/effective, shares it.
    path = _here/"startup_manifest.py"; raw = e.read_public(path,e.MAX_RECORD)
    need(sha(raw) == e.PINS[path.name], "suffix_manifest_source")
    spec = importlib.util.spec_from_file_location("suffix_canonical_manifest",path)
    adapter = importlib.util.module_from_spec(spec); exec(compile(raw,str(path),"exec",dont_inherit=True),adapter.__dict__)
    original_expected = adapter.expected
    def expected(api,collector,state,phase,stage):
        need(adapter.phase_index(phase) >= 11 and state["g0AdjustmentLamports"] in (0,5000), "suffix_phase_range")
        result = original_expected(api,collector,state,phase,stage)
        key = adapter.oracle()["roles"]["g0"]
        balance = int(result[key]["lamports"])-state["g0AdjustmentLamports"]
        need(0 <= balance < 2**64, "suffix_checked_g0_balance")
        result[key] = {**result[key],"lamports":str(balance)}
        return result
    adapter.expected = expected
    def manifest(*, _replay_source_sha=None):
        need(sha(e.read_public(path,e.MAX_RECORD)) == e.PINS[path.name], "suffix_manifest_source")
        return adapter
    core.startup_manifest = manifest
    def barrier(*,expired=False):
        return grant(value,authorization_raw,authorization_sha256,journal,expired=expired)
    def replay(api,records):
        # Grant expiry is enforced by original action boundaries; expired proof
        # may still be inspected/recovered, never used to sign or send.
        barrier(expired=True)
        need(records and set(records[0]["data"]) == {"manifest","baseParentPath","historicalLedger","contextFloor","lastFinalizedSlot","continuationPackage","outerAuthorization","outerAuthorizationSha256"}, "suffix_create_fields")
        first = records[0]["data"]
        need(first["continuationPackage"] == value and first["outerAuthorization"] == a and first["outerAuthorizationSha256"] == authorization_sha256 and
             first["baseParentPath"] == value["baseParentPath"] and first["historicalLedger"] == m["historicalLedger"], "suffix_create_binding")
        def initial():
            state = seed(core,c,api,value,m,old)
            need(first["contextFloor"] == str(state["floor"]) and first["lastFinalizedSlot"] == str(state["lastSlot"]), "suffix_initial_clock_binding")
            return state
        result = replay_suffix(core,api,records,initial)
        need(result["reserved"]+5000 <= int(a["combinedFeeCapLamports"]) and result["actual"] <= result["reserved"], "suffix_combined_fee_cap")
        barrier(expired=True)
        return result
    core.replay = replay
    original_authorization = core.authorization
    def authorization(api,state,raw,pin,**kw):
        barrier(expired=kw.get("allow_expired",False))
        need(raw == inner_raw and pin == a["backendAuthorizationSha256"], "suffix_nested_receipt_changed")
        return original_authorization(api,state,raw,pin,**kw)
    core.authorization = authorization
    # Child callbacks independently revalidate the exact outer source/grant.
    core.signer_backend_source = lambda **kw:_here/BRIDGE
    original_invoke = core.invoke_signer
    def payload(value):
        return {"outerAuthorizationRawBase64":base64.b64encode(authorization_raw).decode(),"outerAuthorizationSha256":authorization_sha256,"input":value}
    def invoke(api,value):
        barrier(); return original_invoke(api,payload(value))
    core.invoke_signer = invoke
    signer = None
    if _signer is not None:
        def signer(value):
            barrier(); return _signer(payload(value))
    base = value["baseParentPath"]
    if action == "CREATE":
        need(request["data"] is None, "suffix_create_data")
        op = core.Journal(j,base,create=True)
        try:
            need(request["expectedTail"] == op.tail(), "suffix_tail_compare_and_swap")
            initial = seed(core,c,j,value,m,old)
            op.append("CREATE",{"manifest":m,"baseParentPath":base,"historicalLedger":copy.deepcopy(m["historicalLedger"]),
                                "contextFloor":str(initial["floor"]),"lastFinalizedSlot":str(initial["lastSlot"]),
                                "continuationPackage":value,"outerAuthorization":a,"outerAuthorizationSha256":authorization_sha256})
            result = {"status":"DURABLE_RELEASE_PREPARATION_ONLY","tail":op.tail(),"completedPhases":11}
        finally:op.close()
    else:
        result = core.operate(request,base,authorization_raw=inner_raw,authorization_sha256=a["backendAuthorizationSha256"],live=live,live_sign=live_sign,_transport=_transport,_signer=signer)
    # Original library reports its canonical ledger; carried uncertainty remains
    # a distinct liability and cannot be cleared by any suffix terminal.
    if "proposedFeeLedger" in result:
        result["combinedReservedFeeLamports"] = str(int(result["proposedFeeLedger"]["reservedFeeLamports"])+5000)
    elif action == "CREATE": result["combinedReservedFeeLamports"] = "65000"
    else:
        # HANDOFF is the immutable controller's early-return action. Derive its
        # current ledger rather than reuse the initial60000 after later phases.
        op = core.Journal(j,base)
        try:
            need(op.tail() == result["tail"], "suffix_handoff_report_tail_changed")
            state = core.replay(j,op.chain)
            result["combinedReservedFeeLamports"] = str(state["reserved"]+5000)
        finally:op.close()
    result.update(carriedOriginalUnknown={"tail":e.TAIL,"possibleAdditionalFeeLamports":"5000","reservationReleased":False},
                  continuationSourceSha256=value["sourcePins"][SOURCE],liveReady=False)
    return result
