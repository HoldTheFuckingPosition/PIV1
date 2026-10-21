#!/usr/bin/env python3
"""Fixed-origin read-only public evidence collection; never a signer or sender."""
import base64
import hashlib
import http.client
import json
import os
from pathlib import Path
import re
import ssl
import stat
import sys
from datetime import datetime, timezone
from decimal import Decimal

ENDPOINT = "https://api.testnet.solana.com"
HOST = "api.testnet.solana.com"
GENESIS = "4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY"
SCHEMA = "piv1-startup-public-collector-v1"
CHECKER_SHA256 = "50080d7baa17098e4ee687dcf0abb5fd4cef27e88154edd0c32b47e4f033fce9"
U64 = (1 << 64) - 1
MAX_JSON = 16 * 1024 * 1024
MAX_ACCOUNT = 1024 * 1024
ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
PROXIES = ("HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "http_proxy", "https_proxy", "all_proxy")
METHODS = frozenset(("getGenesisHash", "getSignatureStatuses", "getTransaction", "getMultipleAccounts", "getBlockHeight", "isBlockhashValid"))
TEMPLATE = {"schema": SCHEMA, "wireBase64": None, "expectedWireSha256": None,
            "expectedMessageSha256": None, "signature": None, "expectedFeeLamports": None,
            "accountKeys": None, "minimumContextSlot": None, "includeExpiry": False}


class Stop(ValueError):
    pass


def require(condition, code):
    if not condition:
        raise Stop(code)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def encode(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False) + "\n").encode()


def strict_json(raw):
    require(isinstance(raw, bytes) and len(raw) <= MAX_JSON, "json_bound")
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, "duplicate_json_key")
            result[key] = value
        return result
    def constant(_):
        raise Stop("json_nonfinite")
    try:
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=pairs, parse_constant=constant, parse_float=Decimal)
    except (UnicodeError, json.JSONDecodeError, RecursionError) as exc:
        raise Stop("invalid_json") from exc
    def depth(item, level=0):
        require(level <= 32, "json_depth")
        if isinstance(item, dict):
            for child in item.values():
                depth(child, level + 1)
        elif isinstance(item, list):
            for child in item:
                depth(child, level + 1)
    depth(value)
    return value


def u64(value, string=False):
    if string:
        require(isinstance(value, str) and re.fullmatch(r"0|[1-9][0-9]{0,19}", value) is not None, "u64_string")
        value = int(value)
    require(type(value) is int and 0 <= value <= U64, "u64_range")
    return value


