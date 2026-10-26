"""Exact startup byte rules for four runtime-dependent fields; no RPC or signing.

Rent values are the frozen selected startup quote profile, not a fresh Rent oracle.
Clock windows are operational checks over fixed-origin supplied bytes, not proofs
of inclusion-time Clock. An ambiguous heartbeat period fails closed.
"""
import base64
import copy
import hashlib
from pathlib import Path
import re
import struct

PROGRAM = "GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux"
PROGRAM_DATA = "6HV5GQesbZpTyBVTiMu64UU4JgCX2D2s489sKt26xpd8"
VAULT = "A2UHpGyiCBdUZDMrofGswNiyS4ATD1ytBSiAsWNdxUTw"
LOADER = "BPFLoaderUpgradeab1e11111111111111111111111"
SQUADS = "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf"
SYSTEM = "11111111111111111111111111111111"
TOKEN = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
CLOCK = "SysvarC1ock11111111111111111111111111111111"
SYSVAR = "Sysvar1111111111111111111111111111111111111"
ELF_SHA = "e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99"
PERIOD = 2592000
I64_MIN, I64_MAX = -(1 << 63), (1 << 63) - 1
KINDS = {"PROGRAM_DATA", "INITIALIZED_TARGET", "PROPOSAL", "HEARTBEAT_REWARD_0"}
RENT_FLOORS = {0: "650240", 84: "1076960", 165: "1488440", 210: "1717040",
               646: "3931920", 891: "5176520", 1014: "5801360", 894597: "4545203000"}
SOURCE_PINS = {
    "programs/piv1/src/genesis_model.rs": "fe1544ee506429ad9b2d02fee8fdea381cd27265d9ad220360076b7d25e6cc44",
    "programs/piv1/src/state/config.rs": "9d7a57c87b6781b943e0e9c13d08f73a21058d400b40540c485e6c596b042198",
    "programs/piv1/src/state/guardian.rs": "ce762257b8025789f8ec43351ce166f0a5f7ad0bb6f05b4c9515173d7f1cd619",
    "programs/piv1/src/state/timing.rs": "c9c1441bd3115ad60bae32c2564e2a6e1cdc29b104df9ff0905e3691e6111ac5",
    "programs/piv1/src/state_persistence.rs": "d26483f7e1b95a020623b85705ab0006cb884e85cff37a0666bafa83123401a5",
    "programs/piv1/src/constants.rs": "2498c79c42d732be84aae423cf31d82997574cc08cca02fb226fea9acaa24f94"}
SOURCE_PINS.update({
    "crates/piv1-math/src/lib.rs": "ea1ae37b4702ff652aeb1b604e8878583d1d7c6e838d33a10a290aaecad0ba1d",
    "programs/piv1/src/state/distribution.rs": "a291d3fb94921cd036161ee669190074fd5f159fe0d9f523a98ef3b0fa25d3a3",
    "programs/piv1/src/genesis_initialization.rs": "4fcc60c7592125da1bda2a549f94933f26eff5f960be1ea5fb98f51350ed879b",
    "programs/piv1/src/guardian_operations.rs": "a7ae2d0f25dba8db9d50dbaea6ab3b3d20b065b990f486f4a33e5152cd6a351b",
    "programs/piv1/src/guardian_clock_accounts.rs": "5cc36b79d429bb3e1818466d429259474c8a26d1c1288da16a71f22033f84946"})
# Retained primary-source identities, independently reviewed before this module.
# They are provenance references, not new runtime filesystem dependencies.
SQUADS_SOURCE_PINS = {
    "proposal.rs": "8d359e950e53280d398c8a1c8e63fbaeae9dcf458c2a2eb86ae4713f53dc07a0",
    "proposal_create.rs": "05346764eb681e23c6cea92390628d98c4c8d815247b4ae2667124cf27fdd585",
    "proposal_vote.rs": "bd5d7a9840752fe646a416cbaf9f5921ad44a7d3898945d503a112b29f4ea8f4",
    "vault_transaction_execute.rs": "ac9161ec101bc96a13e32a329f69734823df478d308718918f4f8608a5376d97"}


