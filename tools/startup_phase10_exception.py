#!/usr/bin/env python3
"""Prepare the exact AU phase-10 proposal; separately gate its new successor.

This command only reads public files and prints JSON. It never imports/runs the
old operator or changes its journal. The explicit successor API has a distinct
namespace, fresh grant, live flags and original-only recovery; this default
preparation command never signs, contacts RPC or enables that API.
"""
import argparse
import base64
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import stat
import struct
import sys
import time

SCHEMA = "piv1-au-phase10-exception-proposal-v1"
TAIL = "8e5cf7b6aa2be23bd6ed08a0fa652a4f58d4141310043b8db5f13e76e8e58a18"
MANIFEST_SHA = "8124788c7241be115525b3143abed53bd346e318b1ff70297e326ba118e1c761"
AUTH_SHA = "509c2d30cef07cc25bb2a54b34e51101dd0b92938bbf745c821106222a0e2523"
STOP_SHA = "07ac1a10e09a897820d564dcfc570e27aac7ba1877b47c4ee39aff16029b312d"
WIRE_SHA = "9d77da3aeca37e219e968c90d0b89b874c6e56bc5926db0e5e95d2e668b931ab"
SIGNATURE = "24vd6ycXp6wCmtdpWDUZAYeWV6TMJniESK9jrMsdT8BntBG18gBgthmWkHGHP5i7g8uTRM26d44DYQ4Vk7gLKiq3"
ELF_SHA = "e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99"
GENESIS = "4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY"
LOADER = "BPFLoaderUpgradeab1e11111111111111111111111"
PINS = {
    "startup_release_operator.py": "e356a0fb152eb6b2684bc1064ea8e984004b59c6303b73b8c1021c7218022114",
    "startup_manifest.py": "790b7954bc2fcfbdfacf0e5d2a00150aac20deaa826cf6ff0b44eb6d8fccad45",
    "startup_manifest.oracle.json": "a1c17f54f8a5ccd953e8e86c792a784c7ce91884570c7f169618f462f43327e6",
    "startup_signer_backend.cjs": "88138f40cc24e83c46474318f89f06713dbde84da45e1f119a786ceeaadf36e4",
}
PHASE = "startup_0010"
CONTEXT_FLOOR = 456428006  # Retained AU original-only recovery context; never freshness proof.
OFFSET, LENGTH, ELF_BYTES = 8604, 956, 894552
MAX_RECORD = 16 * 1024 * 1024


class Stop(ValueError): pass


def need(ok, reason):
    if not ok: raise Stop(reason)


def encode(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False) + "\n").encode()


def sha(raw): return hashlib.sha256(raw).hexdigest()


def b58(raw):
    alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
    number = int.from_bytes(raw, "big"); text = ""
    while number:
        number, digit = divmod(number, 58); text = alphabet[digit] + text
    return "1" * (len(raw) - len(raw.lstrip(b"\0"))) + text


def read_public(path, maximum, *, dir_fd=None):
    """Bounded regular-file read; never follow a leaf symlink or special file."""
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=dir_fd)
    try:
        before = os.fstat(fd)
        need(stat.S_ISREG(before.st_mode) and before.st_nlink == 1 and before.st_uid == os.getuid() and
             0 < before.st_size <= maximum, "public_file_metadata_or_bound")
        chunks = []; size = 0
        while size < before.st_size:
            chunk = os.read(fd, min(65536, before.st_size - size))
            need(chunk, "public_file_changed"); chunks.append(chunk); size += len(chunk)
        identity = lambda s: (s.st_dev, s.st_ino, s.st_uid, s.st_gid, s.st_mode, s.st_nlink, s.st_size, s.st_mtime_ns, s.st_ctime_ns)
        need(not os.read(fd, 1) and identity(before) == identity(os.fstat(fd)) and
             identity(before) == identity(os.stat(path, dir_fd=dir_fd, follow_symlinks=False)), "public_file_changed")
        return b"".join(chunks)
    finally: os.close(fd)


def canonical(raw):
    value = json.loads(raw)
    need(encode(value) == raw, "public_json_not_canonical")
    return value


def records_at(path):
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        names = {"lock"} | {f"{i:08}.json" for i in range(55)}
        need(set(os.listdir(fd)) == names, "exact_55_record_namespace_required")
        records = []; total = 0
        for i in range(55):
            raw = read_public(f"{i:08}.json", MAX_RECORD, dir_fd=fd)
            total += len(raw); need(total <= 128 * 1024 * 1024, "public_journal_bound")
            records.append(canonical(raw))
        need(set(os.listdir(fd)) == names, "public_journal_changed")
        return records
    finally: os.close(fd)


