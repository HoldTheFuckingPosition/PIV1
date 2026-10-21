#!/usr/bin/env python3
"""Fixed Testnet readbacks composed with sealed storage and Clock evidence checks."""
import base64
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import struct
import sys

SCHEMA = "piv1-startup-initializer-readback-v1"
TEMPLATE = {"schema": SCHEMA, "expectedTail": None, "stage": None, "minimumContextSlot": None}
PINS = {"tools/collect_startup_public_evidence.py": "96309ac61a07dc23f204cd7c799b25ec67624af4d1103b9057b25728db85b7f6",
    "tools/startup_initializer_storage.py": "0812e50515b96887e1bf175601c736cf04cdb63609cae8f12afedbf599d4d098",
    "tools/startup_public_journal.py": "bfea079a5740fa66e437930d50793a18289beeae1760af4164e986295c8a6d37"}
PROGRAM = "GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux"
MULTISIG = "9mwto7RNokYc8n8MPtzHpubwwnKcborY2r9RnfZAScVS"
CLOCK = "SysvarC1ock11111111111111111111111111111111"
SYSVAR = "Sysvar1111111111111111111111111111111111111"
U64 = (1 << 64) - 1
I64_MIN, I64_MAX = -(1 << 63), (1 << 63) - 1
MAX_INPUT = 65536
EVIDENCE_BUDGET = 40 * 1024 * 1024
RESERVE = 2 * 1024 * 1024 * 1024
FLAGS = {"liveReady": False, "authorized": False, "executionEligible": False, "timelockReleased": False,
    "chainStateCryptographicallyAuthenticated": False, "snapshotFreshnessAuthenticated": False,
    "preVoteCausalityProved": False, "voteSignatureProvenanceProved": False, "controlledSuccessor": False,
    "journalMutationPerformed": False, "newFeesReserved": False, "signingOrSendPerformed": False,
    "replacementOrAdvancementAuthorized": False, "completeStateOracle": False}


class Stop(ValueError): pass


def need(condition, code):
    if not condition: raise Stop(code)


def sha(raw): return hashlib.sha256(raw).hexdigest()


def encode(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False) + "\n").encode()


def utc(): return datetime.now(timezone.utc).isoformat()


def uint(value):
    need(type(value) is str and re.fullmatch(r"0|[1-9][0-9]{0,19}", value) is not None and int(value) <= U64, "u64_string")
    return int(value)


def request_shape(value):
    need(type(value) is dict and set(value) == set(TEMPLATE) and value["schema"] == SCHEMA, "request_fields_or_schema")
    if value["stage"] is None:
        need(all(value[k] is None for k in ("expectedTail", "minimumContextSlot")), "nullable_fields")
        return False
    need(value["stage"] in ("PRE_VOTE", "POST_VOTE"), "stage")
    tail = value["expectedTail"]
    need(type(tail) is dict and set(tail) == {"sequence", "digest"}, "expected_tail")
    uint(tail["sequence"]); uint(value["minimumContextSlot"])
    need(type(tail["digest"]) is str and re.fullmatch("[0-9a-f]{64}", tail["digest"]) is not None, "tail_digest")
    return True


def helpers():
    repo = Path(os.environ.get("PIV1_PUBLIC_CHECK_REPO", Path(__file__).resolve().parent.parent)).resolve()
    for name, expected in PINS.items():
        path = repo / name
        need(not path.is_symlink() and path.is_file() and sha(path.read_bytes()) == expected, "helper_source_pin")
    def load(name, relative):
        spec = importlib.util.spec_from_file_location(name, repo / relative)
        module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module); return module
    c = load("pinned_readback_collector", "tools/collect_startup_public_evidence.py")
    s = load("pinned_readback_storage", "tools/startup_initializer_storage.py")
    j, actual = s.journal_module()
    need(actual == repo, "source_repo")
    return c, s, j


def inspect_parent(value, path, j):
    parent = j.apply({"schema": j.SCHEMA, "action": "INSPECT", "expectedTail": value["expectedTail"], "segment": None, "event": None}, path)
    need(parent["journalMutationPolicy"] == "TERMINALLY_SEALED_NO_CHILD_CONTROLLER" and parent["initializerSeal"] is not None, "terminal_seal_required")
    return parent


