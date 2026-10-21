#!/usr/bin/env python3
"""Offline durable public checker segments. No network, signer or sender."""
import base64
import copy
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import selectors
import stat
import subprocess
import sys
import time

SCHEMA = "piv1-startup-public-journal-request-v1"
RECORD_SCHEMA = "piv1-startup-public-journal-record-v1"
TEMPLATE = {"schema": SCHEMA, "action": None, "expectedTail": None, "segment": None, "event": None}
NODE = Path("/home/jerem/.local/piv1-toolchains/node-v24.19.0-linux-x64/bin/node")
NODE_SHA = "bc17c508ffeed0ec622934f9b7fa72f8e78da65350e63c3eceb56fa688aa5e12"
CHECKER_SHA = "50080d7baa17098e4ee687dcf0abb5fd4cef27e88154edd0c32b47e4f033fce9"
SDK_SHA = "796768bf5a689d808fb6fabfc5ce0db27eee064782cf87621331b807bab9f7e2"
LOCK_SHA = "a47e4310aa29ffb2ec0943626616bfefb0f0b8d7408090b360018b7e5e215e89"
LOCK_BYTES = b"piv1-startup-public-journal-lock-v1\n"
MAX_INPUT = 16 * 1024 * 1024
MAX_JOURNAL = 64 * 1024 * 1024
MAX_RECORDS = 1024
MAX_SEGMENTS = 32
RESERVE = 2 * 1024 * 1024 * 1024
U64 = (1 << 64) - 1
NODE_SCRIPT = r"""
const fs=require('node:fs');
const c=require(process.env.PIV1_PUBLIC_CHECK_REPO+'/clients/cli/startup-public-check.cjs');
c.verifySources();
const input=fs.readFileSync(0);if(input.length>16777216)process.exit(2);
const manifests=JSON.parse(input.toString('utf8'));
const out=JSON.stringify(manifests.map(m=>c.check(m)));
if(Buffer.byteLength(out)>524288)process.exit(3);
process.stdout.write(out);
"""


class Stop(ValueError):
    def __init__(self, code, diagnostics=None):
        super().__init__(code)
        self.diagnostics = diagnostics


def require(condition, code, diagnostics=None):
    if not condition:
        raise Stop(code, diagnostics)


def encode(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False) + "\n").encode()


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def fields(value, keys):
    require(type(value) is dict and set(value) == set(keys), "object_fields")


def u64(value):
    require(type(value) is str and re.fullmatch(r"0|[1-9][0-9]{0,19}", value) is not None and int(value) <= U64, "u64_string")
    return int(value)


def digest(value):
    require(type(value) is str and re.fullmatch("[0-9a-f]{64}", value) is not None, "digest")
    return value


def parse(raw):
    require(type(raw) is bytes and len(raw) <= MAX_INPUT, "json_bound")
    def pairs(items):
        value = {}
        for key, item in items:
            require(key not in value, "duplicate_json_key")
            value[key] = item
        return value
    def invalid(_):
        raise Stop("noninteger_or_nonfinite_json")
    try:
        value = json.loads(raw.decode(), object_pairs_hook=pairs, parse_float=invalid, parse_constant=invalid)
    except (UnicodeError, json.JSONDecodeError, RecursionError) as exc:
        raise Stop("invalid_json") from exc
    def depth(item, level=0):
        require(level <= 32, "json_depth")
        if type(item) is dict:
            for child in item.values(): depth(child, level + 1)
        if type(item) is list:
            for child in item: depth(child, level + 1)
    depth(value)
    return value


def pins():
    repo = Path(os.environ.get("PIV1_PUBLIC_CHECK_REPO", Path(__file__).resolve().parent.parent))
    expected = {NODE: NODE_SHA, repo / "clients/cli/startup-public-check.cjs": CHECKER_SHA,
                repo / "spikes/task-0.4-jito/package-lock.json": LOCK_SHA,
                repo / "spikes/task-0.4-jito/node_modules/@solana/web3.js/lib/index.cjs.js": SDK_SHA}
    for path, want in expected.items():
        require(not path.is_symlink() and path.is_file(), "source_file")
        h = hashlib.sha256()
        with path.open("rb") as stream:
            for chunk in iter(lambda: stream.read(1024 * 1024), b""): h.update(chunk)
        require(h.hexdigest() == want, "source_pin")
    return repo


