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


# Supplied runtime components only. This is not a durable or authenticated lineage.
CANDIDATE_RUNTIME_SCHEMA = "piv1-candidate-runtime-component-facts-v2"
CANDIDATE_RUNTIME_ORACLE_SHA = "e3ca98bbc901dce6da6d1894c25a34b2649cc0d9bd19c226a11cb1ed10c3b868"
CANDIDATE_RUNTIME_FLAGS = {"liveReady": False, "authorized": False, "controllerReady": False,
    "completeStateOracle": False, "fullSnapshotVerified": False,
    "completePrerequisitesAuthenticated": False, "fundingAuthenticated": False,
    "sourceAuthenticated": False, "freshnessAuthenticated": False,
    "controlledSuccessor": False, "signingOrSendPerformed": False}


def _candidate_runtime_sources():
    import importlib.util
    import json
    repo = Path(__file__).resolve().parent.parent
    path = repo / "tools/startup_manifest.candidate-oracle.json"
    need(not path.is_symlink(), "candidate_runtime_oracle_pin")
    raw = path.read_bytes()
    need(sha(raw) == CANDIDATE_RUNTIME_ORACLE_SHA, "candidate_runtime_oracle_pin")
    ref = json.loads(raw)
    path = repo / "tools/startup_public_journal.py"
    need(not path.is_symlink(), "candidate_runtime_journal_pin")
    journal_raw = path.read_bytes()
    need(sha(journal_raw) == "c492c1fae58caeb88f8fbe137251440626aeb051514b3f6157fefce1a0155b48", "candidate_runtime_journal_pin")
    spec = importlib.util.spec_from_file_location("runtime_candidate_binding", path)
    j = importlib.util.module_from_spec(spec); exec(compile(journal_raw,str(path),"exec",dont_inherit=True),j.__dict__)
    need(j.candidate_pins().resolve() == repo, "candidate_runtime_repo")
    verify_sources(repo)
    return ref,j


def _candidate_runtime_phase(phase):
    need(type(phase) is dict and type(phase.get("index")) is int, "candidate_runtime_phase")
    i = phase["index"]; need(935 <= i <= 946, "candidate_runtime_phase")
    kind = "BOOTSTRAP" if i == 935 else "RECIPIENTS" if i == 936 else "INITIALIZER" if i <= 945 else "HEARTBEAT"
    expected = {"schema": CANDIDATE_PHASE_SCHEMA, "candidateScope": CANDIDATE_SCOPE,
        "id": f"candidate_{i:04}", "index": i, "kind": kind,
        "initializerIndex": i-937 if 937 <= i <= 945 else None, "writeOffset": None, "writeBytes": None}
    need(set(phase) == set(expected) and all(type(phase[k]) is type(v) and phase[k] == v for k,v in expected.items()), "candidate_runtime_phase")
    return i