class Stop(ValueError): pass


def need(ok, reason):
    if not ok: raise Stop(reason)


def sha(raw): return hashlib.sha256(raw).hexdigest()


def verify_sources(repo):
    for relative, expected in SOURCE_PINS.items():
        path = Path(repo) / relative
        need(not path.is_symlink() and sha(path.read_bytes()) == expected, "startup_rule_source_pin")


def shape(c, j, rule):
    need(type(rule) is dict and set(rule) == {"startupRule", "lamports"} and rule["startupRule"] in KINDS, "startup_rule_shape")
    j.u64(rule["lamports"])


def clock(c, account, context, floor):
    need(type(account) is dict and account["exists"] is True and account["owner"] == SYSVAR and account["executable"] is False and c.u64(account["lamports"], True) > 0, "startup_clock_identity")
    raw = c.b64(account["dataBase64"], 40, True)
    need(len(raw) == 40, "startup_clock_size")
    slot, epoch_start, epoch, leader, timestamp = struct.unpack("<QqQQq", raw)
    need(slot <= int(context) and min(slot, int(context)) >= int(floor), "startup_clock_context")
    return {"contextSlot": str(context), "slot": str(slot), "unixTimestamp": str(timestamp),
            "dataBase64": account["dataBase64"], "dataSha256": sha(raw)}


def window(pre, post, inclusion_slot):
    need(pre is not None and post is not None, "startup_clock_window_missing")
    saved_clock(pre); saved_clock(post)
    need(int(pre["slot"]) <= int(pre["contextSlot"]) <= int(inclusion_slot) <= int(post["slot"]) <= int(post["contextSlot"]) and int(pre["unixTimestamp"]) <= int(post["unixTimestamp"]), "startup_clock_window_regression")
    return int(pre["unixTimestamp"]), int(post["unixTimestamp"])


def saved_clock(value):
    need(type(value) is dict and set(value) == {"contextSlot","slot","unixTimestamp","dataBase64","dataSha256"}, "startup_saved_clock")
    raw = base64.b64decode(value["dataBase64"],validate=True)
    need(len(raw) == 40 and base64.b64encode(raw).decode() == value["dataBase64"] and sha(raw) == value["dataSha256"], "startup_saved_clock")
    slot,_,_,_,timestamp = struct.unpack("<QqQQq",raw)
    context = value["contextSlot"]
    need(type(context) is str and re.fullmatch(r"0|[1-9][0-9]{0,19}",context) is not None and slot <= int(context) < 1<<64 and value["slot"] == str(slot) and value["unixTimestamp"] == str(timestamp), "startup_saved_clock")


def period(anchor, timestamp):
    need(I64_MIN <= anchor <= timestamp <= I64_MAX and timestamp - anchor <= I64_MAX, "startup_period_elapsed")
    index = (timestamp - anchor) // PERIOD
    start = anchor + index * PERIOD; end = start + PERIOD
    need(I64_MIN <= start < end <= I64_MAX and start <= timestamp < end, "startup_period_overflow")
    return index


def seal_binding(state):
    seal = state.get("seal")
    need(seal is not None and seal.get("stateBinding") is not None, "startup_state_seal_required")
    binding = seal["stateBinding"]; plan = binding["plan"]; manifest = binding["postManifest"]["initializer"]
    need(plan["program"] == manifest["programId"] == PROGRAM and plan["programData"] == PROGRAM_DATA and plan["vault"] == VAULT, "startup_selected_identity")
    need(manifest["initialization"] == {"initiallyPaused": True, "anchorUnixTimestamp": seal["anchorUnixTimestamp"]}, "startup_sealed_anchor")
    need(len(plan["guardianSlots"]) == 6 and len(plan["alt"]["addresses"]) == 16 and len(binding["targetBumps"]) == 16, "startup_seal_sizes")
    return binding, plan, manifest