def check_segments(segments):
    repo = pins()
    raw = encode([segment["input"] for segment in segments])
    require(len(raw) <= MAX_INPUT, "checker_input_bound")
    process = subprocess.Popen([str(NODE), "-e", NODE_SCRIPT], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, cwd=repo, env={"PIV1_PUBLIC_CHECK_REPO": str(repo)}, close_fds=True)
    selector = selectors.DefaultSelector()
    streams = (process.stdin, process.stdout, process.stderr)
    output = {process.stdout: bytearray(), process.stderr: bytearray()}
    position = 0
    deadline = time.monotonic() + 20
    try:
        for stream in streams:
            os.set_blocking(stream.fileno(), False)
            selector.register(stream, selectors.EVENT_WRITE if stream is process.stdin else selectors.EVENT_READ)
        while selector.get_map():
            remaining = deadline - time.monotonic()
            require(remaining > 0, "checker_timeout")
            for item, _ in selector.select(min(remaining, 0.5)):
                stream = item.fileobj
                if stream is process.stdin:
                    try: position += os.write(stream.fileno(), raw[position:position + 65536])
                    except BrokenPipeError: raise Stop("checker_stdin")
                    if position == len(raw):
                        selector.unregister(stream)
                        stream.close()
                else:
                    chunk = os.read(stream.fileno(), 65536)
                    if not chunk:
                        selector.unregister(stream)
                    else:
                        output[stream].extend(chunk)
                        require(len(output[stream]) <= (524288 if stream is process.stdout else 16384), "checker_output_bound")
        require(process.wait(timeout=max(0.001, deadline - time.monotonic())) == 0, "checker_process_failure")
        require(not output[process.stderr], "checker_stderr")
        reports = parse(bytes(output[process.stdout]))
        require(type(reports) is list and len(reports) == len(segments), "checker_reports")
        return reports
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()
        selector.close()
        for stream in streams:
            if not stream.closed: stream.close()


def directory(path, create=False):
    path = Path(os.path.abspath(path))
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
    parent = os.open("/", flags)
    try:
        for component in path.parent.parts[1:]:
            nxt = os.open(component, flags, dir_fd=parent)
            os.close(parent)
            parent = nxt
        if create:
            space(parent, 8192)
            os.mkdir(path.name, 0o700, dir_fd=parent)
        fd = os.open(path.name, flags, dir_fd=parent)
        try:
            meta = os.fstat(fd)
            require(meta.st_uid == os.getuid() and stat.S_IMODE(meta.st_mode) == 0o700, "journal_mode_owner")
            if create: os.fsync(fd); os.fsync(parent)
            return fd
        except Exception:
            os.close(fd)
            raise
    finally:
        os.close(parent)


def secure_meta(meta):
    require(stat.S_ISREG(meta.st_mode) and meta.st_uid == os.getuid() and stat.S_IMODE(meta.st_mode) == 0o600 and meta.st_nlink == 1, "record_mode_owner_link")


def space(fd, amount):
    info = os.fstatvfs(fd)
    require(info.f_bavail * info.f_frsize >= RESERVE + amount, "reserve_2gib")


def write_file(fd, name, raw):
    space(fd, len(raw) + 4096)
    out = os.open(name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600, dir_fd=fd)
    try:
        os.fchmod(out, 0o600)
        view = memoryview(raw)
        while view:
            count = os.write(out, view)
            require(count > 0, "short_write")
            view = view[count:]
        os.fsync(out)
    finally:
        os.close(out)
    os.fsync(fd)


def read_file(fd, name, maximum):
    source = os.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=fd)
    try:
        meta = os.fstat(source)
        secure_meta(meta)
        require(meta.st_size <= maximum, "record_bound")
        raw = bytearray()
        while True:
            chunk = os.read(source, min(65536, maximum + 1 - len(raw)))
            if not chunk: break
            raw.extend(chunk)
            require(len(raw) <= maximum, "record_bound")
        return bytes(raw)
    finally:
        os.close(source)