def _candidate_runtime_facts(value, c):
    empty = {"schema": CANDIDATE_RUNTIME_SCHEMA, "candidateScope": CANDIDATE_SCOPE,
        "elfSha256": CANDIDATE_ELF_SHA, "oracleSha256": CANDIDATE_RUNTIME_ORACLE_SHA,
        "initializerBindingState": "UNBOUND_TO_INITIALIZER", "initializerBindingSha256": None,
        "deploymentFact": None, "proposal": None, "heartbeatPeriod": None,
        "lastRuntimePhase": None, "lastInclusionSlot": None, "lastPostClock": None,
        "runtimePrerequisiteFloor": None, "pendingPhase": None, "pendingPreClock": None}
    if value is None: return empty
    need(type(value) is dict and set(value) == set(empty), "candidate_runtime_facts_scope")
    need(all(type(value[k]) is type(empty[k]) and value[k] == empty[k] for k in ("schema","candidateScope","elfSha256","oracleSha256")), "candidate_runtime_facts_scope")
    result = copy.deepcopy(value); last = result["lastRuntimePhase"]; pending = result["pendingPhase"]
    need(last is None or type(last) is int and 935 <= last <= 946, "candidate_runtime_cursor")
    need(pending is None or type(pending) is int and pending == (935 if last is None else last+1) and pending <= 946, "candidate_runtime_cursor")
    need((pending is None) == (result["pendingPreClock"] is None), "candidate_runtime_pending_clock")
    if pending is not None: saved_clock(result["pendingPreClock"])
    if last is None:
        need(all(result[k] is None for k in ("deploymentFact","proposal","heartbeatPeriod","lastInclusionSlot","lastPostClock","runtimePrerequisiteFloor")), "candidate_runtime_prefix")
    else:
        saved_clock(result["lastPostClock"])
        slot = c.u64(result["lastInclusionSlot"], True)
        need(slot > 0 and slot <= int(result["lastPostClock"]["slot"]), "candidate_runtime_prior_slot")
        p = result["deploymentFact"]
        need(type(p) is dict and set(p) == {"candidateScope","elfSha256","deploymentSlot"} and p["candidateScope"] == CANDIDATE_SCOPE and p["elfSha256"] == CANDIDATE_ELF_SHA and 0 < c.u64(p["deploymentSlot"],True) <= slot, "candidate_runtime_deployment_fact")
        facts_shape({k:result[k] for k in ("proposal","heartbeatPeriod") if result[k] is not None})
        if last < 940: need(result["proposal"] is None, "candidate_runtime_proposal_order")
        else:
            p = result["proposal"]; count = min(last-940,4); status = 5 if last >= 945 else 3 if count == 4 else 1
            need(type(p) is dict and p["approvedCount"] == count and p["status"] == status, "candidate_runtime_proposal_order")
            need(int(p["timestamp"]) <= int(result["lastPostClock"]["unixTimestamp"]), "candidate_runtime_proposal_future")
        need((result["heartbeatPeriod"] is not None) == (last == 946), "candidate_runtime_heartbeat_order")
        if last < 936: need(result["runtimePrerequisiteFloor"] is None, "candidate_runtime_prerequisite_order")
        else:
            need(0 < c.u64(result["runtimePrerequisiteFloor"],True) <= int(result["lastPostClock"]["contextSlot"]), "candidate_runtime_prerequisite_floor")
    if pending is not None and last is not None:
        pre,post = result["pendingPreClock"],result["lastPostClock"]
        need(int(pre["slot"]) >= int(post["contextSlot"]) and int(pre["contextSlot"]) >= int(post["contextSlot"]) and int(pre["unixTimestamp"]) >= int(post["unixTimestamp"]), "candidate_runtime_pending_clock_regression")
    bound = last is not None and last >= 937 or pending == 937 or pending is not None and pending > 937
    if bound:
        need(result["initializerBindingState"] == "BOUND_AT_INITIALIZER_PRE" and type(result["initializerBindingSha256"]) is str and re.fullmatch("[0-9a-f]{64}",result["initializerBindingSha256"]) is not None, "candidate_runtime_binding_commitment")
    else:
        need(result["initializerBindingState"] == "UNBOUND_TO_INITIALIZER" and result["initializerBindingSha256"] is None, "candidate_runtime_early_binding")
    return result


