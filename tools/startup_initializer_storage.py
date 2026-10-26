#!/usr/bin/env python3
"""Read-only supplied Squads initializer evidence; never a successor or signer."""
import base64
import fcntl
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import selectors
import struct
import subprocess
import sys
import time

SCHEMA = "piv1-startup-initializer-storage-v1"
TEMPLATE = {"schema": SCHEMA, "expectedTail": None, "stage": None, "snapshot": None}
JOURNAL_SHA = "bfea079a5740fa66e437930d50793a18289beeae1760af4164e986295c8a6d37"
SOURCE_PINS = {"tools/startup_public_journal.py": JOURNAL_SHA,
    "programs/piv1/src/squads_accounts.rs": "b5ccacf047d86666eadaf568019ac2df7318fb954d62ae87866ba284324e518b",
    "programs/piv1/src/squads_execution.rs": "589252b8357d76e9af61d4f4aaed7e7d54692db63bddd98ea807cee6357321fd"}
SQUADS = "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf"
ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
MAX_INPUT = 65536
MAX_ACCOUNT = 4096
U64 = (1 << 64) - 1
FLAGS = {"liveReady": False, "authorized": False, "chainStateAuthenticated": False,
    "callerAnchorAuthenticated": False, "snapshotFreshnessAuthenticated": False,
    "preVoteCausalityProved": False, "voteSignatureProvenanceProved": False,
    "timelockReleased": False, "executionEligible": False, "controlledSuccessor": False,
    "signingOrSendPerformed": False, "journalMutationPerformed": False,
    "newFeesReserved": False, "completeStateOracle": False}
PDA_SCRIPT = r"""
const fs=require('node:fs');
const {PublicKey}=require(process.env.PIV1_PUBLIC_CHECK_REPO+'/spikes/task-0.4-jito/node_modules/@solana/web3.js/lib/index.cjs.js');
const raw=fs.readFileSync(0);if(raw.length>4096)process.exit(2);
const x=JSON.parse(raw.toString('utf8')),p=new PublicKey('SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf');
const index=Buffer.alloc(8);index.writeBigUInt64LE(BigInt(x.transactionIndex));
const root=new PublicKey(x.multisigAddress).toBuffer(),text=s=>Buffer.from(s);
const derive=s=>{const [k,b]=PublicKey.findProgramAddressSync(s,p);return {address:k.toBase58(),bump:b}};
const result={multisig:derive([text('multisig'),text('multisig'),new PublicKey(x.createKey).toBuffer()]),
 transaction:derive([text('multisig'),root,text('transaction'),index]),
 proposal:derive([text('multisig'),root,text('transaction'),index,text('proposal')]),
 vault:derive([text('multisig'),root,text('vault'),Buffer.from([x.vaultIndex])])};
const out=JSON.stringify(result);if(Buffer.byteLength(out)>4096)process.exit(3);process.stdout.write(out);
"""


class Stop(ValueError):
    pass


def need(ok, code):
    if not ok: raise Stop(code)


def fields(value, keys):
    need(type(value) is dict and set(value) == set(keys), "object_fields")


def encode(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False) + "\n").encode()


def sha(raw): return hashlib.sha256(raw).hexdigest()


def uint(value):
    need(type(value) is str and re.fullmatch(r"0|[1-9][0-9]{0,19}", value) is not None and int(value) <= U64, "u64_string")
    return int(value)


def parse(raw):
    need(type(raw) is bytes and len(raw) <= MAX_INPUT, "json_bound")
    def pairs(items):
        result = {}
        for key, value in items:
            need(key not in result, "duplicate_json_key"); result[key] = value
        return result
    def invalid(_): raise Stop("noninteger_or_nonfinite_json")
    try: return json.loads(raw.decode(), object_pairs_hook=pairs, parse_float=invalid, parse_constant=invalid)
    except (ValueError, UnicodeError, RecursionError) as exc:
        if isinstance(exc, Stop): raise
        raise Stop("invalid_json") from exc


def b58(raw):
    number = int.from_bytes(raw, "big"); text = ""
    while number: number, digit = divmod(number, 58); text = ALPHABET[digit] + text
    return "1" * (len(raw) - len(raw.lstrip(b"\0"))) + text