def target_bytes(c, state):
    binding, plan, manifest = seal_binding(state)
    roles = {x["role"]: x["address"] for x in plan["native"]["roles"]}
    bumps = binding["targetBumps"]; key = lambda value: c.b58decode(value, 32)
    cfg = bytes([98,115,11,164,170,207,163,20,1,1,1])
    cfg += bytes([bumps[0], binding["authorityBump"], *[bumps[i] for i in (1,14,15,9,10,11,12,13,2)]])
    addresses = [roles[name] for name in ("pool_program", "pool", "validator_list", "reserve", "mint", "token_program")]
    addresses += ["Stake11111111111111111111111111111111111111", SYSTEM, roles["manager_fee"], manifest["protocol"]["referrer"], plan["pivAuthority"]]
    addresses += [roles[name] for name in ("distribution", "principal_token", "pending_token", "pending_sol", "principal_sol", "operational_sol", "distribution_escrow", "kif_sol", "htfp_recipient", "team_owner_recipient", "guardian_registry")]
    cfg += b"".join(map(key, addresses))
    cfg += struct.pack("<7H2q",10000,5900,1950,1950,200,1,1,864000,86400)
    cfg += bytes(2 + 19*8) + struct.pack("<qqQ",int(state["seal"]["anchorUnixTimestamp"]), PERIOD, 0) + bytes(64 + 16)
    distribution = bytes([104,51,125,187,226,55,209,99,1,bumps[1],1]) + bytes(880)
    registry = bytes([72,14,254,2,76,233,97,92,1,bumps[2]]) + bytes(8) + b"".join(map(key,plan["guardianSlots"]))
    rewards = [bytes([169,109,89,17,75,171,105,39,1,bumps[3+i],i]) + bytes(8) + key(guardian) + bytes(33) for i,guardian in enumerate(plan["guardianSlots"])]
    token = bytearray(165); token[:32] = key(roles["mint"]); token[32:64] = key(plan["pivAuthority"]); token[108] = 1
    raw = [cfg,distribution,registry,*rewards,*([b""]*5),bytes(token),bytes(token)]
    need([len(x) for x in raw] == plan["accountSizes"]["pivTargets"], "startup_target_lengths")
    owners = [PROGRAM]*9 + [SYSTEM]*5 + [TOKEN]*2
    return {address:(owner,data) for address,owner,data in zip(plan["alt"]["addresses"],owners,raw)}


def rent_sizes(state, phase):
    """Only frozen typed sizes; an actual RPC rent equality check occurs in SEND."""
    result = set()
    for group in ("expectedPrestate","expectedPoststate"):
        for address,rule in phase[group].items():
            kind = rule.get("startupRule")
            if kind is None: continue
            if kind == "PROGRAM_DATA":
                need(address == PROGRAM_DATA,"startup_program_data_identity"); size = 894597
            else:
                binding,plan,manifest = seal_binding(state)
                if kind == "PROPOSAL": need(address == plan["proposal"],"startup_proposal_identity"); size = 646
                else:
                    need(address in plan["alt"]["addresses"],"startup_target_identity")
                    index = plan["alt"]["addresses"].index(address); size = plan["accountSizes"]["pivTargets"][index]
                    if kind == "HEARTBEAT_REWARD_0": need(index == 3,"startup_heartbeat_role")
            need(rule["lamports"] == RENT_FLOORS.get(size),"startup_frozen_rent_floor")
            result.add(size)
    return sorted(result)


def facts_shape(facts):
    need(type(facts) is dict and set(facts) <= {"deploymentSlot", "proposal", "heartbeatPeriod"}, "startup_facts_shape")
    for name in ("deploymentSlot","heartbeatPeriod"):
        if name in facts:
            value = facts[name]
            need(type(value) is str and re.fullmatch(r"0|[1-9][0-9]{0,19}",value) is not None and (1 if name == "deploymentSlot" else 0) <= int(value) < 1<<64, "startup_"+name+"_fact")
    if "proposal" in facts:
        p = facts["proposal"]
        need(type(p) is dict and set(p) == {"timestamp", "status", "approvedCount"}, "startup_proposal_fact")
        need(type(p["timestamp"]) is str and re.fullmatch(r"0|-?[1-9][0-9]{0,18}",p["timestamp"]) is not None and I64_MIN <= int(p["timestamp"]) <= I64_MAX and type(p["approvedCount"]) is int and 0 <= p["approvedCount"] <= 4, "startup_proposal_fact")
        need(type(p["status"]) is int and p["status"] in ((1,) if p["approvedCount"] < 4 else (3,5)), "startup_proposal_fact")


