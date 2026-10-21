# Consolidated Testnet startup sizing and conditional budget

Date: 2026-10-21 Europe/Warsaw. Source checkpoint: `8581810`.
This is a non-executable evidence summary; the [single M5 register](PIV1_M5_TESTNET_READINESS.md)
owns readiness. **D-048 conditional permission remains retained, readiness UNMET,
NO LIVE TRIGGER, public NOT READY.** M4 stays OPEN_UNWAIVED; stopped drafts and
consumed scopes are untouched. [JSON evidence index](PIV1_TESTNET_CONSOLIDATED_STARTUP_BUDGET.json)
binds the complete public artifacts and separate actual reviews.

## Actual preparation result

One offline assembly and budget reconciliation, followed by seven host-check groups,
pass. Root and separate review independently check all 949 complete unsigned packets,
963 zero signature slots, all 936 ELF chunks and 975 conditional ledger rows.
Root makes 41 official Testnet read-only RPC requests in seven HTTP calls: the
32-account subset, 14 rents, 15 representative fees and recorded freshness.
No new Bank/build/private access/key/signing/simulation/send/funds or live authority
operation occurs. The prior successful local startup is retained, not rerun.

| Ordered phase | Packets | Full wire bytes | Zero signature slots |
| --- | ---: | --- | ---: |
| Seeded loader Buffer create + initialize | 1 | 510 | 2 |
| ELF Writes | 936 | 935 × 1,232; last 968 | 936 |
| Native atomic bootstrap | 1 | 1,035 | 6 |
| Two-recipient provisioning | 1 | 360 | 2 |
| Existing-ALT governed initializer | 9 | 1,215 / 896 / 421 / 383 / 245 / 341 / 341 / 341 / 963 | 16 |
| Production heartbeat | 1 | 601 | 1 |
| Total | 949 | All at most 1,232 | 963 |

Maximal Write payload is 956 bytes; 957 would make a 1,233-byte packet. The
894,552-byte retained candidate hashes to `e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99`.
This is encoding/reassembly proof, not actual upload, deployment or artifact selection.
All full wires, ordered roles, hashes and the 936-row chunk manifest remain in the evidence directory identified by the JSON index;
Git holds their byte/hash descriptors rather than copying those large arrays.

## Observations and fee projection

The finalized account subset is at slot 454903936, within recorded start/end
454903935..454903938. The node reports 4.4.0-beta.1; this is not a coherent whole-Bank
snapshot, source/image equality or current pool/referrer eligibility proof. Exact
original public integers retain seven rent_epoch u64-MAX values. H2 is observed at
4,696,464,320 lamports, G0 at 999,980,000 and source 4WK at 128,763,968,087.
Both recipients and vault 0 are absent; Program/PD/Buffer and the 16 PIV targets
are absent. The observed ALT is the existing exact active 16-entry table; reuse adds
no setup packet/rent or authority signature solely for reuse.

Fifteen actual message quotes at context 454903964 sum to 145,000 lamports.
The first Write represents itself plus 934 unquoted same-size writes; the tail
has its own quote. The additional 4,670,000 projection gives **4,815,000** across
949 packets. These are 15 quotes, not 936 independent Write quotes or an inclusion
fee/Rent cap. G0's pre-refund maximum draw is **4,755,000**.

## Conditional same-payer cashflow

| H2 movement under recorded rent model | Lamports |
| --- | ---: |
| Observed starting balance | 4,696,464,320 |
| Loader Buffer advance, debited before its later refund | 4,545,162,360 |
| Program rent | 833,120 |
| Same-H2 Buffer credit during native deployment | 4,545,162,360 |
| ProgramData rent | 4,545,203,000 |
| H2 after loader | 150,428,200 |
| Two external recipient setup credits | 1,300,480 |
| Initializer gross H2 outflow | 46,400,720 |
| Conditional H2 final | 102,727,000 |
| Required initializer payer residual, counted once | 650,240 |
| Required initial envelope including residual | 4,594,387,560 |
| Headroom above required residual | 102,076,760 |

Initializer gross comprises 25,384,760 for 16 targets, 8,569,960 governance Buffer,
8,514,080 stored transaction and 3,931,920 proposal. The governance Buffer refund
**8,569,960 goes only to creator G0**, never H2; projected G0 final is 1,003,734,960.
Recipient credits are external setup balances, not principal/yield. This excludes
later operational liquidity/test capital and proves no future funding sufficiency.
No new source funding or third-vault transfer is included. Source has no vault 0
rent-floor predicate for that signer identity, but earlier local execution funded
that vault separately: the absent-vault runtime case remains unproved.

## Lifetime, trust and remaining package

All packets share one sizing blockhash, last-valid height 406418031. At the final
recorded read (00:29:59 UTC), height 406417972/slot 454904027 returned valid=true.
That is historical, not current validity or assurance that 936 uploads fit a lifetime.
No automatic blockhash/slot rebinding or operational instruction follows.
The encoded anchor 0 is a sizing placeholder; the real anchor remains null and must
be frozen before storing/votes without silent refresh. Pause does not stop Clock.
Artifact/bootstrap trust remain unselected; G0 can write Buffer until inclusion,
and six exact-message endorsements are not Squads votes or an on-chain global hash
threshold. The invalid direct Deploy-CPI recipe remains invalid. Prior unsigned
Bank 4.2 evidence is not current Testnet 4.4 replay or live signer-control proof.

Next consolidate the exact founder/deployment-readiness package: artifact and
bootstrap trust/version limits, actual sequential indices/state, recipient funding,
Clock-before-votes binding, lifetime/resume rules, operating/test-capital scope and
M4 risk/scope reservation. Continue preparation under the existing mandate; no
routine approval question or live trigger is created by these checks.

All 367 retained tracked inputs outside checkpoint documentation roots and four
selected package sections remain unchanged. The 70 source pins bind the pre-doc
baseline; later package preparation metadata is an explicit hash transition.
Root's read-only budget audit corrected a nonexistent package path after successful
math checks; original/correction remain retained, with no application/RPC failure.
Reviewed integration only; main `824342d`, protected `3677fee`, reserve 2 GiB.