def key(text):
    need(type(text) is str and 32 <= len(text) <= 44 and all(c in ALPHABET for c in text), "public_key")
    number = 0
    for c in text: number = number * 58 + ALPHABET.index(c)
    raw = b"\0" * (len(text) - len(text.lstrip("1"))) + number.to_bytes((number.bit_length() + 7) // 8, "big")
    need(len(raw) == 32 and b58(raw) == text, "public_key")
    return raw


def account(value):
    fields(value, ("exists", "owner", "executable", "lamports", "dataBase64"))
    need(value["exists"] is True and value["owner"] == SQUADS and value["executable"] is False, "account_identity")
    need(uint(value["lamports"]) > 0, "account_lamports")
    text = value["dataBase64"]
    need(type(text) is str and len(text) <= 4 * ((MAX_ACCOUNT + 2) // 3), "account_base64_bound")
    try: raw = base64.b64decode(text, validate=True)
    except ValueError as exc: raise Stop("account_base64") from exc
    need(len(raw) <= MAX_ACCOUNT and base64.b64encode(raw).decode() == text, "account_base64")
    return raw


class Reader:
    def __init__(self, raw): self.raw, self.offset = raw, 0
    def take(self, size):
        need(type(size) is int and 0 <= size <= len(self.raw) - self.offset, "borsh_truncated")
        part = self.raw[self.offset:self.offset + size]; self.offset += size; return part
    def number(self, size): return int.from_bytes(self.take(size), "little")
    def count(self, maximum):
        count = self.number(4); need(count <= maximum, "borsh_count"); return count
    def disc(self, name): need(self.take(8) == hashlib.sha256(("account:" + name).encode()).digest()[:8], "account_discriminator")
    def public_key(self): return b58(self.take(32))
    def same(self, value, size, code): need(self.number(size) == value, code)


def multisig(raw, g, pdas):
    need(len(raw) >= 330, "multisig_allocation")
    r = Reader(raw); r.disc("Multisig")
    need(r.public_key() == g["createKey"] and r.take(32) == bytes(32), "multisig_authority")
    r.same(4, 2, "multisig_threshold"); r.same(g["timeLockSeconds"], 4, "multisig_timelock")
    current, stale = r.number(8), r.number(8)
    need(current == uint(g["transactionIndex"]) and stale == uint(g["staleTransactionIndex"]) and stale < current, "startup_current_or_stale_index")
    option = r.number(1); need(option in (0, 1), "rent_collector_option")
    rent = r.public_key() if option else None
    r.same(pdas["multisig"]["bump"], 1, "multisig_bump"); need(r.count(6) == 6, "member_count")
    members = []
    for _ in range(6):
        member = r.public_key(); permission = r.number(1)
        need(key(member) != bytes(32) and permission == 7 and (not members or key(members[-1]) < key(member)), "member_order_or_permissions")
        members.append(member)
    need(members == sorted(g["guardians"], key=key), "member_set")
    return {"currentTransactionIndex": str(current), "staleTransactionIndex": str(stale), "rentCollector": rent, "members": members, "unusedAllocationBytes": len(raw) - r.offset}


def transaction(raw, seal, g, pdas):
    plan = seal["unsignedInitializerPlan"]
    need(len(raw) == plan["accountSizes"]["transaction"], "transaction_allocation")
    r = Reader(raw); r.disc("VaultTransaction")
    need(r.public_key() == plan["multisig"] and r.public_key() == g["creator"], "transaction_root_or_creator")
    r.same(uint(g["transactionIndex"]), 8, "transaction_index")
    r.same(pdas["transaction"]["bump"], 1, "transaction_bump"); r.same(g["vaultIndex"], 1, "transaction_vault_index")
    r.same(pdas["vault"]["bump"], 1, "vault_bump"); need(r.count(0) == 0, "ephemeral_signers")
    header = r.take(3); signers, writable_signers, writable_other = header
    count = r.count(255); need(count > 0 and writable_signers <= signers <= count and writable_other <= count - signers, "stored_header")
    keys = [r.take(32) for _ in range(count)]; need(len(set(keys)) == count, "stored_duplicate_keys")
    need(r.count(1) == 1, "stored_instruction_count"); program = r.number(1)
    need(program < count and b58(keys[program]) == plan["program"], "stored_program")
    indices = r.take(r.count(35)); need(len(indices) == len(plan["native"]["roles"]) and len(set(indices)) == len(indices) and all(i < count for i in indices), "stored_role_indices")
    need(set(indices) | {program} == set(range(count)), "stored_unused_keys")
    data = r.take(r.count(315)); need(data == bytes.fromhex(plan["native"]["hex"]), "stored_native_mismatch")
    need(r.count(0) == 0 and r.offset == len(raw), "stored_lookup_or_trailing")
    compact = header + bytes([count]) + b"".join(keys) + b"\x01" + bytes([program, len(indices)]) + indices + struct.pack("<H", len(data)) + data + b"\0"
    need(compact == bytes.fromhex(plan["compact"]["hex"]) and sha(compact) == seal["compactSha256"], "stored_compact_mismatch")
    for position, (role, index) in enumerate(zip(plan["native"]["roles"], indices)):
        need(role["index"] == position and role["address"] == b58(keys[index]) and role["signer"] == (index < signers) and role["writable"] == (index < writable_signers if index < signers else index < signers + writable_other), "stored_ordered_roles")
    return {"nativeSha256": sha(data), "compactSha256": sha(compact), "orderedRolesSha256": sha(encode(plan["native"]["roles"])), "storedMessageFormat": "BORSH_U32_VECTORS_REENCODED_TO_EXACT_COMPACT"}


def proposal(raw, stage, plan, g, pdas, members):
    need(len(raw) >= plan["accountSizes"]["proposal"], "proposal_allocation")
    r = Reader(raw); r.disc("Proposal")
    need(r.public_key() == plan["multisig"], "proposal_multisig")
    r.same(uint(g["transactionIndex"]), 8, "proposal_index")
    status = r.number(1); need(status == (1 if stage == "PRE_VOTE" else 3), "proposal_stage_status")
    timestamp = int.from_bytes(r.take(8), "little", signed=True)
    r.same(pdas["proposal"]["bump"], 1, "proposal_bump")
    vectors = []
    for _ in range(3):
        votes = [r.public_key() for _ in range(r.count(6))]
        need(all(v in members for v in votes) and all(key(a) < key(b) for a, b in zip(votes, votes[1:])), "vote_order_or_membership")
        vectors.append(votes)
    approved, rejected, cancelled = vectors
    need(not rejected and not cancelled, "rejected_or_cancelled_votes")
    expected = [] if stage == "PRE_VOTE" else sorted(g["approvers"], key=key)
    need(approved == expected and (stage == "PRE_VOTE" or len(set(approved)) == 4), "stage_approvals")
    return {"status": "Active" if status == 1 else "Approved", "timestamp": str(timestamp), "approved": approved, "rejected": rejected, "cancelled": cancelled, "unusedAllocationBytes": len(raw) - r.offset}


def validate_snapshot(snapshot, seal, pdas, stage):
    need(stage in ("PRE_VOTE", "POST_VOTE"), "stage")
    fields(snapshot, ("contextSlot", "accounts")); context = uint(snapshot["contextSlot"])
    need(context >= max(uint(seal["contextFloor"]), uint(seal["clock"]["contextSlot"])), "snapshot_context_stale")
    plan = seal["unsignedInitializerPlan"]; g = seal["postManifest"]["initializer"]["governance"]
    fields(pdas, ("multisig", "transaction", "proposal", "vault"))
    for name, derived in pdas.items():
        fields(derived, ("address", "bump")); need(derived["address"] == plan[name] and type(derived["bump"]) is int and 0 <= derived["bump"] <= 255, "derived_pda")
    fields(snapshot["accounts"], (plan["multisig"], plan["transaction"], plan["proposal"]))
    raw = {name: account(snapshot["accounts"][plan[name]]) for name in ("multisig", "transaction", "proposal")}
    m = multisig(raw["multisig"], g, pdas)
    return {"multisig": m, "transaction": transaction(raw["transaction"], seal, g, pdas),
        "proposal": proposal(raw["proposal"], stage, plan, g, pdas, m["members"]),
        "accountDataSha256": {plan[name]: sha(data) for name, data in raw.items()}}


def journal_module():
    repo = Path(os.environ.get("PIV1_PUBLIC_CHECK_REPO", Path(__file__).resolve().parent.parent)).resolve()
    for name, expected in SOURCE_PINS.items():
        path = repo / name
        need(not path.is_symlink() and path.is_file() and sha(path.read_bytes()) == expected, "source_pin")
    spec = importlib.util.spec_from_file_location("pinned_storage_parent_journal", repo / "tools/startup_public_journal.py")
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    need(module.pins(planner=True).resolve() == repo, "source_repo")
    return module, repo


def derive_pdas(j, repo, g):
    payload = encode({k: g[k] for k in ("multisigAddress", "createKey", "transactionIndex", "vaultIndex")})
    need(len(payload) <= 4096, "pda_input_bound")
    process = subprocess.Popen([str(j.NODE), "-e", PDA_SCRIPT], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        cwd=repo, env={"PIV1_PUBLIC_CHECK_REPO": str(repo)}, close_fds=True)
    selector = selectors.DefaultSelector(); streams = (process.stdin, process.stdout, process.stderr)
    output = {process.stdout: bytearray(), process.stderr: bytearray()}; position = 0; deadline = time.monotonic() + 20
    try:
        for stream in streams:
            os.set_blocking(stream.fileno(), False); selector.register(stream, selectors.EVENT_WRITE if stream is process.stdin else selectors.EVENT_READ)
        while selector.get_map():
            remaining = deadline - time.monotonic(); need(remaining > 0, "pda_timeout")
            for item, _ in selector.select(min(remaining, 0.5)):
                stream = item.fileobj
                if stream is process.stdin:
                    try: position += os.write(stream.fileno(), payload[position:])
                    except BrokenPipeError as exc: raise Stop("pda_stdin") from exc
                    if position == len(payload): selector.unregister(stream); stream.close()
                else:
                    chunk = os.read(stream.fileno(), 4096)
                    if not chunk: selector.unregister(stream)
                    else: output[stream].extend(chunk); need(len(output[stream]) <= 4096, "pda_output_bound")
        need(process.wait(timeout=max(0.001, deadline - time.monotonic())) == 0 and not output[process.stderr], "pda_process_failure")
        return parse(bytes(output[process.stdout]))
    finally:
        if process.poll() is None: process.kill(); process.wait()
        selector.close()
        for stream in streams:
            if not stream.closed: stream.close()


def check(request, path):
    fields(request, TEMPLATE); need(request["schema"] == SCHEMA, "schema")
    if request["stage"] is None:
        need(request["expectedTail"] is None and request["snapshot"] is None, "nullable_fields")
        return {"status": "INCOMPLETE", **FLAGS}
    need(request["stage"] in ("PRE_VOTE", "POST_VOTE"), "stage")
    fields(request["expectedTail"], ("sequence", "digest")); uint(request["expectedTail"]["sequence"])
    need(type(request["expectedTail"]["digest"]) is str and re.fullmatch("[0-9a-f]{64}", request["expectedTail"]["digest"]) is not None, "tail_digest")
    j, repo = journal_module(); fd = j.directory(path); lock = None
    try:
        lock = os.open("lock", os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=fd); meta = os.fstat(lock); j.secure_meta(meta)
        fcntl.flock(lock, fcntl.LOCK_EX)
        def stable():
            current = os.stat("lock", dir_fd=fd, follow_symlinks=False)
            need((current.st_dev, current.st_ino, current.st_ctime_ns) == (meta.st_dev, meta.st_ino, meta.st_ctime_ns) and j.read_file(fd, "lock", 128) == j.LOCK_BYTES, "lock_replaced_or_corrupt")
        stable(); chain, _ = j.records(fd)
        need(chain[0]["lockIdentity"] == {"device": str(meta.st_dev), "inode": str(meta.st_ino), "ctimeNs": str(meta.st_ctime_ns)}, "stable_lock_replaced")
        need(j.tail(chain) == request["expectedTail"], "tail_compare_and_swap")
        need(chain[-1]["kind"] == "SEAL", "terminal_seal_required")
        checked = j.validate_chain(chain); seal = checked["initializerSeal"]
        need(seal is not None and checked["journalMutationPolicy"] == "TERMINALLY_SEALED_NO_CHILD_CONTROLLER", "terminal_seal_required")
        pdas = derive_pdas(j, repo, seal["postManifest"]["initializer"]["governance"])
        evidence = validate_snapshot(request["snapshot"], seal, pdas, request["stage"]); stable()
        return {"status": "VERIFIED_SUPPLIED_INITIALIZER_STORAGE_ONLY", "stage": request["stage"], "parentTail": j.tail(chain),
            "sealDataSha256": sha(encode(seal)), "postManifestSha256": seal["postManifestSha256"], "snapshotSha256": sha(encode(request["snapshot"])),
            "snapshotContextSlot": request["snapshot"]["contextSlot"], "inheritedLedgerReference": checked["initializerProfile"]["parentLedger"],
            "pdaDerivations": pdas, "evidence": evidence, "checkerProcessInvocations": 1, "initializerPlannerProcessInvocations": 1,
            "pdaProcessInvocations": 1, "nodeProcessInvocations": 3, "journalMutationPolicy": checked["journalMutationPolicy"], **FLAGS}
    except j.Stop as exc: raise Stop(str(exc)) from exc
    finally:
        if lock is not None: fcntl.flock(lock, fcntl.LOCK_UN); os.close(lock)
        os.close(fd)


def main(argv):
    try:
        if argv == ["--template"]: result = TEMPLATE
        elif len(argv) == 4 and argv[0] == "--request" and argv[2] == "--journal":
            with open(argv[1], "rb") as stream: request = parse(stream.read(MAX_INPUT + 1))
            result = check(request, argv[3])
        else: raise Stop("usage")
        sys.stdout.buffer.write(encode(result)); return 0
    except Exception as exc:
        sys.stdout.buffer.write(encode({"status": "STOP", "reason": str(exc) if isinstance(exc, Stop) else "storage_check_failure", **FLAGS})); return 1


if __name__ == "__main__": sys.exit(main(sys.argv[1:]))


# Supplied candidate bytes only: no legacy journal, terminal seal or successor adoption.
CANDIDATE_JOURNAL_SHA = "c492c1fae58caeb88f8fbe137251440626aeb051514b3f6157fefce1a0155b48"


def candidate_journal_module():
    repo = Path(os.environ.get("PIV1_PUBLIC_CHECK_REPO", Path(__file__).resolve().parent.parent)).resolve()
    expected = {**SOURCE_PINS, "tools/startup_public_journal.py": CANDIDATE_JOURNAL_SHA}
    for name, digest in expected.items():
        path = repo / name
        need(not path.is_symlink() and path.is_file() and sha(path.read_bytes()) == digest, "candidate_source_pin")
    spec = importlib.util.spec_from_file_location("candidate_storage_parent", repo / "tools/startup_public_journal.py")
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    need(module.candidate_pins().resolve() == repo, "candidate_source_repo")
    return module, repo


def candidate_check_supplied(binding, snapshot, stage):
    """Check one exact candidate binding and three supplied Squads accounts, without a journal."""
    j, repo = candidate_journal_module()
    try:
        rebuilt = j.validate_candidate_clock_binding(binding)
        pdas = derive_pdas(j, repo, rebuilt["postManifest"]["initializer"]["governance"])
        evidence = validate_snapshot(snapshot, rebuilt, pdas, stage)
        return {"schema": "piv1-candidate-initializer-storage-v2", "scope": j.CANDIDATE_SCOPE,
            "status": "VERIFIED_SUPPLIED_CANDIDATE_INITIALIZER_STORAGE_ONLY", "stage": stage,
            "bindingSha256": sha(encode(rebuilt)), "snapshotSha256": sha(encode(snapshot)),
            "snapshotContextSlot": snapshot["contextSlot"], "pdaDerivations": pdas, "evidence": evidence,
            "initializerPlannerProcessInvocations": 1, "pdaProcessInvocations": 1, **FLAGS}
    except j.Stop as exc:
        raise Stop(str(exc)) from exc
