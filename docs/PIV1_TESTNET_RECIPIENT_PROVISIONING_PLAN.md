# Unsigned recipient provisioning proposal

**ARCHIVED EXPIRED — DO NOT SIGN OR SEND. NO FUNDING IS AUTHORIZED.**
The [public proposal](PIV1_TESTNET_RECIPIENT_PROVISIONING_PLAN.json) preserves the
complete raw helper result, message/wire bytes and hashes unchanged. The
[single M5 register](PIV1_M5_TESTNET_READINESS.md) owns readiness. D-039 permits
preparation; D-046 accepted recipient purposes; consumed D-047 grants no new action.
There is no new decision or acceptance of this provisioning packet or its fee cap.

One unsigned **atomic transaction** contains exactly two System transfers:

| Accepted destination | Vault index | Snapshot balance | Proposed credit, lamports |
| --- | ---: | ---: | ---: |
| HTFP `4yNUJEkagan8hBNy1X9rkNSVwpBDidsSMWTF9ifZtLXJ` | 1 | absent/0 | 650,240 |
| Team `E3AMKJ31TmoYp7tkvQ6ffFrNNSkminYjWpV4jUzNBneo` | 2 | absent/0 | 650,240 |

H2 is the existing source/rent payer; G0 pays network fees. The recipients are
canonical distinct same-multisig vault PDAs and do not sign. This is a System
transfer, not createAccount; it proposes external setup balances, not PIV principal,
yield, distributions or contribution capital. If either recipient is fully funded,
the builder stops with no packet. Invalid owner/data/executable state or Rent0
mismatch also stops. Absent accounts are not reported as funded System accounts.

The exact legacy wire is **360 bytes**, message 231, header [2,0,1], two zero
signature slots, five global roles. G0/H2 are writable signers, HTFP/Team writable
nonsigners, System readonly. No ALT/resource/priority instruction or extra transfer.
Root ran the pure stdlib Python builder once after separate static review; literal
root wire audit and pinned-SDK public roundtrip pass. Separate actual-wire/fee and
final-freshness audits also pass; exact receipts are bound in the JSON.

At account context 454849119, H2 was 4,696,464,320, G0 999,980,000 and both recipients
absent. Fresh Rent0=650,240 matches canonical Rent 5,080 lamports/byte-year, threshold 1,
burn 50. Current raw 330-byte multisig matches retained D-043 and independently parses
six accepted members/mask 7/threshold 4/autonomous/timelock 0. This does not prove four
current votes, spending-limit exclusivity or fresh deployed-Squads image/source equality.
Original 4WK source was not read; ALT state is retained D-047, not refreshed here.

Exact-message read-only fee quote is **10,000 lamports** at 454850381, separately
bound to the raw message. This is contextual evidence; **10,000 is a future proposed
cap**, not accepted spending authority or future fee assurance. H2 credits total
1,300,480; conditional H2 afterward 4,695,163,840 and G0 after quoted fee 999,970,000
are projections only. No funds or fees moved, and no simulated/future state or full
wallet sufficiency is established. The initializer's recipient-state requirement
still remains unmet while both accounts are absent.

Snapshot transport is retained exactly, with last valid block height 406364757.
Final read-only observation shows height 406365015 > last valid 406364757: the
packet is **expired, archived and not an operational instruction**. No blockhash/
amount/packet rebinding or helper rerun occurred. Future use needs exact fresh reconstruction,
state/fee/lifetime review and separate explicit signing/spending authorization.
A historical quote or complete unsigned packet is not a live transaction ready to send.

Root made nine read-only requests in five HTTP batches, including the final
expiry observation. A reviewer input-counter comparison premise was corrected
from preserved public metadata, not a helper/RPC/runtime failure. This task has no private-path/key access, signing,
simulation, submission, funds, authority change, build, application tests or Bank/SBF.
The 198 retained runtime/source inputs and earlier results are preserved. M4 remains
open/unwaived with stopped drafts untouched; M5/public NOT READY/deployment NOT
AUTHORIZED. Next prepare artifact compatibility and the exact governed package
under the existing mandate. Later sensitive live actions require separate scope.
Root publishes reviewed integration only; main 824342d/protected 3677fee and the
2-GiB reserve remain unchanged. Prior immutable cards/proposals/archives stay exact.