def proposal_stage(phase, stage, facts):
    if phase["kind"] == "INITIALIZER":
        index = phase["initializerIndex"]
        need(3 <= index <= 8, "startup_proposal_phase")
        count = index - 3 - (1 if stage == "PRE" else 0)
        if index == 8: count = 4
        need(count >= 0, "startup_proposal_must_be_absent")
        status = 5 if index == 8 and stage == "POST" else 3 if count == 4 else 1
        return count, status, stage == "POST" and index in (3,7,8)
    need(phase["kind"] == "HEARTBEAT" and facts.get("proposal",{}).get("status") == 5, "startup_proposal_phase")
    return 4, 5, False


def transition(state, phase, next_facts, slot, pre_clock, post_clock):
    """Replay only the four source-derived facts that this exact phase may write."""
    facts_shape(next_facts); expected = copy.deepcopy(state.get("runtimeFacts",{}))
    if any(rule.get("startupRule") in ("PROPOSAL","HEARTBEAT_REWARD_0") for group in ("expectedPrestate","expectedPoststate") for rule in phase[group].values()):
        window(pre_clock,post_clock,slot)
    for rule in phase["expectedPoststate"].values():
        kind = rule.get("startupRule")
        if kind == "PROGRAM_DATA" and phase["kind"] == "BOOTSTRAP":
            need("deploymentSlot" not in expected, "startup_deployment_replay")
            expected["deploymentSlot"] = str(slot)
        elif kind == "PROPOSAL":
            count,status,new = proposal_stage(phase,"POST",state.get("runtimeFacts",{}))
            need("proposal" in next_facts, "startup_proposal_fact_missing")
            value = next_facts["proposal"]; prior = state.get("runtimeFacts",{}).get("proposal")
            if new:
                low,high = window(pre_clock,post_clock,slot)
                need(low <= int(value["timestamp"]) <= high, "startup_proposal_timestamp_window")
                if phase["initializerIndex"] == 3: need(prior is None,"startup_proposal_create_replay")
                else: need(prior is not None and prior["approvedCount"] == (3 if phase["initializerIndex"] == 7 else 4) and prior["status"] == (1 if phase["initializerIndex"] == 7 else 3) and int(value["timestamp"]) >= int(prior["timestamp"]),"startup_proposal_prior_fact")
            else:
                need(prior is not None and value["timestamp"] == prior["timestamp"],"startup_proposal_timestamp_changed")
                previous_count = count - 1 if phase["kind"] == "INITIALIZER" else count
                need(prior["approvedCount"] == previous_count and prior["status"] == status,"startup_proposal_prior_fact")
            expected["proposal"] = {"timestamp":value["timestamp"],"approvedCount":count,"status":status}
        elif kind == "HEARTBEAT_REWARD_0":
            low,high = window(pre_clock,post_clock,slot)
            first = period(int(state["seal"]["anchorUnixTimestamp"]),low); last = period(int(state["seal"]["anchorUnixTimestamp"]),high)
            need(first == last,"startup_heartbeat_ambiguous_period")
            expected["heartbeatPeriod"] = str(first)
    need(next_facts == expected,"startup_runtime_facts_transition")