def minimum(value, seal):
    return max(uint(value["minimumContextSlot"]), uint(seal["contextFloor"]), uint(seal["clock"]["contextSlot"]))


def tls_profile(seal):
    plan = seal["unsignedInitializerPlan"]
    return plan["program"] == PROGRAM and plan["multisig"] == MULTISIG


def validate(value, path):
    if not request_shape(value): return {"status": "INCOMPLETE", **FLAGS}
    c, s, j = helpers()
    parent = inspect_parent(value, path, j); seal = parent["initializerSeal"]
    return {"status": "VALID_OFFLINE_READBACK_REQUEST_ONLY", "parentTail": parent["tail"], "sealDataSha256": sha(j.encode(seal)),
        "minimumContextSlot": str(minimum(value, seal)), "tlsCollectionProfileAllowed": tls_profile(seal),
        "checkerProcessInvocations": 1, "initializerPlannerProcessInvocations": 1, "nodeProcessInvocations": 2, **FLAGS}


def output_path(output, parent):
    path = Path(os.path.abspath(output)); journal = Path(os.path.abspath(parent))
    for target, root in ((path, journal), (path.resolve(), journal.resolve())):
        need(target != root and root not in target.parents, "output_inside_parent_forbidden")
    info = os.statvfs(path.parent)
    need(info.f_bavail * info.f_frsize >= RESERVE + EVIDENCE_BUDGET, "reserve_2gib_evidence_budget")
    return path


def clock_evidence(account, context, floor, stage, storage, time_lock):
    need(account["exists"] is True and account["owner"] == SYSVAR and account["executable"] is False and uint(account["lamports"]) > 0, "clock_account_identity")
    text = account["dataBase64"]
    need(type(text) is str and len(text) == 56, "clock_base64")
    try: raw = base64.b64decode(text, validate=True)
    except ValueError as exc: raise Stop("clock_base64") from exc
    need(len(raw) == 40 and base64.b64encode(raw).decode() == text, "clock_base64")
    slot, start, epoch, leader, now = struct.unpack("<QqQQq", raw)
    need(floor <= slot <= context and context >= floor, "clock_context_or_slot_stale")
    result = {"address": CLOCK, "dataSha256": sha(raw), "contextSlot": str(context), "slot": str(slot),
        "epochStartTimestamp": str(start), "epoch": str(epoch), "leaderScheduleEpoch": str(leader), "unixTimestamp": str(now),
        "timelockArithmeticSatisfiedByCollectedClock": None, "timelockCheck": "NOT_APPLICABLE_PRE_VOTE"}
    if stage == "POST_VOTE":
        timestamp = int(storage["evidence"]["proposal"]["timestamp"])
        elapsed = now - timestamp
        need(I64_MIN <= elapsed <= I64_MAX, "timelock_i64_subtraction_overflow")
        need(elapsed >= time_lock, "timelock_not_satisfied")
        result.update(timelockArithmeticSatisfiedByCollectedClock=True, timelockCheck="SIGNED_I64_SUBTRACTION_AND_CONFIGURED_DELAY_SATISFIED",
            approvedTimestamp=str(timestamp), elapsedSeconds=str(elapsed), configuredDelaySeconds=str(time_lock))
    return result


def rpc_result(c, body, index):
    replies = c.strict_json(body)
    need(type(replies) is list and len(replies) == 1 and type(replies[0]) is dict, "rpc_batch_shape")
    response = replies[0]
    need(response.get("jsonrpc") == "2.0" and type(response.get("id")) is int and response["id"] == index, "rpc_envelope_or_id")
    need(("result" in response) != ("error" in response), "rpc_result_error")
    need("error" not in response, "rpc_error")
    return response["result"]