def proposal(records, auth_raw, stop_raw, oracle):
    """Decode the exact pinned history; no historical behavioral replay/adoption."""
    need(type(records) is list and len(records) == 55, "exact_55_records_required")
    previous = None; identity = records[0]["lockIdentity"]
    for index, record in enumerate(records):
        need(set(record) == {"schema", "sequence", "previousDigest", "lockIdentity", "kind", "data", "digest"} and
             record["schema"] == "piv1-startup-release-operator-v1" and record["sequence"] == str(index) and
             record["previousDigest"] == previous and record["lockIdentity"] == identity, "original_record_link")
        need(sha(encode({k:v for k,v in record.items() if k != "digest"})) == record["digest"], "original_record_digest")
        previous = record["digest"]
    need(previous == TAIL, "exact_original_unknown_tail_required")
    need(sha(auth_raw) == AUTH_SHA and sha(stop_raw) == STOP_SHA, "exact_authorization_and_stop_required")
    auth, stop = canonical(auth_raw), canonical(stop_raw)
    first = records[0]["data"]; manifest = first["manifest"]
    need(records[0]["kind"] == "CREATE" and sha(encode(manifest)) == MANIFEST_SHA, "exact_original_manifest_required")
    need(auth["manifestSha256"] == MANIFEST_SHA and auth["baseParentTail"] == manifest["parentTail"] and
         auth["operatorSourceSha256"] == PINS["startup_release_operator.py"] and auth["expiresUnixTimestamp"] == "1792917973",
         "original_authorization_lineage")
    need(stop["phaseIndex"] == 10 and stop["reason"] == "post_intent_lifetime_margin" and
         stop["originalAuthorizationSha256"] == AUTH_SHA, "original_stop_binding")
    need(records[51]["kind"] == "PREPARE_SIGN_ATTEMPT" and records[52]["kind"] == "IMPORT_SEND_ATTEMPT_INTENT" and
         records[54]["kind"] == "UNKNOWN" and records[54]["data"]["id"] == PHASE, "original_unknown_required")
    prepared = records[51]["data"]["entries"][0]
    signed = records[52]["data"]["import"]["entries"][0]
    intent = records[52]["data"]["intent"]
    need(prepared["id"] == signed["id"] == intent["id"] == PHASE and
         intent["authorizationSha256"] == AUTH_SHA and intent["feeLamports"] == prepared["feeQuoteLamports"] == "5000",
         "original_phase_fee_binding")
    wire = base64.b64decode(signed["wireBase64"], validate=True)
    need(sha(wire) == signed["wireSha256"] == intent["wireSha256"] == WIRE_SHA and wire[0] == 1 and
         wire[1:65].hex() == signed["signatureHex"] == intent["signatureHex"] and
         b58(wire[1:65]) == signed["signature"] == intent["signature"] == SIGNATURE, "exact_original_wire_required")
    image = base64.b64decode(manifest["startupProfile"]["elfBase64"], validate=True)
    need(len(image) == ELF_BYTES and sha(image) == ELF_SHA, "exact_selected_artifact_required")
    ref = oracle["writeTemplate"]; roles = oracle["roles"]
    prefix = base64.b64decode(ref["prefixBase64"], validate=True)
    # Exact selected legacy template: four static keys, zero recent blockhash,
    # existing ComputeBudget instructions, and one loader Write at offset8604.
    need(prefix[:4] == bytes([1, 0, 2, 4]) and prefix[132:164] == bytes(32), "inert_source_template_required")
    data = struct.pack("<IIQ", 1, OFFSET, LENGTH) + image[OFFSET:OFFSET+LENGTH]
    message = prefix + bytes([0xcc, 0x07]) + data  # shortvec(972)
    need(wire[65:197] + bytes(32) + wire[229:] == message and
         b58(wire[197:229]) == prepared["lifetime"]["recentBlockhash"] and
         prepared["lifetime"]["lastValidBlockHeight"] == "407927460", "original_write_semantics")
    reviewed = {**{k:v for k,v in ref.items() if k != "prefixBase64"},
                "messageBase64": base64.b64encode(message).decode(), "messageSha256": sha(message)}
    finalized = [r["data"] for r in records if r["kind"] == "FINALIZED"]
    need([d["id"] for d in finalized] == [f"startup_{i:04}" for i in range(10)] and
         sum(int(d["feeLamports"]) for d in finalized) == 55000, "original_finalized_fee_ledger")
    packets = [r["data"]["import"]["entries"][0] for r in records if r["kind"] == "IMPORT_SEND_ATTEMPT_INTENT"]
    need([p["id"] for p in packets] == [f"startup_{i:04}" for i in range(11)], "original_packet_inventory")
    header = struct.pack("<IB", 1, 1) + prefix[4:36]
    branches = []
    for written in (OFFSET, OFFSET + LENGTH):
        raw = header + image[:written] + bytes(ELF_BYTES-written)
        branches.append({"writtenPrefixBytes": written, "accountDataBytes": len(raw), "accountDataSha256": sha(raw),
                         "owner": LOADER, "executable": False, "lamports": oracle["rents"]["894589"]})
    return {
        "schema": SCHEMA, "status": "OFFLINE_PREPARATION_ONLY_NO_LIVE_TRIGGER", "liveReady": False,
        "cluster": "testnet", "genesisHash": GENESIS, "artifact": {"bytes": ELF_BYTES, "sha256": ELF_SHA},
        "sourcePins": dict(PINS), "historicalBehavioralReplayPerformed": False,
        "lineage": {"originalRecordCount": 55, "originalTail": TAIL, "originalCreateDigest": records[0]["digest"],
            "minimumSuccessorContextSlot": str(CONTEXT_FLOOR),
            "originalParentTail": manifest["parentTail"], "originalManifestSha256": MANIFEST_SHA,
            "originalAuthorizationSha256": AUTH_SHA, "originalAuthorizationExpiryUnixTimestamp": auth["expiresUnixTimestamp"],
            "originalStopReceiptSha256": STOP_SHA, "originalIntentRecordDigest": records[52]["digest"],
            "originalStatus": "UNKNOWN_HOLD_PRESERVED", "originalOutstandingPackets": [signed],
            "originalSignedPacketInventory": [{k:p[k] for k in ("id", "wireSha256", "signature", "signatureHex")} for p in packets],
            "originalLifetime": prepared["lifetime"], "originalTerminalInserted": False,
            "historyOrNoninclusionProved": False,
            "localNondispatchEvidenceScope": "Pinned STOP and original records only; historical source review reused; raw send transcript not re-audited."},
        "singleWrite": {"id": PHASE, "kind": "UPLOAD_WRITE", "program": LOADER,
            "buffer": roles["loaderBuffer"], "authority": roles["g0"], "feePayer": roles["g0"],
            "offsetBytes": OFFSET, "lengthBytes": LENGTH, "sliceSha256": sha(image[OFFSET:OFFSET+LENGTH]),
            "reviewed": reviewed, "unsignedWireBase64": base64.b64encode(bytes([1])+bytes(64)+message).decode(),
            "signatureCount": 1, "recentBlockhash": "11111111111111111111111111111111",
            "lifetime": None, "feeQuoteLamports": None, "minimumRemainingBlockHeights": "40"},
        "fullBufferPredicates": {"status": "SOURCE_DERIVED_NOT_OBSERVED", "buffer": roles["loaderBuffer"],
            "authority": roles["g0"], "admissiblePrestateBranches": branches, "requiredPoststate": branches[1],
            "programAbsent": roles["pivProgram"], "programDataAbsent": roles["programData"],
            "branchDoesNotProveOriginalHistoryOrFee": True},
        "feePolicy": {"status": "PROPOSAL_ONLY_NOT_AUTHORIZED_OR_QUOTED", "oldActualLamports": "55000",
            "oldReservedLamports": "60000", "oldPossibleAdditionalLamports": "5000", "oldReservationReleased": False,
            "newSingleWriteFeeCapLamports": "5000", "combinedReservationAfterOneNewIntentLamports": "65000",
            "conditionalWholeSequenceCapIfOtherQuotesUnchangedLamports": "4820000", "h2DebitLamports": "0",
            "newAttemptsAllowedByThisProposal": 0},
        "remainingIntegrationBlocker": "SEPARATE_SOURCE_REVIEW_FRESH_COMPLETED_LIVE_PACKAGE_AND_NEW_LIVE_GRANT_REQUIRED: the isolated one-Write successor is implemented separately; this proposal remains inert and cannot enable downstream progression.",
        "requiredLiveGuards": {"status": "OPEN_NOT_SATISFIED_BY_THIS_PROPOSAL",
            "sourceAndLineage": "Reviewed exact new execution source/package and immutable original evidence; never rebind old source or adopt old journal as a successor.",
            "history": "Fresh original-signature history/packet inventory and original blockheight >407927460 plus invalid original hash; expiry/null/current bytes never prove historical noninclusion or release5000.",
            "state": "Fresh coherent complete894589-byte Buffer matching one full predicate, exact header/G0/owner/rent; Program+ProgramData absent; exact artifact, actual G0/H2 balances and funding/debit ceilings.",
            "clockAndRuntime": "Fresh coherent Clock, exact Testnet genesis/runtime/resources/rents and monotone contexts across durability/callback boundaries.",
            "feeAndLifetime": "Fresh exact message fee <=5000, separate old5000 liability retained, fresh original new-packet lifetime >=40 at every existing boundary; no priority fee, topup or H2 debit.",
            "authority": "Separate completed reviewed exact live package and new explicit founder grant; old authorization expiry is separate from original blockhash expiry.",
            "durabilityAndRecovery": "Durable at-most-once new intent; one send maxRetries0/skipPreflightfalse/finalized; uncertain new result HOLD with new original-signature-only recovery; no resend/replacement/adoption/automatic resume.",
            "scope": "This one Write only; no dependent progression/bootstrap/authority/funding/Mainnet or main publication."},
    }