def b64(value, maximum, empty=False):
    require(isinstance(value, str) and len(value) <= 4 * ((maximum + 2) // 3), "base64_bound")
    require((empty or bool(value)) and len(value) % 4 == 0, "base64")
    try:
        raw = base64.b64decode(value, validate=True)
    except (ValueError, base64.binascii.Error) as exc:
        raise Stop("base64") from exc
    require(len(raw) <= maximum and base64.b64encode(raw).decode() == value, "base64")
    return raw


def b58encode(raw):
    number = int.from_bytes(raw, "big")
    result = ""
    while number:
        number, digit = divmod(number, 58)
        result = ALPHABET[digit] + result
    return "1" * (len(raw) - len(raw.lstrip(b"\0"))) + result


def b58decode(value, size):
    require(isinstance(value, str) and 1 <= len(value) <= (44 if size == 32 else 88), "base58_bound")
    number = 0
    for char in value:
        require(char in ALPHABET, "base58")
        number = number * 58 + ALPHABET.index(char)
    raw = b"\0" * (len(value) - len(value.lstrip("1"))) + (number.to_bytes((number.bit_length() + 7) // 8, "big") if number else b"")
    require(len(raw) == size and b58encode(raw) == value, "base58_size")
    return raw


def packet(value):
    raw = b64(value, 1232)
    count = 0
    end = 0
    for i in range(min(3, len(raw))):
        byte = raw[i]
        require(i < 2 or byte <= 3, "shortvec_overflow")
        count |= (byte & 127) << (7 * i)
        if not byte & 128:
            end = i + 1
            break
    require(end and 1 <= count <= 127 and end == 1, "signature_count_noncanonical")
    offset = end + count * 64
    require(len(raw) >= offset + 3, "packet_truncated")
    message = raw[offset:]
    versioned = bool(message[0] & 128)
    require(not versioned or message[0] == 128, "message_version")
    require(message[1 if versioned else 0] == count, "message_signature_count")
    signature = raw[end:end + 64]
    require(any(signature), "zero_signature")
    # Full message/ALT parsing and Ed25519 verification belong to the pinned checker.
    return {"wire": raw, "version": 0 if versioned else "legacy", "messageSha256": sha(message), "signature": b58encode(signature), "signatureHex": signature.hex()}


def validate(value):
    require(isinstance(value, dict) and set(value) == set(TEMPLATE), "input_fields")
    require(value["schema"] == SCHEMA, "schema")
    missing = [key for key in ("wireBase64", "expectedWireSha256", "expectedFeeLamports", "accountKeys", "minimumContextSlot") if value[key] is None]
    if missing:
        return {"status": "INCOMPLETE", "missing": missing}
    require(type(value["includeExpiry"]) is bool, "include_expiry")
    p = packet(value["wireBase64"])
    require(isinstance(value["expectedWireSha256"], str) and re.fullmatch("[0-9a-f]{64}", value["expectedWireSha256"]) is not None, "wire_hash")
    require(sha(p["wire"]) == value["expectedWireSha256"], "wire_hash_mismatch")
    if value["expectedMessageSha256"] is not None:
        require(value["expectedMessageSha256"] == p["messageSha256"], "message_hash_mismatch")
    if value["signature"] is not None:
        require(b58decode(value["signature"], 64).hex() == p["signatureHex"], "signature_mismatch")
    u64(value["minimumContextSlot"], True)
    u64(value["expectedFeeLamports"], True)
    keys = value["accountKeys"]
    require(isinstance(keys, list) and 1 <= len(keys) <= 64 and all(isinstance(k, str) for k in keys), "account_keys")
    require(len(set(keys)) == len(keys), "duplicate_account")
    for key in keys:
        b58decode(key, 32)
    return {"status": "VALID_PUBLIC_REQUEST_ONLY", "packet": p}


class Store:
    """New public evidence directory; every file is exclusive and immutable here."""
    def __init__(self, output):
        self.path = Path(os.path.abspath(output))
        flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
        parent_fd = os.open("/", flags)
        try:
            for component in self.path.parent.parts[1:]:
                next_fd = os.open(component, flags, dir_fd=parent_fd)
                os.close(parent_fd)
                parent_fd = next_fd
            os.mkdir(self.path.name, 0o700, dir_fd=parent_fd)
            self.fd = os.open(self.path.name, flags, dir_fd=parent_fd)
            require(stat.S_ISDIR(os.fstat(self.fd).st_mode), "output_directory")
            os.fchmod(self.fd, 0o700)
            os.fsync(self.fd)
            os.fsync(parent_fd)
        finally:
            os.close(parent_fd)
        self.artifacts = []

    def write(self, name, raw):
        require(re.fullmatch(r"[a-z0-9.-]+", name) is not None, "artifact_name")
        fd = os.open(name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600, dir_fd=self.fd)
        try:
            os.fchmod(fd, 0o600)
            view = memoryview(raw)
            while view:
                written = os.write(fd, view)
                require(written > 0, "short_write")
                view = view[written:]
            os.fsync(fd)
        finally:
            os.close(fd)
        os.fsync(self.fd)
        record = {"name": name, "bytes": len(raw), "sha256": sha(raw)}
        self.artifacts.append(record)
        return record

    def close(self):
        os.close(self.fd)


class TransportFailure(Stop):
    def __init__(self, code, partial=b""):
        super().__init__(code)
        self.partial = partial


def tls_post(raw):
    require(not any(os.environ.get(key) for key in PROXIES), "proxy_environment_unsupported")
    connection = http.client.HTTPSConnection(HOST, 443, timeout=30, context=ssl.create_default_context())
    body = bytearray()
    try:
        connection.request("POST", "/", body=raw, headers={"Content-Type": "application/json", "Accept": "application/json", "User-Agent": "piv1-public-evidence-v1"})
        response = connection.getresponse()
        headers = list(response.getheaders())
        while True:
            chunk = response.read(min(65536, MAX_JSON + 1 - len(body)))
            if not chunk:
                if response.length not in (None, 0):
                    raise TransportFailure("incomplete_http_body", bytes(body))
                break
            body.extend(chunk)
            if len(body) > MAX_JSON:
                raise TransportFailure("response_bound", bytes(body))
        return response.status, headers, bytes(body)
    except TransportFailure:
        raise
    except (OSError, http.client.HTTPException) as exc:
        if isinstance(exc, http.client.IncompleteRead):
            body.extend(exc.partial)
        raise TransportFailure("tls_http_failure", bytes(body)) from exc
    finally:
        connection.close()


def checker_pin():
    repo = Path(os.environ.get("PIV1_PUBLIC_CHECK_REPO", Path(__file__).resolve().parent.parent))
    path = repo / "clients/cli/startup-public-check.cjs"
    raw = path.read_bytes()
    require(sha(raw) == CHECKER_SHA256, "checker_source_pin")
    return {"path": str(path), "bytes": len(raw), "sha256": sha(raw)}


def normalize_account(value):
    if value is None:
        return {"exists": False, "owner": None, "executable": None, "lamports": None, "dataBase64": None}
    require(isinstance(value, dict), "account_object")
    owner = value.get("owner")
    b58decode(owner, 32)
    require(type(value.get("executable")) is bool, "account_executable")
    data = value.get("data")
    require(isinstance(data, list) and len(data) == 2 and data[1] == "base64", "account_encoding")
    b64(data[0], MAX_ACCOUNT, True)
    return {"exists": True, "owner": owner, "executable": value["executable"], "lamports": str(u64(value.get("lamports"))), "dataBase64": data[0]}


def collect(value, output, *, _transport=None):
    require(_transport is None or callable(_transport), "offline_transport_callable")
    validated = validate(value)
    require(validated["status"] == "VALID_PUBLIC_REQUEST_ONLY", "incomplete_input")
    source = checker_pin()
    p = validated["packet"]
    store = Store(output)
    counts = {"httpAttemptsStarted": 0, "httpResponsesCompleted": 0, "httpTransportFailures": 0, "logicalReadsStarted": 0, "rpcErrors": 0}
    report = {"schema": SCHEMA, "endpoint": ENDPOINT, "expectedGenesis": GENESIS, "liveReady": False, "authorized": False,
              "cryptographicSignaturesVerified": False, "submissionJournalPersisted": False, "chainStateCryptographicallyAuthenticated": False,
              "transportMode": "FIXED_ORIGIN_TLS" if _transport is None else "OFFLINE_INJECTED_TEST", "tlsResponseReceived": False, "historicalInclusionStateClaimed": False, "replacementOrAdvancementAuthorized": False,
              "checkerSource": source, "packetSha256": sha(p["wire"]), "messageSha256": p["messageSha256"], "signature": p["signature"], "signatureHex": p["signatureHex"], "observation": None}
    next_id = 1
    def query(calls):
        nonlocal next_id
        require(counts["httpAttemptsStarted"] < 4 and counts["logicalReadsStarted"] + len(calls) <= 6, "query_budget")
        require(all(method in METHODS for method, _ in calls), "method_not_allowed")
        requests = [{"jsonrpc": "2.0", "id": next_id + i, "method": method, "params": params} for i, (method, params) in enumerate(calls)]
        next_id += len(calls)
        raw = encode(requests)
        index = counts["httpAttemptsStarted"] + 1
        prefix = f"http-{index:02}"
        store.write(prefix + ".request.json", raw)
        prospective = counts.copy()
        prospective["httpAttemptsStarted"] += 1
        prospective["logicalReadsStarted"] += len(calls)
        store.write(prefix + ".started.json", encode({"utc": datetime.now(timezone.utc).isoformat(), "counts": prospective, "requestIds": [q["id"] for q in requests]}))
        counts.update(prospective)
        try:
            status, headers, body = (_transport if _transport is not None else tls_post)(raw)
        except Exception as exc:
            counts["httpTransportFailures"] += 1
            if isinstance(exc, TransportFailure) and exc.partial:
                store.write(prefix + ".partial-response.bin", exc.partial[:MAX_JSON + 1])
            raise Stop(str(exc) if isinstance(exc, Stop) else "transport_failure") from exc
        completed_utc = datetime.now(timezone.utc).isoformat()
        counts["httpResponsesCompleted"] += 1
        if _transport is None:
            report["tlsResponseReceived"] = True
        store.write(prefix + ".response.bin", body)
        store.write(prefix + ".http.json", encode({"status": status, "headers": headers, "responseCompletedUtc": completed_utc}))
        require(type(status) is int and status == 200, "redirect_forbidden" if type(status) is int and 300 <= status < 400 else "http_status")
        responses = strict_json(body)
        require(isinstance(responses, list) and len(responses) == len(requests), "rpc_batch_shape")
        by_id = {}
        for response in responses:
            require(isinstance(response, dict) and response.get("jsonrpc") == "2.0" and type(response.get("id")) is int, "rpc_envelope")
            rid = response["id"]
            require(rid not in by_id and rid in [q["id"] for q in requests], "rpc_id")
            require(("result" in response) != ("error" in response), "rpc_result_error")
            if "error" in response:
                counts["rpcErrors"] += 1
                raise Stop("rpc_error")
            by_id[rid] = response["result"]
        return [by_id[q["id"]] for q in requests]
    try:
        store.write("input.json", encode(value))
        require(query([("getGenesisHash", [])])[0] == GENESIS, "genesis_mismatch")
        status_result, transaction = query([
            ("getSignatureStatuses", [[p["signature"]], {"searchTransactionHistory": True}]),
            ("getTransaction", [p["signature"], {"encoding": "base64", "commitment": "finalized", "maxSupportedTransactionVersion": 0}])])
        if value["includeExpiry"]:
            # Expiry is independent of inclusion; an old finalized transaction can reconcile.
            wire = p["wire"]
            offset = 1 + wire[0] * 64
            message = wire[offset:]
            start = 1 if message[0] == 128 else 0
            cursor = start + 3
            require(cursor < len(message) and message[cursor] < 128, "expiry_message_keys")
            cursor += 1 + message[cursor] * 32
            require(cursor + 32 <= len(message), "expiry_blockhash")
            blockhash = b58encode(message[cursor:cursor + 32])
            height, valid = query([("getBlockHeight", [{"commitment": "finalized"}]), ("isBlockhashValid", [blockhash, {"commitment": "finalized"}])])
            require(isinstance(valid, dict) and type(valid.get("value")) is bool and isinstance(valid.get("context"), dict), "blockhash_result")
            report["expiry"] = {"currentBlockHeight": str(u64(height)), "blockhashValid": valid["value"], "contextSlot": str(u64(valid["context"].get("slot"))), "nonInclusionProved": False}
        require(isinstance(status_result, dict) and isinstance(status_result.get("context"), dict), "status_result")
        context = u64(status_result["context"].get("slot"))
        values = status_result.get("value")
        require(isinstance(values, list) and len(values) == 1, "status_values")
        status = values[0]
        if status is None or transaction is None:
            report.update(status="UNKNOWN", reason="NULL_STATUS_OR_TRANSACTION_NO_NONINCLUSION_PROOF")
        else:
            require(isinstance(status, dict) and isinstance(transaction, dict), "transaction_status_objects")
            slot = u64(status.get("slot"))
            require(context >= slot and u64(transaction.get("slot")) == slot, "transaction_slot_mismatch")
            raw_tx = transaction.get("transaction")
            require(isinstance(raw_tx, list) and len(raw_tx) == 2 and raw_tx[1] == "base64", "transaction_encoding")
            require(b64(raw_tx[0], 1232) == p["wire"], "transaction_wire_mismatch")
            version = transaction.get("version")
            require((type(version) is int and version == 0) or version == "legacy", "transaction_version")
            require(version == p["version"], "transaction_version_mismatch")
            meta = transaction.get("meta")
            require(isinstance(meta, dict) and "err" in meta and "err" in status, "transaction_meta")
            require(status["err"] is None or isinstance(status["err"], (dict, str)), "transaction_error_type")
            fee = u64(meta.get("fee"))
            require(fee == u64(value["expectedFeeLamports"], True), "transaction_fee_mismatch")
            require(encode(status["err"]) == encode(meta["err"]), "transaction_error_mismatch")
            confirmation = status.get("confirmationStatus")
            require(confirmation in ("processed", "confirmed", "finalized"), "confirmation_status")
            observation = {"status": {"signatureHex": p["signatureHex"], "slot": str(slot), "confirmationStatus": confirmation, "err": status["err"]},
                           "transaction": {"slot": str(slot), "wireBase64": raw_tx[0], "feeLamports": str(fee), "err": meta["err"]}, "poststate": None}
            report["observation"] = observation
            if confirmation != "finalized":
                report.update(status="UNKNOWN", reason="NOT_FINALIZED")
            elif meta["err"] is not None:
                report.update(status="STOP", reason="FINALIZED_TRANSACTION_FAILURE")
            else:
                minimum = max(u64(value["minimumContextSlot"], True), slot)
                accounts = query([("getMultipleAccounts", [value["accountKeys"], {"encoding": "base64", "commitment": "finalized", "minContextSlot": minimum}])])[0]
                require(isinstance(accounts, dict) and isinstance(accounts.get("context"), dict), "accounts_result")
                account_context = u64(accounts["context"].get("slot"))
                require(account_context >= minimum, "accounts_context_stale")
                raw_accounts = accounts.get("value")
                require(isinstance(raw_accounts, list) and len(raw_accounts) == len(value["accountKeys"]), "accounts_length")
                observation["poststate"] = {"contextSlot": str(account_context), "accounts": {key: normalize_account(raw) for key, raw in zip(value["accountKeys"], raw_accounts)}}
                report.update(status="COLLECTED_PUBLIC_EVIDENCE_ONLY", reason="FINALIZED_TRANSACTION_AND_CURRENT_DECLARED_ACCOUNT_SUBSET")
        store.write("observation.json", encode(report["observation"]))
    except Exception as exc:
        report.update(status="STOP", reason=str(exc) if isinstance(exc, Stop) else "collector_failure")
    finally:
        report["counts"] = counts.copy()
        report["artifacts"] = store.artifacts.copy()
        try:
            manifest = store.write("manifest.json", encode(report))
            report["manifestReceipt"] = manifest
        finally:
            store.close()
    return report


def read_input(path):
    with open(path, "rb") as stream:
        raw = stream.read(MAX_JSON + 1)
    return strict_json(raw)


def main(argv):
    try:
        if argv == ["--template"]:
            result = TEMPLATE
        elif len(argv) == 2 and argv[0] == "--validate":
            result = validate(read_input(argv[1]))
            result.pop("packet", None)
            result.update(liveReady=False, authorized=False, networkRequests=0)
        elif len(argv) == 4 and argv[0] == "--collect" and argv[2] == "--output":
            result = collect(read_input(argv[1]), argv[3])
        else:
            raise Stop("usage")
        sys.stdout.buffer.write(encode(result))
        return 1 if result.get("status") in ("STOP", "UNKNOWN") else 0
    except Exception as exc:
        sys.stdout.buffer.write(encode({"status": "STOP", "reason": str(exc) if isinstance(exc, Stop) else "collector_failure", "liveReady": False, "authorized": False}))
        return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