def collect(value, parent_path, output, *, _transport=None):
    need(_transport is None or callable(_transport), "offline_transport_callable")
    if not request_shape(value): return {"status": "INCOMPLETE", **FLAGS}
    c, s, j = helpers()
    parent = inspect_parent(value, parent_path, j); seal = parent["initializerSeal"]; plan = seal["unsignedInitializerPlan"]
    # CLI cannot query the portable synthetic profile; injected functions are test-only.
    need(_transport is not None or tls_profile(seal), "actual_testnet_profile_required_before_collection")
    out = output_path(output, parent_path)
    keys = [plan[name] for name in ("multisig", "transaction", "proposal")] + [CLOCK]
    floor = minimum(value, seal); store = c.Store(out); written = 0
    counts = {"httpAttemptsStarted": 0, "httpResponsesCompleted": 0, "httpTransportFailures": 0, "logicalReadsStarted": 0, "rpcErrors": 0, "liveTransportInvocationsStarted": 0, "injectedTransportInvocationsStarted": 0}
    report = {"schema": SCHEMA, "status": "STOP", "reason": "NOT_COMPLETE", "endpoint": c.ENDPOINT, "expectedGenesis": c.GENESIS,
        "transportMode": "FIXED_ORIGIN_TLS" if _transport is None else "OFFLINE_INJECTED_TEST", "fixedOriginTlsResponseReceived": False,
        "genesisMatched": False, "parentTail": parent["tail"], "sealDataSha256": sha(j.encode(seal)),
        "postManifestSha256": seal["postManifestSha256"], "inheritedLedgerReference": parent["initializerProfile"]["parentLedger"],
        "minimumContextSlot": str(floor), "accountKeys": keys, "sourcePins": PINS, "preCollectionNodeProcessInvocations": 2, "storageReport": None, "clockEvidence": None, **FLAGS}
    def write(name, raw):
        nonlocal written
        need(written + len(raw) + 4096 <= EVIDENCE_BUDGET, "evidence_budget")
        space = os.fstatvfs(store.fd)
        need(space.f_bavail * space.f_frsize >= RESERVE + len(raw) + 4096, "reserve_2gib")
        receipt = store.write(name, raw); written += len(raw); return receipt
    def query(method, params):
        need(counts["httpAttemptsStarted"] < 2 and method in ("getGenesisHash", "getMultipleAccounts"), "query_budget_or_method")
        index = counts["httpAttemptsStarted"] + 1; prefix = f"http-{index:02}"
        requests = [{"jsonrpc": "2.0", "id": index, "method": method, "params": params}]; raw = c.encode(requests)
        write(prefix + ".request.json", raw)
        prospective = counts.copy(); prospective["httpAttemptsStarted"] += 1; prospective["logicalReadsStarted"] += 1
        prospective["liveTransportInvocationsStarted" if _transport is None else "injectedTransportInvocationsStarted"] += 1
        write(prefix + ".started.json", encode({"requestStartedUtc": utc(), "counts": prospective, "requestIds": [index]}))
        counts.update(prospective)
        try: status, headers, body = (_transport if _transport is not None else c.tls_post)(raw)
        except Exception as exc:
            counts["httpTransportFailures"] += 1
            if isinstance(exc, c.TransportFailure) and exc.partial: write(prefix + ".partial-response.bin", exc.partial[:c.MAX_JSON + 1])
            write(prefix + ".failure.json", encode({"transportFailedUtc": utc(), "reason": str(exc) if isinstance(exc, c.Stop) else "transport_failure"}))
            raise Stop(str(exc) if isinstance(exc, c.Stop) else "transport_failure") from exc
        completed = utc(); counts["httpResponsesCompleted"] += 1
        if _transport is None: report["fixedOriginTlsResponseReceived"] = True
        need(type(body) is bytes and len(body) <= c.MAX_JSON, "response_bound")
        write(prefix + ".response.bin", body); write(prefix + ".http.json", encode({"status": status, "headers": headers, "responseCompletedUtc": completed}))
        need(type(status) is int and status == 200, "redirect_forbidden" if type(status) is int and 300 <= status < 400 else "http_status")
        try: return rpc_result(c, body, index)
        except Stop as exc:
            if str(exc) == "rpc_error": counts["rpcErrors"] += 1
            raise
    try:
        write("input.json", encode(value))
        write("parent-reference.json", encode({k: report[k] for k in ("parentTail", "sealDataSha256", "postManifestSha256", "inheritedLedgerReference", "sourcePins")}))
        need(query("getGenesisHash", []) == c.GENESIS, "genesis_mismatch"); report["genesisMatched"] = True
        result = query("getMultipleAccounts", [keys, {"encoding": "base64", "commitment": "finalized", "minContextSlot": floor}])
        need(type(result) is dict and type(result.get("context")) is dict, "accounts_result")
        context = c.u64(result["context"].get("slot")); need(context >= floor, "accounts_context_stale")
        values = result.get("value"); need(type(values) is list and len(values) == 4, "accounts_length")
        normalized = {}
        for address, raw in zip(keys, values):
            if raw is not None:
                need(type(raw) is dict and type(raw.get("data")) is list and len(raw["data"]) == 2, "account_data")
                c.b64(raw["data"][0], 4096, True)
            normalized[address] = c.normalize_account(raw)
        snapshot = {"contextSlot": str(context), "accounts": {address: normalized[address] for address in keys[:3]}}
        clock = normalized[CLOCK]
        write("storage-snapshot.json", encode(snapshot)); write("clock-account.json", encode({"address": CLOCK, "contextSlot": str(context), **clock}))
        storage_request = {"schema": s.SCHEMA, "expectedTail": value["expectedTail"], "stage": value["stage"], "snapshot": snapshot}
        write("storage-request.json", encode(storage_request))
        storage = s.check(storage_request, parent_path)
        need(storage["status"] == "VERIFIED_SUPPLIED_INITIALIZER_STORAGE_ONLY" and storage["parentTail"] == parent["tail"] and storage["sealDataSha256"] == report["sealDataSha256"] and storage["inheritedLedgerReference"] == report["inheritedLedgerReference"], "parent_or_storage_rebind")
        report["storageReport"] = storage; write("storage-report.json", encode(storage))
        timing = clock_evidence(clock, context, floor, value["stage"], storage, seal["postManifest"]["initializer"]["governance"]["timeLockSeconds"])
        report.update(status="VERIFIED_COLLECTED_STORAGE_AND_CLOCK_EVIDENCE_ONLY", reason="EXACT_SEALED_STORAGE_AND_CONTEXTUAL_CLOCK_CHECKS", clockEvidence=timing,
            snapshotContextSlot=str(context), snapshotSha256=sha(encode(snapshot)), checkerProcessInvocations=2, initializerPlannerProcessInvocations=2,
            pdaProcessInvocations=1, nodeProcessInvocations=5)
        write("clock-evidence.json", encode(timing))
    except Exception as exc:
        report.update(status="STOP", reason=str(exc) if isinstance(exc, (Stop, c.Stop, s.Stop, j.Stop)) else "readback_failure")
    finally:
        report["counts"] = counts.copy(); report["artifacts"] = store.artifacts.copy()
        try: report["manifestReceipt"] = write("manifest.json", encode(report))
        finally: store.close()
    return report