def verify(c, state, phase, stage, expected, accounts, *, inclusion_slot=None, pre_clock=None, post_clock=None):
    """Return complete next facts only after every declared typed account matches."""
    facts = copy.deepcopy(state.get("runtimeFacts", {})); facts_shape(facts)
    for address, rule in expected.items():
        if "startupRule" not in rule: continue
        a = accounts[address]
        need(a["exists"] is True and a["executable"] is False, "startup_account_identity")
        raw = c.b64(a["dataBase64"], c.MAX_ACCOUNT, True)
        need(rule["lamports"] == RENT_FLOORS.get(len(raw)) and a["lamports"] == rule["lamports"], "startup_frozen_rent_floor")
        kind = rule["startupRule"]
        if kind == "PROGRAM_DATA":
            need(address == PROGRAM_DATA and a["owner"] == LOADER and len(raw) == 894597, "startup_program_data_identity")
            if phase["kind"] == "BOOTSTRAP" and stage == "POST":
                need("deploymentSlot" not in facts and inclusion_slot is not None, "startup_deployment_replay")
                slot = int(inclusion_slot); facts["deploymentSlot"] = str(slot)
            else:
                need("deploymentSlot" in facts, "startup_deployment_fact_missing"); slot = int(facts["deploymentSlot"])
            need(raw[:45] == struct.pack("<IQB",3,slot,1) + c.b58decode(VAULT,32) and sha(raw[45:]) == ELF_SHA, "startup_program_data_bytes")
        elif kind == "INITIALIZED_TARGET":
            need(phase["kind"] == "INITIALIZER" and phase["initializerIndex"] == 8 and stage == "POST" or phase["kind"] == "HEARTBEAT", "startup_target_phase")
            targets = target_bytes(c,state)
            need(address in targets and (a["owner"],raw) == targets[address], "startup_target_bytes")
        elif kind == "PROPOSAL":
            binding, plan, manifest = seal_binding(state)
            need(address == plan["proposal"] and a["owner"] == SQUADS and len(raw) == 646, "startup_proposal_identity")
            count,status,new_timestamp = proposal_stage(phase,stage,facts)
            timestamp = struct.unpack_from("<q",raw,49)[0]
            if new_timestamp:
                low,high = window(pre_clock,post_clock,inclusion_slot)
                need(low <= timestamp <= high, "startup_proposal_timestamp_window")
                if phase["initializerIndex"] == 3: need("proposal" not in facts, "startup_proposal_create_replay")
                else:
                    prior = facts.get("proposal")
                    need(prior is not None and prior["approvedCount"] == (3 if phase["initializerIndex"] == 7 else 4) and prior["status"] == (1 if phase["initializerIndex"] == 7 else 3) and timestamp >= int(prior["timestamp"]), "startup_proposal_prior_fact")
            else:
                prior = facts.get("proposal"); need(prior is not None and prior["timestamp"] == str(timestamp), "startup_proposal_timestamp_changed")
                previous_count = count - 1 if stage == "POST" and phase["kind"] == "INITIALIZER" else count
                need(prior["approvedCount"] == previous_count and prior["status"] == status, "startup_proposal_prior_fact")
            approved = sorted(manifest["governance"]["approvers"][:count], key=lambda value:c.b58decode(value,32))
            body = hashlib.sha256(b"account:Proposal").digest()[:8] + c.b58decode(plan["multisig"],32) + struct.pack("<QBqB",int(manifest["governance"]["transactionIndex"]),status,timestamp,binding["proposalBump"])
            body += struct.pack("<I",count) + b"".join(c.b58decode(value,32) for value in approved) + bytes(8)
            need(raw == body + bytes(646-len(body)), "startup_proposal_bytes")
            if stage == "POST": facts["proposal"] = {"timestamp":str(timestamp),"status":status,"approvedCount":count}
        else:
            binding,plan,manifest = seal_binding(state)
            need(kind == "HEARTBEAT_REWARD_0" and phase["kind"] == "HEARTBEAT" and address == plan["alt"]["addresses"][3] and a["owner"] == PROGRAM, "startup_heartbeat_role")
            baseline = target_bytes(c,state)[address][1]
            if stage == "POST":
                low,high = window(pre_clock,post_clock,inclusion_slot)
                first = period(int(state["seal"]["anchorUnixTimestamp"]),low); last = period(int(state["seal"]["anchorUnixTimestamp"]),high)
                need(first == last, "startup_heartbeat_ambiguous_period")
                need("heartbeatPeriod" not in facts or first >= int(facts["heartbeatPeriod"]), "startup_heartbeat_regression")
                value = first; facts["heartbeatPeriod"] = str(value)
            else: value = int(facts["heartbeatPeriod"]) if "heartbeatPeriod" in facts else None
            data = baseline if value is None else baseline[:51] + b"\1" + struct.pack("<Q",value) + bytes(24)
            need(raw == data, "startup_heartbeat_bytes")
    facts_shape(facts)
    return facts


