# Proposed governed loader-v3 deployment recipe

**PROPOSED, NOT APPROVED. No buffer handover, funding or deployment is authorized.**
The [single M5 register](PIV1_M5_TESTNET_READINESS.md) owns readiness. The
[JSON summary](PIV1_TESTNET_GOVERNED_LOADER_PROPOSAL.json) is non-executable;
all existing selected package inputs and artifact selection remain unchanged.
This extends the [unapproved governance proposal](PIV1_TESTNET_GOVERNANCE_ALT_BUDGET_PROPOSAL.md).

The retained candidate is 894,552 bytes, SHA-256
`e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99`.
Production ABI 3.0.0 and retained loader 4.2.0 source agree on the relevant variants
and 36/37/45-byte layouts. Actual loader Cargo metadata uses interface **7.0.0**;
the initial 5.0 premise was corrected from source before helper execution. Source
provenance checks bind archives/extracted files; no Testnet implementation equality,
new build, successful runtime or deployment compatibility is inferred.

1. Propose a loader Buffer derived with `create_with_seed`: G0 base, public seed
   `piv1-testnet-e29735e1`, loader owner, address
   `HGP9XhQmoHGCUbgoSATZ19zZHonxWB3uo1hKF3SK3M6K`. No new key is needed.
   H2 pays for 894,589 bytes. Create and InitializeBuffer(authority G0) are atomic.
2. G0 writes the full retained ELF with raw loader Write packets. Complete legacy
   sizing includes every signature slot and three explicit resource requests.
   Maximum payload is 956 bytes (1,232-byte packet); 957 needs 1,233 bytes.
   There are 936 writes; the last is offset 893,860, length 692, packet 968 bytes.
   Root and independent review reconstruct every chunk and the exact full hash.
3. **Before final verification and votes**, separately authorize and confirm a
   **buffer-only** SetAuthority from G0 to the proposed Squads vault0. It uses the
   supported unchecked variant: old G0 signs; the exact new vault PDA is readonly
   and cannot sign at top level. This sensitive handover is unapproved. It is not
   a PROGRAM upgrade-authority transfer or hot bootstrap deployment.
4. After handover, freshly verify loader owner, Buffer header/authority, exact
   length and **all ELF bytes/hash**, then store/propose and collect four distinct
   current-member approvals. G0 alone must have lost write authority before votes.
   Deploy names a buffer address, not a hash: there is no onchain hash guard.
   Valid 4/6 governance can still mutate/reclaim the buffer or later upgrade;
   it is not immutable. Cancellation/reclaim needs separate 4/6 approval and scope.
5. Recheck bytes, authority, all relevant accounts and current governance before
   execution. The final outer transaction atomically creates the 36-byte Program
   account and executes the stored loader deployment through Squads. Never allocate
   Program in an earlier transaction, which would expose an initialization race.
   Proposed upgrade authority is Squads vault0 **from first deployment**.

Direct Squads creation fits without an ALT or Squads upload buffer: native Deploy
is 12 bytes/eight accounts; stored compact is 318 bytes, create payload 333 and
stored account 419. Final atomic packet is 815 bytes with G0, H2 and the existing
PIV program identity as three real outer signers. Squads supplies only the vault
CPI signature. Proposed G0..G3 approvals are not actual votes or custody proof.

| Phase | Full bytes | Zero signature slots |
| --- | ---: | ---: |
| Buffer create + initialize | 510 | 2 |
| Each full write / last write | 1,232 / 968 | 1 |
| Buffer-only handover | 296 | 1 |
| Stored transaction create | 756 | 2 |
| Active proposal create | 439 | 2 |
| G0 approval / each G1..G3 approval | 301 / 397 | 1 / 2 |
| Atomic Program create + execute | 815 | 3 |

Each alternative totals 945 packets and 953 zero signature slots. Requested caps
are 1,400,000 CU, 262,144 heap bytes and 67,108,864 loaded-account bytes per packet.
They are sizing requests, not measured use, live sufficiency or the initializer's
8-MiB profile. Context/index 1 is counterfactual; fresh exact indices/lifetimes
remain open. No complete wires or executable manifest are published in this summary.

| Conditional H2 cashflow, lamports | 1x capacity | 2x capacity |
| --- | ---: | ---: |
| ProgramData max bytes of ELF | 894,552 | 1,789,104 |
| Buffer allocation, initially funded | 4,545,162,360 | 4,545,162,360 |
| Stored transaction + proposal allocations | 6,710,680 | 6,710,680 |
| Program36 allocation in final atomic transaction | 833,120 | 833,120 |
| Buffer credit back to SAME H2, before ProgramData creation | 4,545,162,360 | 4,545,162,360 |
| ProgramData allocation | 4,545,203,000 | 9,089,527,160 |
| Conditional peak H2 outflow / final held total | **4,552,746,800** | **9,097,070,960** |
| With optional 650,240 H2 reserve | 4,553,397,040 | 9,097,721,200 |

The optional reserve is a budget choice, not a loader requirement. These source-flow
models assume zero prefunds and exact proposed actors; actual wallet funding/peak
remains unknown. Loader credits H2 before paying ProgramData, so adding Buffer and
ProgramData as net cost would double-count reused funds. This is separate from the
initializer's later G0-only buffer refund and recipient/operational/test-capital budgets.
G0's fee projection is **4,765,000 lamports**, using retained 5,000/signature pricing;
these new messages have **no fresh fee quote or cap**. All other rent prices are
retained; the helper projected R419=2,778,760 from the retained linear observations.
A separate fresh root rent419 quote confirms that value; it does not refresh all rents.

Root made only three new read-only official Testnet requests: genesis, three known
feature accounts, and rent419. At finalized context 454824983 the named old-SBPF
deployment-disable account was absent; V2/V3 feature accounts were present/active.
Candidate flags0 denotes SBPFv0. Pinned local source rejects old SBPF if that named
gate is active. Its observed absence does not establish full current runtime/ELF
compatibility or deployability. No wallet snapshot, fresh fee query, simulation,
artifact execution, build, Bank/SBF or deployment occurred. Public evidence hashes
bind the actual offline run and independent wire/cashflow/feature reviews.

All capacity/artifact/authority choices remain unaccepted. No new keys/private reads,
signing, funds or authority operation occurred. D-039 covers preparation; D-041,
D-043 and D-045 remain consumed. M4 stays open/unwaived with stopped drafts untouched;
M5 is preparation/public NOT READY/deployment NOT AUTHORIZED. Root publishes reviewed
integration only; main `824342d` and protected `3677fee` stay unchanged. Preserve the
2-GiB reserve and all evidence. Exact future funding and execution need their own scope.
