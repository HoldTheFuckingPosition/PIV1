# Local native bootstrap and newly deployed heartbeat — bounded success

**LOCAL PASS / BOOTSTRAP RECIPE UNSELECTED / PUBLIC PACKAGE NOT READY.** D-048's
conditional deployment permission remains recorded with **READINESS UNMET / NO LIVE
TRIGGER**. The [single M5 register](PIV1_M5_TESTNET_READINESS.md) owns readiness.

Root performs one guarded host build and one positive local Bank run; both pass,
with no diagnostics or guard stop. The isolated test executes two successful local
transactions: native atomic bootstrap 1,035 bytes/six zero signature slots, then
production heartbeat 601 bytes/one zero slot at slot 2. Root and separate actual
audits verify 171 evidence records, 159 account records, four complete fresh maps,
all native roles/global privileges and exactly one System CPI at outer 7/stack 2.
Rust ordering and the local blockhash differ from archived JavaScript wire bytes;
semantic equivalence is verified, not raw archived-message replay.

Program and ProgramData are genuinely absent from genesis. No PIV precache, load
or state correction substitutes for deployment. The top-level native loader verifies
the unchanged 894,552-byte retained ELF at actual public ID
`GAjHRaC3XM98SoRoFy5B8P2ypTS1ndf1UPhXmjWgtPux`. Fresh full reads verify executable
Program (36 bytes), canonical loader-owned ProgramData (894,597 bytes), slot 1/Some(vault 0), every ELF
byte and SHA-256 `e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99`.
The committed final authority is `A2UHpGyiCBdUZDMrofGswNiyS4ATD1ytBSiAsWNdxUTw`;
Buffer drains to the same H2 and is purged after zero lamports. The local native
authority change did execute; **no LIVE authority operation or deployment occurred**.

The later-slot heartbeat invokes the newly deployed production program using
source-valid synthetic Config/registry/reward fixtures. Only G0's 5,000 local fee
and reward 0's last-active Option tag change: None→Some(period0). Full other state,
money/debt ledgers, remaining guardians, MS bytes/index and deployed Program/PD are
preserved. No Squads invocation, proposal, vote or membership-policy enforcement
occurs. Child-slot Clock/SlotHistory/SlotHashes changes are checked separately from
transaction deltas; no independent parent Bank hash computation is claimed.

Measured local bootstrap/heartbeat usage is 5,940/59,920 CU and 895,430/897,649
loaded bytes; local fees are 30,000/5,000, total 35,000. Requested caps remain
1,400,000 CU, 262,144 heap bytes and 64 MiB loaded data. Same-H2 fixture advance
4,545,162,360 from 4,696,464,320 leaves 151,301,960 before bootstrap; native credit
and Program/PD rent leave 150,428,200 afterward. G0 999,980,000 becomes 999,945,000.
These are **local synthetic balances**, not refreshed live wallets or a complete
project-funding proof. Upload/initializer/recipient/operational/test-capital costs
and the prior retained upload fee projection are separate from these two local fees.

This closes the bounded local loader verification plus deployed-entrypoint gap.
It does not prove upload: exact Buffer with Some(G0) bytes were preloaded, with rent debited
from the same H2. Application fixtures have separate synthetic capital; initializer
creation/full lifecycle are unproved. All seven signature slots are zero and local
Bank entries bypass Ed25519 verification: no real key control, actual endorsements,
independent custody or Squads votes are established. Clock 100/anchor 0, Rent 5080/
threshold1/burn50 and named GetSysvar/V2/V3 features describe local Bank 4.2, not an
exact replay of retained observed Testnet 4.4 or its whole feature set.

G0 is temporary initial authority inside the atomic sequence; final vault 0 is
verified at commit. The bootstrap trust recipe and artifact selection remain open
for consolidated readiness/risk review. Native Deploy binds Buffer address, not an
ELF hash; G0 can write it after review until inclusion. This test supplies no native
hash/current-membership/global threshold or alternate-message prevention. Confirmed
final 4/6 governance and accepted economics stay unchanged. No new routine founder
decision is requested. The failed direct loader-CPI recipe remains
**INVALID_DIRECT_DEPLOY_CPI_DO_NOT_EXECUTE**; its immutable files/evidence are not
rewritten or rerun by this different positive native test.

Next prepare the exact remaining governed initializer/trust/version/resource and
complete funding/recipient package, including the Clock anchor frozen before votes,
then the separate M4 risk/scope and final readiness review. M4 stays OPEN_UNWAIVED;
stopped operations/drafts and consumed scopes remain untouched. There is no RPC,
network refresh, private access, new key, signing, live funds, live handover or live
deployment here. One host harness build is new; no PIV rebuild/dependency/toolchain/
ELF change occurred and 198 retained inputs match. Main 824342d/protected 3677fee and
the 2 GiB reserve stay unchanged; root owns reviewed integration publication.
See [the exact public evidence index](PIV1_TESTNET_ATOMIC_BOOTSTRAP_RUNTIME.json).