def prepare(record_directory, authorization_file, stop_file):
    here = Path(__file__).parent
    sources = {name: read_public(here/name, MAX_RECORD) for name in PINS}
    need(all(sha(raw) == PINS[name] for name,raw in sources.items()), "unchanged_original_source_pins_required")
    result = proposal(records_at(record_directory), read_public(authorization_file, 32768), read_public(stop_file, 32768),
                      json.loads(sources["startup_manifest.oracle.json"]))
    result["preparerSourceSha256"] = sha(read_public(Path(__file__), MAX_RECORD))
    # Recheck actual original sources after the complete bounded read/derivation.
    need(all(read_public(here/name, MAX_RECORD) == raw for name,raw in sources.items()), "original_source_changed")
    return result


# The successor never calls the historical operator's operate/replay functions.
# Its immutable primitives run at their real source path; the fresh outer grant
# names both sources and a separate, fresh nested backend compatibility receipt.
CONTROLLER_SCHEMA = "piv1-au-phase10-controller-v1"
AUTH_SCHEMA = "piv1-au-phase10-authorization-v1"
BRIDGE = "startup_phase10_signer_bridge.cjs"


def package(record_directory, authorization_file, stop_file):
    evidence = [str(Path(p).absolute()) for p in (record_directory, authorization_file, stop_file)]
    return {"schema": "piv1-au-phase10-package-v1", "evidence": evidence,
            "proposal": prepare(*evidence), "bridgeSourceSha256": sha(read_public(Path(__file__).with_name(BRIDGE), MAX_RECORD))}


def checked_package(value):
    need(type(value) is dict and set(value) == {"schema", "evidence", "proposal", "bridgeSourceSha256"} and
         value["schema"] == "piv1-au-phase10-package-v1" and type(value["evidence"]) is list and len(value["evidence"]) == 3,
         "successor_package_shape")
    need(package(*value["evidence"]) == value, "successor_package_changed")
    return value["proposal"]


def successor_authorization(value, raw, pin, journal, *, expired=False):
    p = checked_package(value)
    need(type(raw) is bytes and sha(raw) == pin, "successor_authorization_hash")
    a = canonical(raw)
    need(set(a) == {"schema", "scope", "journalPath", "packageSha256", "originalTail", "sourcePins", "expiresUnixTimestamp",
                    "signerProfileSha256", "feeBudgetNamespace", "backendAuthorizationRawBase64", "backendAuthorizationSha256", "writeSliceSha256"},
         "successor_authorization_fields")
    pins = {**PINS, Path(__file__).name: p["preparerSourceSha256"], BRIDGE: value["bridgeSourceSha256"]}
    path = Path(journal)
    need(str(path) == os.path.abspath(journal) and not path.is_symlink() and
         path.name == "au-phase10-" + TAIL + ".exception" and
         Path(__file__).resolve().parent.parent not in path.parents, "exact_successor_namespace")
    need(a["schema"] == AUTH_SCHEMA and a["scope"] == "EXACT_SINGLE_AU_PHASE10_EXCEPTION" and
         a["journalPath"] == str(path) and a["packageSha256"] == sha(encode(value)) and a["originalTail"] == TAIL and
         a["sourcePins"] == pins and a["writeSliceSha256"] == p["singleWrite"]["sliceSha256"] and
         a["feeBudgetNamespace"] == "piv1-testnet-startup-release-" + a["packageSha256"], "successor_authorization_binding")
    def decimal(x):
        need(type(x) is str and x.isascii() and x.isdecimal() and str(int(x)) == x and 0 <= int(x) < 2**64, "successor_u64")
        return int(x)
    expiry = decimal(a["expiresUnixTimestamp"])
    need(expired or expiry > int(time.time()), "successor_authorization_expired")
    need(type(a["signerProfileSha256"]) is str and len(a["signerProfileSha256"]) == 64 and
         all(x in "0123456789abcdef" for x in a["signerProfileSha256"]), "successor_profile_hash")
    inner_raw = base64.b64decode(a["backendAuthorizationRawBase64"], validate=True)
    need(base64.b64encode(inner_raw).decode() == a["backendAuthorizationRawBase64"] and sha(inner_raw) == a["backendAuthorizationSha256"] != AUTH_SHA,
         "fresh_nested_authorization_required")
    inner = canonical(inner_raw)
    need(inner == {"schema": "piv1-launcher-release-authorization-v2", "manifestSha256": a["packageSha256"],
                  "baseParentTail": {"sequence": "54", "digest": TAIL}, "operatorSourceSha256": PINS["startup_release_operator.py"],
                  "expiresUnixTimestamp": a["expiresUnixTimestamp"], "scope": "EXACT_BOUNDED_TESTNET_RELEASE",
                  "feeBudgetNamespace": a["feeBudgetNamespace"], "signerBackendSha256": PINS["startup_signer_backend.cjs"],
                  "signerProfileSha256": a["signerProfileSha256"]}, "nested_compatibility_contract")
    return a


def successor_core():
    path = Path(__file__).with_name("startup_release_operator.py")
    raw = read_public(path, MAX_RECORD)
    need(sha(raw) == PINS[path.name], "original_core_source_pin")
    spec = importlib.util.spec_from_file_location("phase10_immutable_primitives", path)
    core = importlib.util.module_from_spec(spec); exec(compile(raw, str(path), "exec", dont_inherit=True), core.__dict__)
    # Isolated module only. No old record or old source identity is rewritten.
    core.SCHEMA = CONTROLLER_SCHEMA; core.LOCK = b"piv1-au-phase10-exception-lock-v1\n"
    core.MAX_RECORDS = 64  # One new Write, explicit original-only recovery; no unbounded retries.
    core.journal_path = lambda base: Path(base)
    return core