# Explicit candidate-only entrypoint. Legacy KINDS, facts and dispatch stay intact.
CANDIDATE_SCOPE = "piv1-testnet-4ccc-in02-unselected-v2"
CANDIDATE_PHASE_SCHEMA = "piv1-unselected-startup-phase-v2"
CANDIDATE_PROGRAM_DATA_RULE = "CANDIDATE_PROGRAM_DATA_V2"
CANDIDATE_ELF_SHA = "4ccc59ab4f37d2af98cf2dc3f2b4b1aaf701cb4061cd0315e378c0bae2b69cf2"


def candidate_program_data_rule():
    """Frozen preview rent, not a fresh quote or a caller-selected image policy."""
    return {"startupRule": CANDIDATE_PROGRAM_DATA_RULE, "lamports": "4534027000"}


def verify_candidate_program_data(c, phase, stage, expected, accounts, *,
                                  prior_fact=None, inclusion_slot=None):
    """Check one exact candidate ProgramData account and a separately scoped fact.

    This does not accept a legacy runtimeFacts deploymentSlot, prove creation
    lineage, or integrate a controller/seal/prestate. No signatures or I/O occur.
    """
    need(type(phase) is dict and type(phase.get("index")) is int, "candidate_phase_shape")
    index = phase["index"]
    need(935 <= index < 947, "candidate_program_data_phase")
    kind = ("BOOTSTRAP" if index == 935 else "RECIPIENTS" if index == 936
            else "INITIALIZER" if index <= 945 else "HEARTBEAT")
    init_index = index - 937 if 937 <= index <= 945 else None
    descriptor = {"schema": CANDIDATE_PHASE_SCHEMA, "candidateScope": CANDIDATE_SCOPE,
                  "id": f"candidate_{index:04}", "index": index, "kind": kind,
                  "initializerIndex": init_index, "writeOffset": None, "writeBytes": None}
    need(set(phase) == set(descriptor)
         and all(type(phase[key]) is type(value) and phase[key] == value
                 for key, value in descriptor.items()), "candidate_phase_shape")
    need(stage in ("PRE", "POST") and not (index == 935 and stage == "PRE"),
         "candidate_program_data_phase")
    need(type(expected) is dict and expected == {PROGRAM_DATA: candidate_program_data_rule()},
         "candidate_program_data_rule")
    need(type(accounts) is dict and set(accounts) == {PROGRAM_DATA}, "candidate_program_data_coverage")
    a = accounts[PROGRAM_DATA]
    need(type(a) is dict and set(a) == {"exists", "owner", "executable", "lamports", "dataBase64"}
         and a["exists"] is True and a["owner"] == LOADER and a["executable"] is False,
         "candidate_program_data_identity")
    raw = c.b64(a["dataBase64"], c.MAX_ACCOUNT, True)
    need(len(raw) == 892397, "candidate_program_data_identity")
    need(a["lamports"] == "4534027000", "candidate_frozen_rent_floor")
    if index == 935:
        need(prior_fact is None and inclusion_slot is not None, "candidate_deployment_replay")
        slot = c.u64(inclusion_slot, True)
        need(slot > 0, "candidate_deployment_slot")
        fact = {"candidateScope": CANDIDATE_SCOPE, "elfSha256": CANDIDATE_ELF_SHA,
                "deploymentSlot": str(slot)}
    else:
        need(type(prior_fact) is dict
             and set(prior_fact) == {"candidateScope", "elfSha256", "deploymentSlot"}
             and prior_fact["candidateScope"] == CANDIDATE_SCOPE
             and prior_fact["elfSha256"] == CANDIDATE_ELF_SHA, "candidate_deployment_fact")
        slot = c.u64(prior_fact["deploymentSlot"], True)
        need(slot > 0, "candidate_deployment_slot")
        fact = copy.deepcopy(prior_fact)
    need(raw[:45] == struct.pack("<IQB", 3, slot, 1) + c.b58decode(VAULT, 32)
         and sha(raw[45:]) == CANDIDATE_ELF_SHA, "candidate_program_data_bytes")
    return fact
