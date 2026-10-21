"""Host-only raw Squads oracle/invariant tests; no signing, network or Bank."""
import base64
import copy
import fcntl
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import struct
import tempfile
import unittest
from unittest import mock

HERE = Path(__file__).parent
REPO = Path(os.environ.get("PIV1_PUBLIC_CHECK_REPO", HERE.parent))

def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module); return module

s = load("storage_gate", HERE / "startup_initializer_storage.py")
j = load("fixture_journal", REPO / "tools/startup_public_journal.py")
h = load("fixture_historical", REPO / "tools/test_startup_public_journal.py")
RAW = (HERE / "startup_initializer_storage.fixture.json").read_bytes()
assert hashlib.sha256(RAW).hexdigest() == "d7bffcddd02695e6f7fe0355018778870042260b1bbc6c58c0343aac3d4ab6b3"
FIXTURE = json.loads(RAW)
A = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"

# Independent serializer/PDA oracle: no production decoder used to construct bytes.
def public(text):
    number = 0
    for char in text: number = number * 58 + A.index(char)
    return number.to_bytes(32, "big")

def text(raw):
    number = int.from_bytes(raw, "big"); result = ""
    while number: number, digit = divmod(number, 58); result = A[digit] + result
    return "1" * (32 - len(raw.lstrip(b"\0"))) + result