def recheck_live_grant(value, a, raw, pin):
    """Final source/expiry barrier after the last raw lifetime response fsync.

    Full immutable history/package replay precedes this small barrier; it must
    not consume the freshly observed margin again through crypto callbacks.
    """
    need(sha(raw) == pin and encode(a) == raw and a["packageSha256"] == sha(encode(value)) and
         int(a["expiresUnixTimestamp"]) > int(time.time()), "successor_final_authorization")
    here = Path(__file__).parent
    need(all(sha(read_public(here/name, MAX_RECORD)) == digest for name, digest in a["sourcePins"].items()), "successor_final_source_pins")


def physical_balances(oracle, debit, new_fee=0):
    need(type(debit) is int and debit in (55000, 60000) and type(new_fee) is int and 0 <= new_fee <= 5000,
         "exact_physical_debit_branch")
    r = oracle["roles"]
    return int(oracle["baseline"][r["g0"]]["lamports"]) - debit - new_fee, int(oracle["baseline"][r["h2"]]["lamports"]) - int(oracle["rents"]["894589"])


def downstream_balance_adjustment(debit, new_fee):
    """Exact adjustment to unchanged actor_balances(11+), never a tolerance.

    The canonical schedule already includes phase10's 5,000-lamport fee.
    This physical debit is not a conclusion about the old UNKNOWN history.
    No later phase is enabled by this one-Write controller.
    """
    need(debit in (55000, 60000) and new_fee == 5000, "downstream_exact_debit")
    return debit + new_fee - 60000


def verify_snapshot(core, c, p, accounts, context, floor, *, debit=None, post=False, pre_clock=None):
    oracle = core.startup_manifest().oracle(); roles = oracle["roles"]
    keys = core.startup_manifest().required_keys({"kind": "UPLOAD_WRITE"})
    need(set(accounts) == set(keys), "successor_complete_account_inventory")
    clock = core.state_rules().clock(c, accounts[roles["clock"]], context, floor)
    if pre_clock:
        need(int(clock["unixTimestamp"]) >= int(pre_clock["unixTimestamp"]) and int(clock["slot"]) >= int(pre_clock["slot"]), "successor_clock_regression")
    candidates = (55000, 60000) if debit is None else (debit,)
    observed_g0 = int(accounts[roles["g0"]]["lamports"])
    branches = [d for d in candidates if physical_balances(oracle, d, 5000 if post else 0)[0] == observed_g0]
    need(len(branches) == 1, "successor_exact_g0_branch")
    debit = branches[0]; g, h = physical_balances(oracle, debit, 5000 if post else 0)
    predicates = p["fullBufferPredicates"]
    buffers = [predicates["requiredPoststate"]] if post else predicates["admissiblePrestateBranches"]
    for key in keys:
        if key == roles["clock"]: continue
        digest = core.state_digest(c, accounts[key])
        if key == roles["loaderBuffer"]:
            need(any(digest == {"exists": True, "owner": b["owner"], "executable": False, "lamports": b["lamports"],
                                "dataBytes": b["accountDataBytes"], "dataSha256": b["accountDataSha256"]} for b in buffers), "successor_full_buffer_predicate")
        else:
            expected = dict(oracle["baseline"][key])
            if key in (roles["g0"], roles["h2"]): expected["lamports"] = str(g if key == roles["g0"] else h)
            need(digest == expected, "successor_exact_account:" + key)
    need(g >= 5000 and h >= 0, "successor_funding_floor")
    return {"physicalOldDebitLamports": str(debit), "clock": clock, "contextSlot": str(context)}


def guard_calls(core, p, prepared, floor):
    option = {"commitment": "finalized", "minContextSlot": floor}
    keys = core.startup_manifest().required_keys({"kind": "UPLOAD_WRITE"})
    return [("getGenesisHash", []), ("getVersion", []), ("getBlockHeight", [option]),
            ("isBlockhashValid", [p["lineage"]["originalLifetime"]["recentBlockhash"], option]),
            ("getSignatureStatuses", [[SIGNATURE], {"searchTransactionHistory": True}]),
            ("getTransaction", [SIGNATURE, {"encoding": "base64", "commitment": "finalized", "maxSupportedTransactionVersion": 0}]),
            ("isBlockhashValid", [prepared["lifetime"]["recentBlockhash"], option]),
            ("getFeeForMessage", [prepared["reviewed"]["messageBase64"], option]),
            ("getMultipleAccounts", [keys, {"encoding": "base64", **option}]),
            ("getMinimumBalanceForRentExemption", [894589, {"commitment": "finalized"}])]


def verify_guard(core, c, p, prepared, floor, results, *, debit=None, prior=None):
    need(len(results) == 10, "successor_guard_results")
    genesis, version, height, old_valid, status, tx, valid, fee, snapshot, rent = results
    oracle = core.startup_manifest().oracle()
    need(genesis == GENESIS and version == oracle["version"], "successor_genesis_runtime")
    height = c.u64(height)
    need(height > 407927460 and type(old_valid) is dict and old_valid.get("value") is False and
         c.u64(old_valid["context"]["slot"]) >= floor, "successor_original_expiry")
    need(type(status) is dict and status.get("value") == [None] and c.u64(status["context"]["slot"]) >= floor and tx is None,
         "successor_original_history_requires_review")
    life = prepared["lifetime"]; last = int(life["lastValidBlockHeight"])
    need(int(life["currentBlockHeight"]) <= height <= last - 40 and (prior is None or height >= int(prior["height"])), "successor_original_new_lifetime_margin")
    need(type(valid) is dict and valid.get("value") is True and c.u64(valid["context"]["slot"]) >= floor, "successor_new_blockhash_invalid")
    need(type(fee) is dict and c.u64(fee["context"]["slot"]) >= floor and str(c.u64(fee["value"])) == prepared["feeQuoteLamports"] == "5000",
         "successor_fresh_exact_fee")
    need(str(c.u64(rent)) == oracle["rents"]["894589"], "successor_fresh_rent")
    keys = core.startup_manifest().required_keys({"kind": "UPLOAD_WRITE"})
    context = c.u64(snapshot["context"]["slot"])
    need(context >= max(floor, c.u64(valid["context"]["slot"]), c.u64(fee["context"]["slot"]), c.u64(status["context"]["slot"]), c.u64(old_valid["context"]["slot"])) and
         type(snapshot.get("value")) is list and len(snapshot["value"]) == len(keys), "successor_coherent_snapshot")
    accounts = {k: c.normalize_account(v) for k, v in zip(keys, snapshot["value"])}
    result = verify_snapshot(core, c, p, accounts, context, floor, debit=debit, pre_clock=prior["clock"] if prior else None)
    return {**result, "height": str(height), "oldUnknownPreserved": True, "oldReservedLamports": "60000", "oldPossibleFeeLamports": "5000"}