def records(fd):
    names = os.listdir(fd)
    require("lock" in names and len(names) <= MAX_RECORDS + 1 and all(name == "lock" or re.fullmatch(r"[0-9]{8}\.json", name) for name in names), "journal_namespace")
    names = sorted(name for name in names if name != "lock")
    require(names and names == [f"{i:08}.json" for i in range(len(names))], "record_gap_or_missing")
    chain = []
    total = 0
    previous = None
    for i, name in enumerate(names):
        raw = read_file(fd, name, MAX_INPUT)
        total += len(raw)
        require(total <= MAX_JOURNAL, "journal_bound")
        record = parse(raw)
        fields(record, ("schema", "sequence", "previousDigest", "lockIdentity", "kind", "data", "digest"))
        require(encode(record) == raw and record["schema"] == RECORD_SCHEMA and record["sequence"] == str(i) and record["previousDigest"] == previous, "record_order_or_canonical")
        body = {key: value for key, value in record.items() if key != "digest"}
        require(digest(record["digest"]) == sha(encode(body)), "record_digest")
        fields(record["lockIdentity"], ("device", "inode", "ctimeNs"))
        u64(record["lockIdentity"]["device"]); u64(record["lockIdentity"]["inode"]); u64(record["lockIdentity"]["ctimeNs"])
        require(not chain or record["lockIdentity"] == chain[0]["lockIdentity"], "lock_identity_changed")
        previous = record["digest"]
        chain.append(record)
    return chain, total


def tail(chain):
    return {"sequence": chain[-1]["sequence"], "digest": chain[-1]["digest"]}


def reconstruct(chain):
    segments = []
    ids = set()
    for record in chain:
        if record["kind"] == "BEGIN":
            segment = copy.deepcopy(record["data"])
            fields(segment, ("id", "input"))
            require(type(segment["id"]) is str and re.fullmatch(r"[A-Za-z0-9_-]{1,64}", segment["id"]) is not None and segment["id"] not in ids, "segment_id")
            require(type(segment["input"]) is dict and segment["input"].get("trace") == [], "segment_empty_trace_required")
            ids.add(segment["id"])
            segments.append(segment)
            require(len(segments) <= MAX_SEGMENTS, "segment_bound")
        elif record["kind"] == "EVENT":
            require(segments, "event_before_segment")
            event = record["data"]
            require(type(event) is dict, "event_object")
            prior = segments[-1]["input"]["trace"]
            require(not prior or prior[-1].get("state") != "FAILED", "failed_segment_halted")
            prior.append(copy.deepcopy(event))
        else:
            raise Stop("record_kind")
    require(segments, "segment_required")
    return segments


def validate_chain(chain):
    segments = reconstruct(chain)
    reports = check_segments(segments)
    first = segments[0]["input"]
    cap = u64(first.get("cumulativeFeeCapLamports"))
    phases = set(); packets = set(); signatures = set()
    reserved = actual = 0
    last_slot = None
    for i, (segment, report) in enumerate(zip(segments, reports)):
        data = segment["input"]
        require(data.get("frozen") == first.get("frozen") and data.get("cumulativeFeeCapLamports") == first.get("cumulativeFeeCapLamports"), "cross_segment_freeze_or_cap")
        success = report.get("status") == "VERIFIED_PUBLIC_DECLARATIONS_ONLY"
        pending = report.get("status") == "STOP" and report.get("reason") == "PENDING_OR_UNKNOWN_NO_REPLACEMENT"
        failed = report.get("status") == "STOP" and report.get("reason") == "FINALIZED_FAILURE"
        require(success or pending or failed, "checker_rejected", report)
        require(i == len(segments) - 1 or success, "predecessor_unresolved", report)
        reserved += u64(report.get("reservedFeeLamports")); actual += u64(report.get("actualFeeLamports"))
        require(reserved <= min(cap, U64) and actual <= min(cap, U64), "global_fee_cap")
        for phase in data["phases"]:
            require(phase["id"] not in phases, "global_phase_replay")
            phases.add(phase["id"])
            raw = base64.b64decode(phase["packet"]["signedWireBase64"], validate=True)
            packet_hash, signature = sha(raw), raw[1:65].hex()
            require(packet_hash not in packets and signature not in signatures, "global_packet_replay")
            packets.add(packet_hash); signatures.add(signature)
        for event in data["trace"]:
            if event["state"] == "FINALIZED":
                slot = u64(event["observation"]["status"]["slot"])
                require(last_slot is None or slot > last_slot, "global_slot_not_later")
                last_slot = slot
    return {"segments": len(segments), "reservedFeeLamports": str(reserved), "actualFeeLamports": str(actual), "checkerReports": reports, "checkerProcessInvocations": 1, "nextSegmentAllowedByDeclarations": reports[-1].get("status") == "VERIFIED_PUBLIC_DECLARATIONS_ONLY"}


def make_record(chain, kind, data, lock_identity):
    body = {"schema": RECORD_SCHEMA, "sequence": str(len(chain)), "previousDigest": chain[-1]["digest"] if chain else None, "lockIdentity": lock_identity, "kind": kind, "data": data}
    return {**body, "digest": sha(encode(body))}


