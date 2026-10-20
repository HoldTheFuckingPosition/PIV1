# D-047 finalized Testnet funding and ALT setup

**THREE PHASES FINALIZED; AUTHORIZATION CONSUMED. DO NOT REPEAT.**
The founder replied “on avance j'autorise” to advance the exact funding/ALT scope.
The [public result](PIV1_TESTNET_FUNDING_ALT_SETUP.json) binds authorization, signed
bytes/signatures, balances and audits. The [single M5 register](PIV1_M5_TESTNET_READINESS.md)
owns readiness. This auxiliary setup does not authorize deployment or later spending.

| Phase | Finalized slot | Network fee, lamports |
| --- | ---: | ---: |
| Fixed 4WK→H2 credit 4,700,000,000 | 454845162 | 5,000 |
| One new ALT create | 454845438 | 10,000 |
| Same-table exact 16 extension | 454845694 | 10,000 |

Actual ALT: **4SoVCaX8tzTE1R6jGMJTArPq75afvBtVyr93Mm6hbTFF**, derived from G0
and fresh SlotHashes member 454845200, bump 255, after funding finalized. Final
reread 454845700 verifies the ALT program as table owner, G0 authority,
568 bytes, all 16 exact ordered entries, active deactivationSlot=u64::MAX and a
strictly later slot than extension 454845694. The JSON supplies raw table bytes/hash.
The table itself is owned by the ALT program; the separate deployed ALT program
image is loader-owned and source-equality is not claimed.

| Factual final balances at 454845700 | Lamports |
| --- | ---: |
| Original 4WK source | 128,763,968,087 |
| H2 | 4,696,464,320 |
| G0 | 999,980,000 |
| Held in ALT | 3,535,680 |

Source paid 5,000; G0 paid 20,000; total fees 25,000. ALT held rent is 3,535,680
(create 934,720 + extension 2,600,960). Funding 4.7 SOL is a wallet transfer, not an
expense or proof of a fully funded project. H2's remaining balance is factual;
it grants no spending authority or assurance of future fees/rent/eligibility.
No recipient account was provisioned, G0 refilled or unallocated headroom spent.

Each phase passed root approval/unsigned review, one signature attempt, exact
signed simulation(sigVerify=true, no blockhash replacement), one send(maxRetries0),
finalization and complete balance/account reconciliation before the next phase.
Root verifies all five Ed25519 signatures and on-chain bytes. Separate actual
fund/create/extend and aggregate actual-outcome reviews pass; exact receipts are bound.
There were 92 logical RPC requests/90 HTTP: 86 ordinary reads, three signed
simulations and three sends. No failure, resend or new signature retry occurred.
Existing key reads were source 1, G0 2 and H2 2 by root only; no new key, program-key
read, private bytes/seeds/private-file hashes or copies are published.

Canonical Rent and the actual deployed ALT image were checked before signing,
before send and afterward. Rent 5,080 lamports per byte-year, threshold 1, burn 50 produces the quoted
56/568-byte floors. No native inclusion-time atomic rent cap is claimed; actual
debits equal the reviewed quotes. Root/reference/oracle/parser/host-count premise
corrections are retained public-review evidence, not failed runtime or live retries.
No broader current-runtime or PIV deployability claim follows from this ALT lifecycle.

G0 remains a hot ALT manager: append/deactivate/eventual close are possible, not
rewriting existing indexed entries or singly approving PIV economics. No further
ALT change is authorized. Future use needs fresh authority/content/active/warmup
checks and its own exact package. The old D-046 expired candidate/wires and
immutable card/proposals remain preserved; no expired packet was reused.

D-047 is consumed, as are D-041/D-043/D-045. No recipient, buffer handover,
PIV deployment, initializer/economic action, main publication or future refill is
authorized. Next prepare the remaining recipient/artifact/Clock-anchor/governed
package under the existing preparation mandate; later sensitive live actions need
their own exact scope. Old tests/Bank/SBF are retained, not rerun;
no new build or application tests. M4 stays open/unwaived with stopped drafts
untouched; M5/public NOT READY/PIV deployment NOT AUTHORIZED. Reviewed integration
only; main 824342d/protected 3677fee and the 2-GiB reserve remain unchanged.
