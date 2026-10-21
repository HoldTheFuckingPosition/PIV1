# Local native deployment and governed initialization — evidence checkpoint

2026-10-21 Europe/Warsaw. **D-048 CONDITIONAL PERMISSION / READINESS UNMET /
NO LIVE TRIGGER / PUBLIC PACKAGE NOT READY.** The [single M5 register](PIV1_M5_TESTNET_READINESS.md#finite-readiness-matrix)
owns readiness. This [public evidence index](PIV1_TESTNET_STARTUP_GOVERNED_INITIALIZER.json)
records an isolated local proof, not an executable live package.

Final v5 passes 11 positive local entries: native atomic bootstrap at the actual
PIV ID, nine existing-ALT Squads initializer packets and production heartbeat.
Program, canonical ProgramData and all 16 PIV targets are absent at genesis.
Native loader verification deploys the unchanged 894,552-byte ELF; final authority
is vault 0. Child slot 2 then stores the initializer, creates the active proposal,
processes G0..G3 approval bodies, executes the real Squads CPI initializer and
successfully heartbeats while paused. No application state is preloaded or patched.

Root and separate independent audits verify 721 evidence records, 633 account
records in 13 fresh full maps, 11 complete wires, 23 zero signature slots and 46
ordered CPIs. Of these, 41 belong to initialization: PIV at outer 3/stack 2,
16 System transfers, 11 allocations, 11 assignments and two Token initializations.
Native loader's PD-create System CPI is outer 7/stack 2. Full source-literal Config,
Idle distribution, registry, six rewards, five System targets and two Token states
match; initial economic ledgers are zero. The multisig index moves 0→1, proposal
ends Executed, and heartbeat changes only reward 0 activity plus G0's fee.

| Final local entry | Complete bytes | Zero slots | CU | Loaded bytes |
| --- | ---: | ---: | ---: | ---: |
| atomic_native_bootstrap | 1035 | 6 | 5,940 | 895,494 |
| buffer_create | 1215 | 2 | 15,762 | 980,912 |
| buffer_extend_1 | 896 | 1 | 10,431 | 982,386 |
| transaction_from_buffer | 421 | 2 | 57,816 | 982,535 |
| proposal_create | 383 | 2 | 15,844 | 980,912 |
| approve_1 | 245 | 1 | 9,606 | 981,473 |
| approve_2 | 341 | 2 | 10,259 | 981,537 |
| approve_3 | 341 | 2 | 10,707 | 981,537 |
| approve_4 | 341 | 2 | 11,549 | 981,537 |
| execute | 963 | 2 | 992,404 | 3,833,006 |
| initialized_paused_positive_heartbeat | 601 | 1 | 59,920 | 897,649 |

Measured initializer usage is 992,404 CU/3,833,006 loaded bytes under requested
1.4M CU, 256-KiB heap and 8-MiB loaded-data limits. Bootstrap measures
5,940 CU/895,494 loaded bytes here; this is distinct from older retained results.
These measurements establish this local profile, not future network sufficiency.

The same synthetic H2 starts at 4,696,464,320 and advances 4,545,162,360 to the
preloaded ELF Buffer. Native deployment credits that Buffer back to H2 before PD
rent, leaving 150,428,200. Governed initialization debits H2 gross 46,400,720:
25,384,760 target floors, 8,569,960 buffer, 8,514,080 stored transaction and
3,931,920 proposal. H2 ends 104,027,480. The buffer refund goes only to creator G0,
never H2; G0 ends 1,008,434,960 after that refund and 115,000 final-process fees.
These are local fixture balances, not a refreshed wallet or complete funding budget.

The history is retained: four builds (v1 compile failure; v3/v4/v5 pass) and three
processes. V3 failed before Bank/any transaction because 11 original rent_epoch
u64MAX values differed from a lossy embedded JS copy; Rust now uses the original
hash-bound public JSON. V4 completed nine entries/100,000 local fees, then stopped
on a harness ALT-index-order assumption before execute. V5 preserves the actual
Rust permutation and full semantic checks and completes all 11 entries/115,000
local fees. V2 snapshot was unexecuted. Compiler errors, requested-meta correction,
all raw logs and read-only audit-reader corrections remain separate evidence;
no product defect or failed PIV initializer is inferred from these harness failures.

The ELF Buffer is preloaded Some(G0), so upload/Buffer creation is not proved.
Recipients/governance vault are separately capitalized fixtures; actual provisioning
is not proved. Retained external images/accounts and ProgramData header/Rent
normalization are explicit. Historical D-047 ALT bytes feed planning unchanged;
only local lastExtendedSlot bytes 12..19 become zero for Bank slots 1/2. Local
Clock 100/anchor 0, Rent 5080/threshold 1/burn 50 and three enabled features are
fixtures, not a real anchor or a full Testnet feature clone. Bank 4.2 is not the
retained observed Testnet 4.4 runtime. Rust key/lookup ordering and local blockhash
may differ from archived JS; exact instructions and global privileges agree.

All signatures are zero. Real Squads code processes local approval bodies, but
no guardian key control or public signatures are proved. G0's temporary bootstrap
authority/Buffer-write trust remains unselected; final vault authority alone does
not supply an immutable bytecode-hash guard. Artifact selection, full economic
lifecycle and live readiness remain open. The prior failed direct Deploy-CPI
recipe stays invalid; prior immutable records and stopped M4 drafts are untouched.

Exact source/input/binary/build/run and root/separate audit hashes are indexed in
JSON; full wires/maps remain in hash-bound public evidence outside Git. Before
docs, 367 retained tracked inputs and 63 pins matched. Only package preparation
metadata changes one pin afterward; its four selected sections stay exact.
No fresh standalone 198-input audit or post-doc execution-baseline claim is made.
No new RPC, private access, key/signing, live funds/authority/deployment, PIV build,
dependency or toolchain change occurred.

Next consolidate the exact public startup/trust/version/resource/funding and
recipient package, actual sequential indices/transport and Clock literal frozen
before storage/votes. M4 remains OPEN_UNWAIVED; D-048 conditional permission is
retained with no live trigger or renewed consumed scope. Reviewed integration only,
main 824342d/protected 3677fee, reserve 2 GiB. No routine approval question follows.