def retained_rpc(core, c, directory, calls):
    """Replay the exact saved queries/raw responses, never a derived snapshot."""
    results = []
    for start in range(0, len(calls), 5):
        prefix = f"rpc-{start//5+1:02}"
        queries = [{"jsonrpc": "2.0", "id": i+1, "method": m, "params": v} for i, (m, v) in enumerate(calls[start:start+5])]
        raw = {suffix: read_public(Path(directory)/(prefix+suffix), MAX_RECORD) for suffix in (".request.json", ".http.json", ".response.bin")}
        need(raw[".request.json"] == encode(queries) and canonical(raw[".http.json"])["status"] == 200, "successor_raw_request_http")
        replies = c.strict_json(raw[".response.bin"])
        need(type(replies) is list and len(replies) == len(queries), "successor_raw_count")
        by = {}
        for reply in replies:
            need(type(reply) is dict and reply.get("jsonrpc") == "2.0" and type(reply.get("id")) is int and
                 1 <= reply["id"] <= len(queries) and reply["id"] not in by and "result" in reply and "error" not in reply, "successor_raw_result_error")
            by[reply["id"]] = reply["result"]
        results += [by[i+1] for i in range(len(queries))]
    return results


def rpc_receipt(directory, count):
    names = [f"rpc-{i:02}{suffix}" for i in range(1, count+1) for suffix in (".request.json", ".started.json", ".http.json", ".response.bin")]
    return {"path": str(directory), "files": {name: sha(read_public(Path(directory)/name, MAX_RECORD)) for name in names}}


def verify_rpc_receipt(receipt, directory, count):
    need(receipt == rpc_receipt(directory, count), "successor_raw_receipt_changed")


def signing_request(p, a, prepared, predecessor):
    body = {"schema": "piv1-au-phase10-signing-request-v1", "cluster": "testnet", "genesisHash": GENESIS,
            "operatorTail": predecessor, "operatorSourceSha256": p["preparerSourceSha256"], "authorizationSha256": sha(encode(a)),
            "baseParentPath": a["journalPath"], "baseParentTail": {"sequence": "54", "digest": TAIL},
            "manifestSha256": a["packageSha256"], "sealSha256": None,
            "packets": [{"id": PHASE, "preparedRecordDigest": predecessor, "preparedSha256": sha(encode(prepared)),
                         "unsignedWireSha256": sha(base64.b64decode(prepared["wireBase64"], validate=True)),
                         "messageSha256": prepared["reviewed"]["messageSha256"], "requiredSigners": prepared["reviewed"]["requiredSigners"]}]}
    return {**body, "requestId": sha(encode(body))}


def signed_response(core, c, j, p, a, attempt, response):
    request = attempt["request"]; backend_request = {**request, "schema": "piv1-startup-signing-request-v1",
        "operatorSourceSha256": PINS["startup_release_operator.py"], "authorizationSha256": a["backendAuthorizationSha256"]}
    del backend_request["requestId"]; backend_request["requestId"] = sha(encode(backend_request))
    need(set(response) == {"schema", "requestId", "backendRequest", "backendResponse"} and
         response["schema"] == "piv1-au-phase10-signing-response-v1" and response["requestId"] == request["requestId"] and
         response["backendRequest"] == backend_request, "successor_nested_response_binding")
    backend = response["backendResponse"]
    need(set(backend) == {"schema", "requestId", "packets"} and backend["schema"] == "piv1-startup-signing-response-v1" and
         backend["requestId"] == backend_request["requestId"] and type(backend["packets"]) is list and len(backend["packets"]) == 1,
         "successor_backend_response_binding")
    signed = core.signed_entry(c, {"prepared": {PHASE: attempt["prepared"]}, "signed": {}, "intents": {}}, backend["packets"][0])
    old = p["lineage"]["originalSignedPacketInventory"]
    need(not any(signed["wireSha256"] == x["wireSha256"] or signed["signatureHex"] == x["signatureHex"] for x in old), "successor_original_packet_replay")
    phase = {"id": PHASE, "feeCapLamports": "5000"}
    frozen = {"packageSha256": a["packageSha256"], "payloadSha256": p["artifact"]["sha256"], "anchorUnixTimestamp": None, "transactionIndex": "0"}
    core.check_signed(j, frozen, phase, attempt["prepared"], signed)
    return signed


