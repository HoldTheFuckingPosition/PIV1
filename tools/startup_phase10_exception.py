#!/usr/bin/env python3
"""Prepare one inert AU phase-10 Write proposal from exact public evidence.

This command only reads public files and prints JSON. It never imports/runs the
old operator, changes its journal, signs, contacts RPC, or enables a successor.
Fresh live guards and old-fee accounting integration remain unimplemented.
"""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import stat
import struct
import sys

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
        "remainingIntegrationBlocker": "LIVE_SUCCESSOR_ACCOUNTING_NOT_IMPLEMENTED: canonical downstream G0 balances must account for retained old-fee uncertainty without clearing old UNKNOWN or weakening account equality.",
        "requiredLiveGuards": {"status": "OPEN_NOT_IMPLEMENTED_OR_SATISFIED_BY_THIS_PROPOSAL",
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