def apply(request, journal):
    request = copy.deepcopy(request)
    fields(request, TEMPLATE)
    require(request["schema"] == SCHEMA, "schema")
    action = request["action"]
    if action is None: return {"status": "INCOMPLETE", "liveReady": False, "authorized": False}
    require(action in ("CREATE", "START_SEGMENT", "APPEND_EVENT", "INSPECT"), "action")
    create = action == "CREATE"
    if create: require(request["expectedTail"] is None, "create_tail_null")
    else:
        fields(request["expectedTail"], ("sequence", "digest")); u64(request["expectedTail"]["sequence"]); digest(request["expectedTail"]["digest"])
    require((request["segment"] is not None) == (action in ("CREATE", "START_SEGMENT")) and (request["event"] is not None) == (action == "APPEND_EVENT"), "action_fields")
    fd = directory(journal, create)
    lock = None
    try:
        if create: write_file(fd, "lock", LOCK_BYTES)
        lock = os.open("lock", os.O_RDWR | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=fd)
        meta = os.fstat(lock); secure_meta(meta)
        fcntl.flock(lock, fcntl.LOCK_EX)
        current = os.stat("lock", dir_fd=fd, follow_symlinks=False)
        require(current.st_ino == meta.st_ino and current.st_dev == meta.st_dev and current.st_ctime_ns == meta.st_ctime_ns and read_file(fd, "lock", 128) == LOCK_BYTES, "lock_replaced_or_corrupt")
        if create:
            require(os.listdir(fd) == ["lock"], "new_journal_not_empty")
            chain, total = [], 0
        else:
            chain, total = records(fd)
            require(chain[0]["lockIdentity"] == {"device": str(meta.st_dev), "inode": str(meta.st_ino), "ctimeNs": str(meta.st_ctime_ns)}, "stable_lock_replaced")
            require(tail(chain) == request["expectedTail"], "tail_compare_and_swap", {"expectedTail": request["expectedTail"], "observedTail": tail(chain)})
        if action == "INSPECT":
            checked = validate_chain(chain)
            return {"status": "VERIFIED_LOCAL_JOURNAL_ONLY", "tail": tail(chain), **checked, "liveReady": False, "authorized": False, "callerAnchorAuthenticated": False}
        record = make_record(chain, "EVENT" if action == "APPEND_EVENT" else "BEGIN", request["event"] if action == "APPEND_EVENT" else request["segment"], {"device": str(meta.st_dev), "inode": str(meta.st_ino), "ctimeNs": str(meta.st_ctime_ns)})
        candidate = chain + [record]
        require(len(candidate) <= MAX_RECORDS, "record_count_bound")
        checked = validate_chain(candidate)
        raw = encode(record)
        require(len(raw) <= MAX_INPUT and total + len(raw) <= MAX_JOURNAL, "journal_bound")
        current = os.stat("lock", dir_fd=fd, follow_symlinks=False)
        require(current.st_ino == meta.st_ino and current.st_dev == meta.st_dev and current.st_ctime_ns == meta.st_ctime_ns, "lock_replaced")
        try:
            write_file(fd, f"{len(chain):08}.json", raw)
        except Exception as exc:
            raise Stop("commit_uncertain_no_retry_or_adoption", {"lastAcknowledgedTail": tail(chain) if chain else None, "candidateTail": tail(candidate)}) from exc
        return {"status": "DURABLE_PUBLIC_RECORD_ONLY", "tail": tail(candidate), **checked, "liveReady": False, "authorized": False,
                "callerAnchorAuthenticated": False, "chainStateAuthenticated": False, "operatorOrClockController": False}
    finally:
        if lock is not None:
            fcntl.flock(lock, fcntl.LOCK_UN)
            os.close(lock)
        os.close(fd)


def main(argv):
    try:
        if argv == ["--template"]: result = TEMPLATE
        elif len(argv) == 4 and argv[0] == "--request" and argv[2] == "--journal":
            with open(argv[1], "rb") as stream: value = parse(stream.read(MAX_INPUT + 1))
            result = apply(value, argv[3])
        else: raise Stop("usage")
        sys.stdout.buffer.write(encode(result))
        return 0
    except Exception as exc:
        result = {"status": "STOP", "reason": str(exc) if isinstance(exc, Stop) else "journal_failure", "liveReady": False, "authorized": False}
        if isinstance(exc, Stop) and exc.diagnostics is not None: result["diagnostics"] = exc.diagnostics
        sys.stdout.buffer.write(encode(result))
        return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