def verify_candidate_runtime(c, phase, stage, accounts, *, prior_runtime=None,
                             clock_binding=None, context_slot=None, inclusion_slot=None):
    """Verify only supplied Clock/ProgramData/proposal/targets/heartbeat components.

    Ordered PRE/POST component facts are not authenticated execution receipts or a
    completed parent. No whole ledger, funding, TLS, signature or durable admission.
    """
    i = _candidate_runtime_phase(phase)
    need(type(stage) is str and stage in ("PRE","POST"), "candidate_runtime_stage")
    need(i >= 937 or clock_binding is None, "candidate_runtime_early_binding")
    ref,j = _candidate_runtime_sources(); r = ref["roles"]
    facts = _candidate_runtime_facts(prior_runtime,c)
    if stage == "PRE":
        need(facts["pendingPhase"] is None and i == (935 if facts["lastRuntimePhase"] is None else facts["lastRuntimePhase"]+1), "candidate_runtime_pre_order")
        need(inclusion_slot is None, "candidate_runtime_pre_inclusion")
    else: need(prior_runtime is not None and facts["pendingPhase"] == i, "candidate_runtime_post_order")
    expected_keys = {CLOCK,PROGRAM_DATA}
    if i >= 940: expected_keys.add(r["proposal1"])
    if i >= 945: expected_keys.update(ref["targetAddresses"])
    need(type(accounts) is dict and set(accounts) == expected_keys, "candidate_runtime_account_set")
    for a in accounts.values():
        need(type(a) is dict and set(a) == {"exists","owner","executable","lamports","dataBase64"}, "candidate_runtime_account_shape")
    context = c.u64(context_slot,True)
    floor = int(ref["baselineContextSlot"])
    if facts["lastPostClock"] is not None: floor = max(floor,int(facts["lastPostClock"]["contextSlot"]))
    actual_clock = clock(c,accounts[CLOCK],context,floor)
    if facts["lastPostClock"] is not None:
        need(int(actual_clock["unixTimestamp"]) >= int(facts["lastPostClock"]["unixTimestamp"]), "candidate_runtime_clock_regression")
    binding = None
    if i >= 937:
        need(clock_binding is not None, "candidate_runtime_binding_required")
        rebuilt = j.validate_candidate_clock_binding(clock_binding)
        plan = rebuilt["unsignedInitializerPlan"]
        need(plan["alt"]["addresses"] == ref["targetAddresses"] and plan["accountSizes"] == ref["accountSizes"], "candidate_runtime_binding_policy")
        digest = sha(j.encode(rebuilt))
        if i == 937 and stage == "PRE":
            need(facts["lastRuntimePhase"] == 936 and facts["initializerBindingSha256"] is None, "candidate_runtime_seal_boundary")
            prerequisite = int(facts["runtimePrerequisiteFloor"])
            need(int(rebuilt["clockDecoded"]["slot"]) >= prerequisite and int(rebuilt["clock"]["contextSlot"]) >= prerequisite and int(rebuilt["clockDecoded"]["unixTimestamp"]) >= int(facts["lastPostClock"]["unixTimestamp"]), "candidate_runtime_binding_before_prerequisite")
            need(rebuilt["clock"]["contextSlot"] == context_slot and rebuilt["clock"]["dataBase64"] == accounts[CLOCK]["dataBase64"], "candidate_runtime_binding_preclock")
            facts["initializerBindingState"] = "BOUND_AT_INITIALIZER_PRE"; facts["initializerBindingSha256"] = digest
        else: need(facts["initializerBindingSha256"] == digest, "candidate_runtime_binding_changed")
        if facts["proposal"] is not None:
            need(int(facts["proposal"]["timestamp"]) >= int(rebuilt["clockDecoded"]["unixTimestamp"]), "candidate_runtime_proposal_before_anchor")
        binding = {"plan":plan,"postManifest":rebuilt["postManifest"],**ref["bumps"]}
    if stage == "POST":
        slot = c.u64(inclusion_slot,True)
        need(slot > 0 and (facts["lastInclusionSlot"] is None or slot > int(facts["lastInclusionSlot"])), "candidate_runtime_slot_order")
        window(facts["pendingPreClock"],actual_clock,slot)
    if i == 935 and stage == "PRE":
        need(accounts[PROGRAM_DATA] == {"exists":False,"owner":None,"executable":None,"lamports":None,"dataBase64":None}, "candidate_runtime_programdata_preexists")
    else:
        facts["deploymentFact"] = verify_candidate_program_data(c,phase,stage,
            {PROGRAM_DATA:candidate_program_data_rule()},{PROGRAM_DATA:accounts[PROGRAM_DATA]},
            prior_fact=facts["deploymentFact"],inclusion_slot=inclusion_slot)
    local = {k:facts[k] for k in ("proposal","heartbeatPeriod") if facts[k] is not None}
    view = {"runtimeFacts":local,"seal":None if binding is None else {"stateBinding":binding,"anchorUnixTimestamp":rebuilt["clockDecoded"]["unixTimestamp"]}}
    rules = {}
    if i == 940 and stage == "PRE":
        need(accounts[r["proposal1"]] == {"exists":False,"owner":None,"executable":None,"lamports":None,"dataBase64":None}, "candidate_runtime_proposal_preexists")
    elif i >= 940: rules[r["proposal1"]] = {"startupRule":"PROPOSAL","lamports":ref["rents"]["646"]}
    if i == 945 and stage == "PRE":
        need(all(accounts[a] == {"exists":False,"owner":None,"executable":None,"lamports":None,"dataBase64":None} for a in ref["targetAddresses"]), "candidate_runtime_target_preexists")
    elif i >= 945:
        for index,address in enumerate(ref["targetAddresses"]):
            rules[address] = {"startupRule":"HEARTBEAT_REWARD_0" if i == 946 and index == 3 else "INITIALIZED_TARGET","lamports":ref["rents"][str(ref["accountSizes"]["pivTargets"][index])]}
    legacy_phase = {**phase,"expectedPrestate":rules if stage == "PRE" else {},"expectedPoststate":rules if stage == "POST" else {}}
    next_local = verify(c,view,legacy_phase,stage,rules,accounts,inclusion_slot=inclusion_slot,
        pre_clock=facts["pendingPreClock"],post_clock=actual_clock if stage == "POST" else None)
    if stage == "POST":
        transition(view,legacy_phase,next_local,slot,facts["pendingPreClock"],actual_clock)
        facts.update({k:next_local.get(k) for k in ("proposal","heartbeatPeriod")})
        facts.update(lastRuntimePhase=i,lastInclusionSlot=str(slot),lastPostClock=actual_clock,pendingPhase=None,pendingPreClock=None)
        if i == 936: facts["runtimePrerequisiteFloor"] = str(max(slot,int(actual_clock["contextSlot"])))
    else: facts.update(pendingPhase=i,pendingPreClock=actual_clock)
    _candidate_runtime_facts(facts,c)
    return {"status":"VERIFIED_SUPPLIED_CANDIDATE_RUNTIME_COMPONENTS_ONLY","candidateScope":CANDIDATE_SCOPE,
        "phase":phase["id"],"stage":stage,"checkedAccountKeys":sorted(expected_keys),"clock":actual_clock,
        "runtimeFacts":facts,"runtimePredicateVerification":True,
        "evidenceScope":"SUPPLIED_HOST_COMPONENT_BYTES_NOT_AUTHENTICATED_DURABLE_EXECUTION",**CANDIDATE_RUNTIME_FLAGS}