def oracle_pdas(g):
    prime = (1 << 255) - 19; d = -121665 * pow(121666, -1, prime) % prime
    def decompresses(raw):
        y = int.from_bytes(raw, "little") & ((1 << 255) - 1)
        yy = y*y % prime; denominator = (d*yy + 1) % prime
        if not denominator: return False
        xx = (yy - 1) * pow(denominator, -1, prime) % prime
        return xx == 0 or pow(xx, (prime - 1)//2, prime) == 1
    def derive(seeds):
        for bump in range(255, -1, -1):
            raw = hashlib.sha256(b"".join(seeds) + bytes([bump]) + public(s.SQUADS) + b"ProgramDerivedAddress").digest()
            if not decompresses(raw): return {"address": text(raw), "bump": bump}
        raise AssertionError("no synthetic PDA")
    root = [b"multisig", public(g["multisigAddress"])]; index = struct.pack("<Q", int(g["transactionIndex"]))
    return {"multisig": derive([b"multisig", b"multisig", public(g["createKey"])]),
        "transaction": derive(root + [b"transaction", index]), "proposal": derive(root + [b"transaction", index, b"proposal"]),
        "vault": derive(root + [b"vault", bytes([g["vaultIndex"]])])}

def disc(name): return hashlib.sha256(("account:" + name).encode()).digest()[:8]
def u32(value): return struct.pack("<I", value)
def u64(value): return struct.pack("<Q", int(value))
def vector(keys): return u32(len(keys)) + b"".join(public(k) for k in keys)

def stored_transaction(seal, pdas):
    plan = seal["unsignedInitializerPlan"]; g = seal["postManifest"]["initializer"]["governance"]
    compact = bytes.fromhex(plan["compact"]["hex"])
    count = compact[3]; boundary = 4 + count*32; cursor = boundary
    assert compact[cursor] == 1; program, n = compact[cursor+1:cursor+3]; cursor += 3
    indices = compact[cursor:cursor+n]; cursor += n
    size = int.from_bytes(compact[cursor:cursor+2], "little"); cursor += 2
    payload = compact[cursor:cursor+size]; cursor += size; assert compact[cursor:] == b"\0"
    return (disc("VaultTransaction") + public(plan["multisig"]) + public(g["creator"]) + u64(g["transactionIndex"])
        + bytes([pdas["transaction"]["bump"], g["vaultIndex"], pdas["vault"]["bump"]]) + u32(0)
        + compact[:3] + u32(count) + compact[4:boundary] + u32(1) + bytes([program]) + u32(n) + indices
        + u32(size) + payload + u32(0))

def stored_multisig(g, pdas, rent=None, members=None, permissions=None, current=None, stale=None, threshold=4, timelock=None, config=None, padding=0xA5):
    members = sorted(g["guardians"], key=public) if members is None else members
    permissions = [7]*len(members) if permissions is None else permissions
    raw = (disc("Multisig") + public(g["createKey"]) + (bytes(32) if config is None else public(config))
        + struct.pack("<H", threshold) + u32(g["timeLockSeconds"] if timelock is None else timelock)
        + u64(g["transactionIndex"] if current is None else current) + u64(g["staleTransactionIndex"] if stale is None else stale)
        + (b"\0" if rent is None else b"\1" + public(rent)) + bytes([pdas["multisig"]["bump"]]) + u32(len(members))
        + b"".join(public(k) + bytes([p]) for k, p in zip(members, permissions)))
    return raw + bytes([padding])*max(0, 330-len(raw))

def stored_proposal(plan, g, pdas, stage, approved=None, rejected=(), cancelled=(), status=None, timestamp=1234567890, padding=0xA6):
    votes = sorted(g["approvers"], key=public) if stage == "POST_VOTE" else []
    if approved is not None: votes = approved
    raw = (disc("Proposal") + public(plan["multisig"]) + u64(g["transactionIndex"])
        + bytes([status if status is not None else 3 if stage == "POST_VOTE" else 1]) + struct.pack("<q", timestamp)
        + bytes([pdas["proposal"]["bump"]]) + vector(votes) + vector(rejected) + vector(cancelled))
    return raw + bytes([padding])*max(0, 646-len(raw))

def wrapped(raw):
    return {"exists": True, "owner": s.SQUADS, "executable": False, "lamports": "1", "dataBase64": base64.b64encode(raw).decode()}

def snapshot(seal, pdas, stage):
    plan = seal["unsignedInitializerPlan"]; g = seal["postManifest"]["initializer"]["governance"]
    return {"contextSlot": seal["clock"]["contextSlot"], "accounts": {
        plan["multisig"]: wrapped(stored_multisig(g, pdas)), plan["transaction"]: wrapped(stored_transaction(seal, pdas)),
        plan["proposal"]: wrapped(stored_proposal(plan, g, pdas, stage))}}


class InitializerStorageTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory(); cls.path = Path(cls.temp.name)/"sealed"
        segment = h.segment(); segment["input"]["frozen"]["transactionIndex"] = FIXTURE["preManifest"]["initializer"]["governance"]["transactionIndex"]
        tail = j.apply(h.request("CREATE", s=segment), cls.path)["tail"]
        tail, _ = h.append_states(cls.path, segment, tail, ("PREPARED", "SUBMISSION_START_RECORDED", "FINALIZED"))
        cls.unsealed = Path(cls.temp.name)/"unsealed"; cls.unsealed_tail = h.copy_local_journal(cls.path, cls.unsealed)
        tail = j.apply(h.request("FREEZE_INITIALIZER_PROFILE", tail, s=FIXTURE["preManifest"]), cls.path)["tail"]
        result = j.apply(h.request("SEAL_INITIALIZER_CLOCK", tail, s=FIXTURE["clock"]), cls.path)
        cls.tail, cls.seal = result["tail"], result["initializerSeal"]
        cls.plan = cls.seal["unsignedInitializerPlan"]; cls.g = cls.seal["postManifest"]["initializer"]["governance"]
        cls.pdas = oracle_pdas(cls.g); cls.raw = stored_transaction(cls.seal, cls.pdas)
        cls.initial_bytes = {p.name:p.read_bytes() for p in cls.path.iterdir()}

    @classmethod
    def tearDownClass(cls): cls.temp.cleanup()

    def request(self, stage="PRE_VOTE"):
        return {"schema":s.SCHEMA, "expectedTail":copy.deepcopy(self.tail), "stage":stage, "snapshot":snapshot(self.seal,self.pdas,stage)}

    def pure(self, value, stage="PRE_VOTE", seal=None):
        return s.validate_snapshot(value, seal or self.seal, self.pdas, stage)

    def test_nullable_is_strict_and_does_not_touch_journal_or_import_helpers(self):
        with mock.patch.object(s,"journal_module",side_effect=AssertionError("nullable import/access")):
            result = s.check(copy.deepcopy(s.TEMPLATE), "/never-created-public-null-journal")
            self.assertEqual(result["status"],"INCOMPLETE"); self.assertFalse(result["liveReady"])
            for field in ("expectedTail","snapshot"):
                bad=copy.deepcopy(s.TEMPLATE);bad[field]={}
                with self.assertRaises(s.Stop):s.check(bad,"/never-created-public-null-journal")

    def test_actual_pinned_pre_and_post_gate_exact_public_bytes_and_parent_immutable(self):
        for stage in ("PRE_VOTE","POST_VOTE"):
            result=s.check(self.request(stage),self.path)
            self.assertEqual(result["status"],"VERIFIED_SUPPLIED_INITIALIZER_STORAGE_ONLY")
            self.assertEqual(result["pdaDerivations"],self.pdas)
            self.assertEqual(result["inheritedLedgerReference"],{"actualFeeLamports":"5000","reservedFeeLamports":"5000","cumulativeFeeCapLamports":"25000"})
            self.assertEqual(result["evidence"]["transaction"]["compactSha256"],self.seal["compactSha256"])
            self.assertEqual(result["evidence"]["transaction"]["nativeSha256"],self.seal["nativeSha256"])
            self.assertEqual(result["evidence"]["transaction"]["orderedRolesSha256"],self.seal["orderedRolesSha256"])
            self.assertEqual(result["nodeProcessInvocations"],3)
            self.assertTrue(all(result[k] is False for k in s.FLAGS))
            self.assertEqual(self.initial_bytes,{p.name:p.read_bytes() for p in self.path.iterdir()})
        # Stored Borsh is deliberately not the compact suffix representation.
        self.assertFalse(self.raw.endswith(bytes.fromhex(self.plan["compact"]["hex"])))

    def test_exhaustive_stored_transaction_truncation_and_extra_bytes_reject(self):
        for size in range(len(self.raw)):
            with self.assertRaises(s.Stop):s.transaction(self.raw[:size],self.seal,self.g,self.pdas)
        for suffix in (b"\0",b"\xff",bytes(32)):
            with self.assertRaises(s.Stop):s.transaction(self.raw+suffix,self.seal,self.g,self.pdas)

    def test_generated_every_stored_transaction_byte_substitution_is_rejected(self):
        for position in range(len(self.raw)):
            changed=bytearray(self.raw);changed[position]^=1
            with self.assertRaises(s.Stop,msg=str(position)):s.transaction(bytes(changed),self.seal,self.g,self.pdas)

    def test_stored_u32_lengths_duplicates_roles_program_and_lookup_are_semantic(self):
        # Header begins after discriminator/root/creator/index/3 bumps/empty vec.
        start=8+32+32+8+3+4; keys=start+3+4; count=self.raw[start+3:start+7]; n=int.from_bytes(count,"little")
        for position,value in [(start+3,b"\xff"*4),(keys+32,self.raw[keys:keys+32]),(keys+32*n,b"\x02\0\0\0"), (len(self.raw)-4,b"\1\0\0\0")]:
            changed=bytearray(self.raw);changed[position:position+len(value)]=value
            with self.assertRaises(s.Stop):s.transaction(bytes(changed),self.seal,self.g,self.pdas)
        changed=copy.deepcopy(self.seal);changed["unsignedInitializerPlan"]["native"]["roles"][0]["writable"]^=True
        with self.assertRaisesRegex(s.Stop,"stored_ordered_roles"):s.transaction(self.raw,changed,self.g,self.pdas)

    def test_variable_rent_option_and_source_valid_dirty_allocation_tails(self):
        for rent in (None,self.g["creator"]):
            raw=stored_multisig(self.g,self.pdas,rent=rent)
            parsed=s.multisig(raw,self.g,self.pdas)
            self.assertEqual(parsed["rentCollector"],rent)
            self.assertEqual(parsed["unusedAllocationBytes"],32 if rent is None else 0)
        for stage in ("PRE_VOTE","POST_VOTE"):
            raw=stored_proposal(self.plan,self.g,self.pdas,stage)
            parsed=s.proposal(raw,stage,self.plan,self.g,self.pdas,sorted(self.g["guardians"],key=public))
            self.assertGreater(parsed["unusedAllocationBytes"],0)
        invalid=bytearray(stored_multisig(self.g,self.pdas));invalid[94]=2
        with self.assertRaisesRegex(s.Stop,"rent_collector_option"):s.multisig(bytes(invalid),self.g,self.pdas)

    def test_multisig_startup_members_permissions_authority_threshold_index_timelock(self):
        members=sorted(self.g["guardians"],key=public)
        changes=[{"members":list(reversed(members))},{"members":[members[0]]*6},{"members":members[:-1]},
            {"members":["11111111111111111111111111111111"]+members[1:]},{"threshold":3}, {"config":self.g["creator"]},
            {"timelock":self.g["timeLockSeconds"]+1},{"current":"72"},{"current":"74"},{"current":str(s.U64)}, {"stale":"73"},{"stale":"1"}]
        changes += [{"permissions":[p]+[7]*5} for p in (0,1,2,3,6,8,255)]
        for change in changes:
            with self.assertRaises(s.Stop,msg=str(change)):s.multisig(stored_multisig(self.g,self.pdas,**change),self.g,self.pdas)

    def test_proposal_stage_status_exact_sorted_four_and_vote_ineligibility(self):
        members=sorted(self.g["guardians"],key=public); approvals=sorted(self.g["approvers"],key=public)
        for stage in ("PRE_VOTE","POST_VOTE"):
            for status in range(7):
                if status==(1 if stage=="PRE_VOTE" else 3):continue
                raw=stored_proposal(self.plan,self.g,self.pdas,stage,status=status)
                with self.assertRaisesRegex(s.Stop,"proposal_stage_status"):s.proposal(raw,stage,self.plan,self.g,self.pdas,members)
        for change in [{"approved":[]},{"approved":approvals[:3]},{"approved":members[:5]}, {"approved":approvals[::-1]},
            {"approved":[approvals[0]]*4},{"approved":[self.g["createKey"]]+approvals[:3]}, {"rejected":approvals[:1]},{"cancelled":approvals[:1]}]:
            raw=stored_proposal(self.plan,self.g,self.pdas,"POST_VOTE",**change)
            with self.assertRaises(s.Stop):s.proposal(raw,"POST_VOTE",self.plan,self.g,self.pdas,members)
        raw=stored_proposal(self.plan,self.g,self.pdas,"PRE_VOTE",approved=approvals)
        with self.assertRaisesRegex(s.Stop,"stage_approvals"):s.proposal(raw,"PRE_VOTE",self.plan,self.g,self.pdas,members)

    def test_allocation_lengths_and_proposal_count_signed_timestamp_edges(self):
        members=sorted(self.g["guardians"],key=public)
        for parser,raw in [(lambda b:s.multisig(b,self.g,self.pdas),stored_multisig(self.g,self.pdas)),
            (lambda b:s.proposal(b,"PRE_VOTE",self.plan,self.g,self.pdas,members),stored_proposal(self.plan,self.g,self.pdas,"PRE_VOTE"))]:
            for size in range(len(raw)):
                with self.assertRaises(s.Stop):parser(raw[:size])
        raw=bytearray(stored_proposal(self.plan,self.g,self.pdas,"PRE_VOTE"));raw[58:62]=u32(7)
        with self.assertRaisesRegex(s.Stop,"borsh_count"):s.proposal(bytes(raw),"PRE_VOTE",self.plan,self.g,self.pdas,members)
        for timestamp in (-(1<<63),-1,0,(1<<63)-1):
            raw=stored_proposal(self.plan,self.g,self.pdas,"PRE_VOTE",timestamp=timestamp)
            self.assertEqual(s.proposal(raw,"PRE_VOTE",self.plan,self.g,self.pdas,members)["timestamp"],str(timestamp))

    def test_common_snapshot_context_u64_account_identity_exact_set_and_canonical_data(self):
        value=snapshot(self.seal,self.pdas,"PRE_VOTE");value["contextSlot"]=str(s.U64);self.pure(value)
        for bad in (True,1,-1,"01",str(s.U64+1),"-1",str(int(self.seal["clock"]["contextSlot"])-1)):
            value=snapshot(self.seal,self.pdas,"PRE_VOTE");value["contextSlot"]=bad
            with self.assertRaises(s.Stop):self.pure(value)
        for change in ({"exists":False},{"owner":"11111111111111111111111111111111"},{"executable":True},{"lamports":"0"},{"lamports":True},{"lamports":str(s.U64+1)},{"dataBase64":"AA==\n"},{"dataBase64":"AB=="}):
            value=snapshot(self.seal,self.pdas,"PRE_VOTE");value["accounts"][self.plan["transaction"]].update(change)
            with self.assertRaises(s.Stop):self.pure(value)
        value=snapshot(self.seal,self.pdas,"PRE_VOTE");value["accounts"][self.plan["multisig"]]["lamports"]=str(s.U64);self.pure(value)
        for mutate in (lambda a:a.pop(self.plan["transaction"]),lambda a:a.update({self.g["createKey"]:wrapped(b"x")})):
            value=snapshot(self.seal,self.pdas,"PRE_VOTE");mutate(value["accounts"])
            with self.assertRaises(s.Stop):self.pure(value)
        pdas=copy.deepcopy(self.pdas);pdas["transaction"]["address"]=self.g["createKey"]
        with self.assertRaisesRegex(s.Stop,"derived_pda"):s.validate_snapshot(snapshot(self.seal,self.pdas,"PRE_VOTE"),self.seal,pdas,"PRE_VOTE")

    def test_external_tail_unsealed_corrupt_parent_and_cap_reset_rejected_without_writes(self):
        bad=self.request();bad["expectedTail"]["sequence"]="4"
        with self.assertRaisesRegex(s.Stop,"tail_compare_and_swap"):s.check(bad,self.path)
        bad=self.request();bad["expectedTail"]=self.unsealed_tail
        with self.assertRaisesRegex(s.Stop,"terminal_seal_required"):s.check(bad,self.unsealed)
        bad=self.request();bad["cumulativeFeeCapLamports"]="0"
        with self.assertRaisesRegex(s.Stop,"object_fields"):s.check(bad,self.path)
        with tempfile.TemporaryDirectory() as temp:
            path=Path(temp)/"corrupt";h.copy_local_journal(self.unsealed,path)
            item=path/"00000000.json";item.write_bytes(item.read_bytes()[:-1])
            before={p.name:p.read_bytes() for p in path.iterdir()}
            with self.assertRaises(s.Stop):s.check(self.request(),path)
            self.assertEqual(before,{p.name:p.read_bytes() for p in path.iterdir()})
        self.assertEqual(self.initial_bytes,{p.name:p.read_bytes() for p in self.path.iterdir()})

    def test_source_pin_is_checked_before_import_and_strict_json_public_encodings(self):
        with mock.patch.dict(s.SOURCE_PINS,{"tools/startup_public_journal.py":"0"*64}):
            with mock.patch.object(s.importlib.util,"spec_from_file_location",side_effect=AssertionError("must not execute unpinned module")):
                with self.assertRaisesRegex(s.Stop,"source_pin"):s.journal_module()
        for raw in (b'{"x":0,"x":1}',b'{"x":NaN}',b'{"x":1.5}',b'{"x":Infinity}',b'['*2000):
            with self.assertRaises(s.Stop):s.parse(raw)
        for raw in [bytes(32),bytes([255])*32]+[public(k) for k in self.g["guardians"]]:
            self.assertEqual(s.key(s.b58(raw)),raw)
        for bad in ("0"*32,"1"*31,"1"*33,"z"*44,None):
            with self.assertRaises(s.Stop):s.key(bad)

    def test_exclusive_stable_lock_held_through_rederivation_and_snapshot(self):
        real_validate=j.validate_chain;real_snapshot=s.validate_snapshot;seen=[]
        def assert_locked():
            fd=os.open(self.path/"lock",os.O_RDONLY)
            try:
                with self.assertRaises(BlockingIOError):fcntl.flock(fd,fcntl.LOCK_EX|fcntl.LOCK_NB)
            finally:os.close(fd)
        def validated(chain):assert_locked();seen.append("parent");return real_validate(chain)
        def inspected(*args):assert_locked();seen.append("snapshot");return real_snapshot(*args)
        with mock.patch.object(s,"journal_module",return_value=(j,REPO)):
            with mock.patch.object(j,"validate_chain",side_effect=validated),mock.patch.object(s,"validate_snapshot",side_effect=inspected):
                result=s.check(self.request(),self.path)
        self.assertEqual(seen,["parent","snapshot"]);self.assertFalse(result["journalMutationPerformed"])
        self.assertEqual(self.initial_bytes,{p.name:p.read_bytes() for p in self.path.iterdir()})


if __name__ == "__main__":unittest.main()
