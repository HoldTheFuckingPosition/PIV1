# Exact unsigned funding and ALT packets — D-046 preparation

**ARCHIVED EXPIRED — DO NOT SIGN OR SEND. PROFILE ACCEPTED; DEPLOYMENT NOT AUTHORIZED.**
The founder said: “ok je valide le profil proposé, prépare les transactions exactes
pour le financement et l’ALT, sans rien envoyer ni déployer pour le moment stp”.
[D-046](PIV1_DECISIONS.md#d-046--confirmed--profile-acceptance-and-three-unsigned-fundingalt-packets-only)
records profile acceptance and preparation only. The [public packet JSON](PIV1_TESTNET_FUNDING_ALT_PACKETS.json)
contains complete messages/wires with **all five signatures zero**. It is public
snapshot evidence, not a runnable signing helper or authorization to submit.
The [single M5 register](PIV1_M5_TESTNET_READINESS.md) owns readiness.

Accepted profile: six existing guardians G0..G5 in PIV slot order; permutation
[3,1,4,0,5,2]; same-multisig governance/HTFP/Team indices 0/1/2 and acknowledged
recipient purposes; G0 creator/executor/fee payer/ALT manager, G0..G3 approvers,
distinct H2 rent payer. Startup pause=true and technical capacity 1x/894,552 bytes
are accepted profile choices, not selected artifact or deployment approval.
The accepted KIF policy captures and freezes a fresh official Clock literal before
initializer storage/votes. Actual anchor is null; no old fixture timestamp is reused.
Pause does not stop Clock/KIF periods. No automatic unpause or immediate yield.

| Exact unsigned packet | Bytes | Signature slots | Observed fee / cap, lamports |
| --- | ---: | ---: | ---: |
| Fixed 4WK→H2 credit 4,700,000,000; source is fee payer | 215 | 1 | 5,000 / 10,000 |
| ALT create; G0 fee/authority, H2 rent payer | 346 | 2 | 10,000 / 10,000 |
| ALT extend with exact 16 entries | 858 | 2 | 10,000 / 10,000 |

All three use the exact blockhash/lifetime in the JSON, with no priority/resource
instructions. Fee queries at context 454837490 bind exact message/wire hashes:
total 25,000, source 5,000 and G0 ALT 20,000. These are observed quotes, not a future
fee guarantee. No recipient provision, G0 refill or other transfer is included.
Native ALT create authority is requested readonly/nonsigner per canonical Rust ABI;
the pinned SDK default signer flag is explicitly normalized. G0 is nevertheless a
global writable signer because it pays the fee. Extend requests the G0 signature.

Raw finalized SlotHashes/wallet snapshot context 454837377 has 512 entries. Newest
member 454837376 derives candidate **4wDohUABtup3qve9qxXYHEqif3QcbjKKpUqZWsKeWMcS**.
It was absent at 454837433; ALT program was observed executable. The table was not
created or extended. Exact16 ordered contents are copied from the accepted profile.
Actual live table address/recent slot/owner/extension/content/warmup stay unset.
The candidate and blockhash are mechanical snapshot bindings, not standing selections.
Final read-only observation at 20:26:07 UTC found height 406352303 > last valid
406351915: the blockhash expired. SlotHashes 454837978..454837463 no longer contains
selected 454837376. **Both gates expired; all three wires are archived, do not sign
or send.** No helper rerun, automatic candidate/packet rebinding or live ALT selection occurred.

Source balance was 133,463,973,087; G0 had 1,000,000,000; H2 was absent/zero in the
common snapshot. No wallet balance was changed by this task. H2 can pay ALT rent only **after actual confirmed
funding**, which has not occurred. R56=934,720; R568=3,535,680; extension delta 2,600,960.
Under this zero-prefund conditional model, confirmed 4.7 SOL minus ALT rent would leave
H2 with 4,696,464,320, not an observed/simulated balance. Source pays network fees;
G0 pays ALT fees. Recipient floors and later economics are excluded from these packets.

A blockhash can expire independently of the selected slot leaving SlotHashes.
Before any future execution, separately authorize the exact package, capture/review
current state and both lifetime gates, and keep all packet hashes bound. A new slot
changes the ALT address; it cannot be silently substituted. Later table verification
must check owner, authority, active state, exactly16 ordered entries/no extras and
execution strictly after actual final extension. G0 can append/deactivate/eventually
close, not rewrite indexed entries or singly authorize PIV economic changes.

Root ran derive and prepare once each, stderr empty, after separate source review;
three read-only exact-message fee queries followed without rerunning the helper.
Independent actual-wire and final-freshness audits pass; their exact receipts
are bound in the public JSON. Root made 11 read-only requests in five HTTP batches. No keys/private files, signing, simulation, send, funds, ALT operation,
authority change, build, application tests or Bank/SBF runtime occurred. Old results
are retained; pool/referrer/current Squads state and artifact compatibility were not
refreshed. No signer/control, successful ALT lifecycle or public readiness is proved.

D-041/D-043/D-045 stay consumed. M4 remains open/unwaived, stopped drafts untouched;
M5 is preparation/public NOT READY/deployment NOT AUTHORIZED. Next resolve exact
future authorization and fresh lifetimes/state; no signing or submission follows
profile acceptance. Root publishes reviewed integration only; main 824342d and
protected 3677fee stay unchanged. Preserve all evidence and the 2-GiB reserve.