def successor_replay(core, c, j, records, journal):
    need(records and records[0]["kind"] == "CREATE", "successor_create_required")
    first = records[0]["data"]
    need(set(first) == {"package", "authorization", "authorizationSha256"}, "successor_create_fields")
    value = first["package"]; p = checked_package(value)
    raw = encode(first["authorization"]); pin = first["authorizationSha256"]
    a = successor_authorization(value, raw, pin, journal, expired=True)
    root = Path(journal).with_name(Path(journal).name + ".evidence")
    state = {"stage": "CREATED", "package": value, "proposal": p, "authorization": a, "authorizationRaw": raw,
             "authorizationSha256": pin, "attempt": None, "intent": None, "floor": CONTEXT_FLOOR, "recoveries": 0}
    for record in records[1:]:
        kind, d = record["kind"], record["data"]
        sequence = int(record["sequence"])
        if kind == "SIGN_ATTEMPT":
            need(state["stage"] == "CREATED" and set(d) == {"prepared", "quote", "request", "guard", "guardResult", "mode"}, "successor_sign_once")
            prepared = d["prepared"]
            need(set(prepared) == {"id", "wireBase64", "reviewed", "feeQuoteLamports", "lifetime"} and prepared["id"] == PHASE and
                 prepared["feeQuoteLamports"] == "5000" and prepared["lifetime"]["minimumRemainingBlockHeights"] == "40", "successor_prepared_shape")
            quote_dir = root/f"{sequence:08}-quote"; verify_rpc_receipt(d["quote"], quote_dir, 1)
            initial = retained_rpc(core, c, quote_dir, [("getGenesisHash", []), ("getLatestBlockhash", [{"commitment": "finalized", "minContextSlot": CONTEXT_FLOOR}])])
            need(initial[0] == GENESIS, "successor_quote_genesis")
            quoted = core.latest_quote(c, {"floor": CONTEXT_FLOOR}, initial[1]); life = prepared["lifetime"]
            need(life["recentBlockhash"] == quoted["recentBlockhash"] and life["lastValidBlockHeight"] == quoted["lastValidBlockHeight"], "successor_original_quote_lifetime")
            unsigned, review = core.rebind_unsigned(c, p["singleWrite"]["unsignedWireBase64"], p["singleWrite"]["reviewed"], life["recentBlockhash"])
            need(unsigned == prepared["wireBase64"] and review == prepared["reviewed"] and
                 d["request"] == signing_request(p, a, prepared, record["previousDigest"]) and
                 d["mode"] in ("OFFLINE_INJECTED_TEST", "FIXED_ORIGIN_TLS"), "successor_exact_signing_request")
            guard_dir = root/f"{sequence:08}-presign"; verify_rpc_receipt(d["guard"], guard_dir, 2)
            result = verify_guard(core, c, p, prepared, int(quoted["contextSlot"]), retained_rpc(core, c, guard_dir, guard_calls(core, p, prepared, int(quoted["contextSlot"]))))
            need(result == d["guardResult"], "successor_presign_guard_binding")
            state.update(stage="SIGN_ATTEMPT", attempt=d, floor=int(result["contextSlot"]))
        elif kind == "INTENT":
            need(state["stage"] == "SIGN_ATTEMPT" and set(d) == {"responseSha256", "signed", "guard", "guardResult", "authorizationSha256", "mode"} and
                 d["authorizationSha256"] == pin and d["mode"] == state["attempt"]["mode"], "successor_intent_once")
            response_raw = read_public(root/"00000001-response"/"signer-response.json", 262144)
            need(sha(response_raw) == d["responseSha256"], "successor_retained_response_changed")
            signed = signed_response(core, c, j, p, a, state["attempt"], canonical(response_raw))
            need(signed == d["signed"], "successor_signed_intent_binding")
            before = state["attempt"]["guardResult"]; guard_dir = root/f"{sequence:08}-postsign"
            verify_rpc_receipt(d["guard"], guard_dir, 2)
            result = verify_guard(core, c, p, state["attempt"]["prepared"], state["floor"], retained_rpc(core, c, guard_dir, guard_calls(core, p, state["attempt"]["prepared"], state["floor"])), debit=int(before["physicalOldDebitLamports"]), prior=before)
            need(result == d["guardResult"], "successor_postsign_guard_binding")
            state.update(stage="INTENT", intent=d, floor=int(result["contextSlot"]))
        elif kind == "ORIGINAL_RESPONSE_HOLD":
            need(state["stage"] == "SIGN_ATTEMPT" and set(d) == {"responseSha256", "signed"}, "successor_own_original_response_only")
            response_raw = read_public(root/"00000001-response"/"signer-response.json", 262144)
            need(sha(response_raw) == d["responseSha256"] and
                 signed_response(core, c, j, p, a, state["attempt"], canonical(response_raw)) == d["signed"], "successor_own_response_binding")
            state.update(stage="ORIGINAL_RESPONSE_HOLD", intent={**d, "guardResult": state["attempt"]["guardResult"]})
        elif kind in ("SUBMITTED", "UNKNOWN"):
            need(state["stage"] in ("INTENT", "SUBMITTED", "UNKNOWN", "RECOVER_ATTEMPT") and set(d) == {"reason"}, "successor_unknown_transition")
            need(kind != "SUBMITTED" or state["stage"] == "INTENT", "successor_no_second_send")
            state["stage"] = kind
        elif kind == "RECOVER_ATTEMPT":
            need(state["intent"] is not None and state["stage"] in ("INTENT", "SUBMITTED", "UNKNOWN", "RECOVER_ATTEMPT", "ORIGINAL_RESPONSE_HOLD") and d == {"id": PHASE}, "successor_original_only_recovery")
            state.update(stage="RECOVER_ATTEMPT", recoveries=state["recoveries"]+1)
        elif kind in ("FINALIZED", "FAILED"):
            need(state["stage"] == "RECOVER_ATTEMPT" and set(d) == {"manifestSha256"}, "successor_terminal_original_proof_only")
            directory = root/f"{sequence-1:08}-recovery"/"collection"
            report = retained_recovery(core, c, j, state, directory, d["manifestSha256"])
            need(report["observation"]["status"]["confirmationStatus"] == "finalized" and
                 (report["observation"]["status"]["err"] is None) == (kind == "FINALIZED"), "successor_terminal_result")
            state["stage"] = kind
        else: raise Stop("successor_record_kind")
    return state


def recovery_request(core, state):
    prepared = state["attempt"]["prepared"]; signed = state["intent"]["signed"]
    return {"schema": core.helpers()[1].SCHEMA, "wireBase64": signed["wireBase64"], "expectedWireSha256": signed["wireSha256"],
            "expectedMessageSha256": prepared["reviewed"]["messageSha256"], "signature": signed["signature"],
            "expectedFeeLamports": "5000", "accountKeys": core.startup_manifest().required_keys({"kind": "UPLOAD_WRITE"}),
            "minimumContextSlot": str(state["floor"]), "includeExpiry": False}


def retained_recovery(core, c, j, state, directory, digest):
    raw = read_public(directory/"manifest.json", MAX_RECORD); need(sha(raw) == digest, "successor_recovery_manifest_hash")
    report = canonical(raw); expected_input = recovery_request(core, state)
    need(read_public(directory/"input.json", 32768) == encode(expected_input), "successor_recovery_original_input")
    names = set()
    for artifact in report["artifacts"]:
        name = artifact["name"]
        need(type(name) is str and "/" not in name and name not in names and name != "manifest.json", "successor_recovery_artifact_name")
        data = read_public(directory/name, MAX_RECORD); names.add(name)
        need(len(data) == artifact["bytes"] and sha(data) == artifact["sha256"], "successor_recovery_raw_hash")
    def query(index, queries):
        prefix = f"http-{index:02}"
        required = {prefix+x for x in (".request.json", ".started.json", ".response.bin", ".http.json")}
        need(required <= names and read_public(directory/(prefix+".request.json"), 32768) == encode(queries) and
             canonical(read_public(directory/(prefix+".http.json"), 32768))["status"] == 200, "successor_recovery_raw_request")
        replies = c.strict_json(read_public(directory/(prefix+".response.bin"), MAX_RECORD)); by = {}
        need(type(replies) is list and len(replies) == len(queries), "successor_recovery_response_count")
        for r in replies:
            need(type(r) is dict and r.get("jsonrpc") == "2.0" and type(r.get("id")) is int and r["id"] in [q["id"] for q in queries] and
                 r["id"] not in by and "result" in r and "error" not in r, "successor_recovery_result_error")
            by[r["id"]] = r["result"]
        return [by[q["id"]] for q in queries]
    need(query(1, [{"jsonrpc": "2.0", "id": 1, "method": "getGenesisHash", "params": []}]) == [GENESIS], "successor_recovery_genesis")
    signed = state["intent"]["signed"]
    status, tx = query(2, [{"jsonrpc": "2.0", "id": 2, "method": "getSignatureStatuses", "params": [[signed["signature"]], {"searchTransactionHistory": True}]},
                           {"jsonrpc": "2.0", "id": 3, "method": "getTransaction", "params": [signed["signature"], {"encoding": "base64", "commitment": "finalized", "maxSupportedTransactionVersion": 0}]}])
    obs = report["observation"]
    need(type(status) is dict and type(status.get("value")) is list and len(status["value"]) == 1 and type(tx) is dict and
         type(status["value"][0]) is dict and obs is not None, "successor_recovery_complete_history")
    s = status["value"][0]; slot = c.u64(s["slot"])
    need(slot >= state["floor"] and c.u64(status["context"]["slot"]) >= slot and c.u64(tx["slot"]) == slot and
         s["confirmationStatus"] == "finalized" and "err" in s and tx["meta"]["err"] == s["err"] and
         tx["transaction"] == [signed["wireBase64"], "base64"] and tx["version"] == "legacy" and str(c.u64(tx["meta"]["fee"])) == "5000",
         "successor_recovery_exact_original_transaction")
    expected_obs = {"status": {"signatureHex": signed["signatureHex"], "slot": str(slot), "confirmationStatus": "finalized", "err": s["err"]},
                    "transaction": {"slot": str(slot), "wireBase64": signed["wireBase64"], "feeLamports": "5000", "err": s["err"]}, "poststate": None}
    if s["err"] is None:
        minimum = max(state["floor"], slot)
        snapshot = query(3, [{"jsonrpc": "2.0", "id": 4, "method": "getMultipleAccounts", "params": [expected_input["accountKeys"], {"encoding": "base64", "commitment": "finalized", "minContextSlot": minimum}]}])[0]
        context = c.u64(snapshot["context"]["slot"])
        need(type(snapshot.get("value")) is list and len(snapshot["value"]) == len(expected_input["accountKeys"]), "successor_recovery_full_snapshot")
        accounts = {k: c.normalize_account(v) for k, v in zip(expected_input["accountKeys"], snapshot["value"])}
        post = verify_snapshot(core, c, state["proposal"], accounts, context, minimum,
                               debit=int(state["intent"]["guardResult"]["physicalOldDebitLamports"]), post=True)
        core.state_rules().window(state["intent"]["guardResult"]["clock"], post["clock"], slot)
        expected_obs["poststate"] = {"contextSlot": str(context), "accounts": accounts}
    need(obs == expected_obs and read_public(directory/"observation.json", MAX_RECORD) == encode(obs), "successor_recovery_observation_raw_binding")
    a = state["authorization"]; p = state["proposal"]
    core.check_signed(j, {"packageSha256": a["packageSha256"], "payloadSha256": p["artifact"]["sha256"], "anchorUnixTimestamp": None, "transactionIndex": "0"},
                      {"id": PHASE, "feeCapLamports": "5000"}, state["attempt"]["prepared"], signed, obs)
    return report