def read_input(path):
    with open(path, "rb") as stream: raw = stream.read(MAX_INPUT + 1)
    need(len(raw) <= MAX_INPUT, "input_bound")
    def pairs(items):
        value = {}
        for key, child in items:
            need(key not in value, "duplicate_json_key"); value[key] = child
        return value
    def invalid(_): raise Stop("noninteger_or_nonfinite_input_json")
    try: value = json.loads(raw.decode(), object_pairs_hook=pairs, parse_float=invalid, parse_constant=invalid)
    except (UnicodeError, ValueError, RecursionError) as exc:
        if isinstance(exc, Stop): raise
        raise Stop("input_json") from exc
    def depth(item, level=0):
        need(level <= 32, "input_depth")
        if type(item) is dict:
            for child in item.values(): depth(child, level + 1)
        elif type(item) is list:
            for child in item: depth(child, level + 1)
    depth(value)
    return value


def main(argv):
    try:
        if argv == ["--template"]: result = TEMPLATE
        elif len(argv) == 4 and argv[0] == "--validate" and argv[2] == "--journal": result = validate(read_input(argv[1]), argv[3])
        elif len(argv) == 6 and argv[0] == "--collect" and argv[2] == "--journal" and argv[4] == "--output": result = collect(read_input(argv[1]), argv[3], argv[5])
        else: raise Stop("usage")
        sys.stdout.buffer.write(encode(result)); return 1 if result.get("status") == "STOP" else 0
    except Exception as exc:
        sys.stdout.buffer.write(encode({"status": "STOP", "reason": str(exc) if isinstance(exc, Stop) else "readback_failure", **FLAGS})); return 1


if __name__ == "__main__": sys.exit(main(sys.argv[1:]))