def successor(action, value, journal, *, authorization_raw, authorization_sha256, expected_tail=None,
              live=False, live_sign=False, signer_profile_path=None, _transport=None, _signer=None):
    """Execute only a separately authorized new one-Write namespace.

    There is no automatic recovery, retry, second Write or downstream action.
    Test seams must be supplied together; they can never enter a live journal.
    """
    need(action in ("CREATE", "INSPECT", "CAPTURE_SIGN_SEND", "RECOVER"), "successor_action")
    need((_transport is None) == (_signer is None), "successor_test_mode_pair")
    need(action in ("CREATE", "INSPECT") or live is True, "successor_live_transport_not_enabled")
    need(action != "CAPTURE_SIGN_SEND" or live_sign is True, "successor_live_sign_not_enabled")
    a = successor_authorization(value, authorization_raw, authorization_sha256, journal, expired=action in ("INSPECT", "RECOVER"))
    core = successor_core(); _, c, _, j = core.helpers()
    core.replay = lambda journal_api, records: successor_replay(core, c, journal_api, records, journal)
    core.signer_backend_source = lambda: Path(__file__).with_name(BRIDGE)
    op = core.Journal(j, journal, create=action == "CREATE")
    root = Path(journal).with_name(Path(journal).name + ".evidence")
    def fresh():
        op.stable()
        need(set(os.listdir(op.fd)) == {"lock"} | {f"{i:08}.json" for i in range(len(op.chain))} and
             all(j.read_file(op.fd, f"{i:08}.json", MAX_RECORD) == encode(r) for i, r in enumerate(op.chain)), "successor_durable_chain_changed")
        return core.replay(j, op.chain)
    def store(label, sequence=None):
        core.evidence_root(j, op, {"manifest": {}}, "RECOVER" if label == "recovery" else "SEND")
        return c.Store(root/f"{len(op.chain) if sequence is None else sequence:08}-{label}")
    def guard(prepared, label, floor, debit=None, prior=None):
        successor_authorization(value, authorization_raw, authorization_sha256, journal)
        output = store(label)
        try:
            rpc = core.Rpc(c, j, output, _transport); calls = guard_calls(core, value["proposal"], prepared, floor); results = []
            for start in range(0, len(calls), 5): results += rpc.query(calls[start:start+5])
            result = verify_guard(core, c, value["proposal"], prepared, floor, results, debit=debit, prior=prior)
            receipt = rpc_receipt(output.path, 2)
        finally: output.close()
        successor_authorization(value, authorization_raw, authorization_sha256, journal)
        return receipt, result
    try:
        need(expected_tail == op.tail(), "successor_tail_compare_and_swap")
        if action == "CREATE":
            op.append("CREATE", {"package": value, "authorization": a, "authorizationSha256": authorization_sha256})
        else:
            state = fresh()
            need(state["package"] == value and state["authorizationRaw"] == authorization_raw, "successor_original_grant_required")
            mode = "OFFLINE_INJECTED_TEST" if _transport is not None else "FIXED_ORIGIN_TLS"
            if state["attempt"] is not None: need(state["attempt"]["mode"] == mode or action == "INSPECT", "successor_transport_mode_changed")
            if action == "CAPTURE_SIGN_SEND":
                need(state["stage"] == "CREATED", "successor_attempt_consumed_no_retry")
                need(type(signer_profile_path) is str and os.path.isabs(signer_profile_path) and
                     os.path.normpath(signer_profile_path) == signer_profile_path and "\0" not in signer_profile_path and
                     Path(__file__).resolve().parent.parent not in Path(signer_profile_path).parents, "successor_private_profile_path")
                output = store("quote")
                try:
                    rpc = core.Rpc(c, j, output, _transport)
                    initial = rpc.query([("getGenesisHash", []), ("getLatestBlockhash", [{"commitment": "finalized", "minContextSlot": CONTEXT_FLOOR}])])
                    need(initial[0] == GENESIS, "successor_quote_genesis")
                    quote = core.latest_quote(c, {"floor": CONTEXT_FLOOR}, initial[1]); quote_receipt = rpc_receipt(output.path, 1)
                finally: output.close()
                p = value["proposal"]; write = p["singleWrite"]
                wire, review = core.rebind_unsigned(c, write["unsignedWireBase64"], write["reviewed"], quote["recentBlockhash"])
                prepared = {"id": PHASE, "wireBase64": wire, "reviewed": review, "feeQuoteLamports": "5000",
                            "lifetime": {"recentBlockhash": quote["recentBlockhash"], "lastValidBlockHeight": quote["lastValidBlockHeight"],
                                         "currentBlockHeight": "407927461", "minimumRemainingBlockHeights": "40"}}
                before_receipt, before = guard(prepared, "presign", int(quote["contextSlot"]))
                prepared["lifetime"]["currentBlockHeight"] = before["height"]
                request = signing_request(p, a, prepared, op.tail())
                response_store = store("response")
                try:
                    op.append("SIGN_ATTEMPT", {"prepared": prepared, "quote": quote_receipt, "request": request, "guard": before_receipt,
                                               "guardResult": before, "mode": mode})
                    state = fresh()
                    # Durability can consume lifetime or expose changed state before keys.
                    guard(prepared, "post-sign-attempt", state["floor"], int(before["physicalOldDebitLamports"]), before)
                    payload = {"authorizationRawBase64": base64.b64encode(authorization_raw).decode(), "authorizationSha256": authorization_sha256,
                               "request": request, "packets": [prepared], "signerProfilePath": signer_profile_path}
                    response = _signer(payload) if _signer is not None else core.invoke_signer(j, payload)
                    response_raw = encode(response); need(len(response_raw) <= 262144, "successor_signer_response_bound")
                    response_store.write("signer-response.json", response_raw)
                finally: response_store.close()
                state = fresh()
                signed = signed_response(core, c, j, p, a, state["attempt"], canonical(response_raw))
                receipt, after = guard(prepared, "postsign", state["floor"], int(before["physicalOldDebitLamports"]), before)
                op.append("INTENT", {"responseSha256": sha(response_raw), "signed": signed, "guard": receipt, "guardResult": after,
                                     "authorizationSha256": authorization_sha256, "mode": mode})
                state = fresh()  # Reopen committed record bytes before the final guards.
                output = store("send")
                try:
                    _, dispatch = guard(prepared, "predispatch", state["floor"], int(before["physicalOldDebitLamports"]), after)
                    rpc = core.Rpc(c, j, output, _transport)
                    # The full account batch and its fsyncs can consume lifetime.
                    # Reuse the original small post-INTENT lifetime boundary.
                    floor = int(dispatch["contextSlot"]); life = prepared["lifetime"]
                    successor_authorization(value, authorization_raw, authorization_sha256, journal); fresh()
                    height, valid = rpc.query([("getBlockHeight", [{"commitment": "finalized", "minContextSlot": floor}]),
                                              ("isBlockhashValid", [life["recentBlockhash"], {"commitment": "finalized", "minContextSlot": floor}])])
                    need(c.u64(height) >= int(dispatch["height"]) and c.u64(height) <= int(life["lastValidBlockHeight"])-40 and
                         type(valid) is dict and valid.get("value") is True and c.u64(valid["context"]["slot"]) >= floor, "successor_post_intent_lifetime_margin")
                    recheck_live_grant(value, a, authorization_raw, authorization_sha256)
                    op.stable()
                    need(set(os.listdir(op.fd)) == {"lock"} | {f"{i:08}.json" for i in range(len(op.chain))} and
                         all(j.read_file(op.fd, f"{i:08}.json", MAX_RECORD) == encode(record) for i, record in enumerate(op.chain)), "successor_final_intent_changed")
                    try:
                        result = rpc.query([("sendTransaction", [state["intent"]["signed"]["wireBase64"],
                            {"encoding": "base64", "skipPreflight": False, "maxRetries": 0, "preflightCommitment": "finalized", "minContextSlot": int(dispatch["contextSlot"])}])])[0]
                        need(result == signed["signature"], "successor_send_signature_mismatch")
                        op.append("SUBMITTED", {"reason": "ONE_ORIGINAL_SEND_ACKNOWLEDGED"})
                    except Exception: op.append("UNKNOWN", {"reason": "NEW_ORIGINAL_SIGNATURE_ONLY_NO_RESEND"})
                finally: output.close()
            elif action == "RECOVER":
                if state["stage"] == "SIGN_ATTEMPT":
                    response_raw = read_public(root/"00000001-response"/"signer-response.json", 262144)
                    signed = signed_response(core, c, j, state["proposal"], a, state["attempt"], canonical(response_raw))
                    op.append("ORIGINAL_RESPONSE_HOLD", {"responseSha256": sha(response_raw), "signed": signed})
                    state = fresh()
                need(state["intent"] is not None and state["stage"] in ("INTENT", "SUBMITTED", "UNKNOWN", "RECOVER_ATTEMPT", "ORIGINAL_RESPONSE_HOLD"), "successor_original_intent_required")
                output = store("recovery"); directory = output.path; output.close()
                op.append("RECOVER_ATTEMPT", {"id": PHASE})
                request = recovery_request(core, state)
                report = (c.collect(request, directory/"collection", _transport=core.bounded_transport(c, _transport)) if _transport is not None else
                          core.native_collection("RECOVER", request, directory/"collection"))
                obs = report.get("observation")
                if obs is not None and obs["status"]["confirmationStatus"] == "finalized" and (obs["status"]["err"] is not None or
                        report["status"] == "COLLECTED_PUBLIC_EVIDENCE_ONLY" and obs["poststate"] is not None):
                    op.append("FINALIZED" if obs["status"]["err"] is None else "FAILED", {"manifestSha256": report["manifestReceipt"]["sha256"]})
                else: op.append("UNKNOWN", {"reason": "NEW_ORIGINAL_SIGNATURE_UNRESOLVED"})
        state = fresh(); intent = state["intent"]; terminal = state["stage"] in ("FINALIZED", "FAILED")
        debit = int(intent["guardResult"]["physicalOldDebitLamports"]) if intent else None
        return {"schema": CONTROLLER_SCHEMA, "status": state["stage"], "tail": op.tail(), "liveReady": False,
                "originalStatus": "UNKNOWN_HOLD_PRESERVED", "oldActualLamports": "55000", "oldReservedLamports": "60000",
                "oldPossibleFeeLamports": "5000", "combinedReservedLamports": "65000" if state["attempt"] else "60000",
                "combinedKnownActualLamports": "60000" if terminal else "55000", "newActualLamports": "5000" if terminal else "0",
                "physicalOldDebitLamports": str(debit) if debit is not None else None,
                "downstreamCanonicalG0AdjustmentLamports": str(downstream_balance_adjustment(debit, 5000)) if state["stage"] == "FINALIZED" else None,
                "dependentProgressionAllowed": False, "automaticRecoveryOrRetry": False}
    finally: op.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operator_records", help="Exact public55-record AU directory; read only")
    parser.add_argument("original_authorization", help="Pinned public activated-authorization-v1.json")
    parser.add_argument("original_stop", help="Pinned public upload-stop-v1.json")
    args = parser.parse_args()
    sys.stdout.buffer.write(encode(prepare(args.operator_records, args.original_authorization, args.original_stop)))


if __name__ == "__main__":
    try: main()
    except Exception as exc:
        print(json.dumps({"status": "STOP", "reason": str(exc) if isinstance(exc, Stop) else "invalid_public_input", "liveReady": False}), file=sys.stderr)
        raise SystemExit(1)
