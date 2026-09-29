# Task 2.44 — Production protected principal SOL deposit

Status: **TECHNICALLY VALIDATED / PENDING FOUNDER ACCEPTANCE**, under D-026/D-030 M2 with the required bounded M3
deposit component. Starting integration is
`c47272acbb2f9628cc308935b1c6b5f1fa575406`; main remains
`8912cfeffcd83fa12cc1a68937a9de8dd5c6b091`. Root verified jerem, clean single
worktree and matching remote refs. Technical validation is not founder acceptance.

## Scope and completion gate

Expose accepted Task 2.6 principal SOL conversion through the production boundary
and the pinned SPL 2.0.3 `DepositSolWithSlippage` CPI. This is the concrete M3
dependency needed for this M2 handler; the full adapter remains incomplete.
Canonical P-014/P-019/P-020/P-023, A-001, G-004 and master sections 4.5/10 apply.
Do not fabricate the pure adapter's mock revision, capacity or liquidity facts.

Only recognized historical SOL in normalized, unpaused bound Idle custody may
convert. Preserve pending assets, carry, earned KIF, operations, rent, the entire
round, timing, HWM and cumulative counters. Support only zero actual deposit fees
and conversions preserving both historical book value and HWM coverage. Leave
unsupported amounts queued; no optimizer, subsidy, cost policy or HWM exception.
General fee/rounding-loss support remains OPEN.

The strict `PIV1SP01` version-1 boundary carries input SOL and a stronger
caller minimum in 25 bytes. Twenty roles, or nineteen when the configured fee
receivers coincide, extend the bootstrap account order with the derived pool
withdraw authority. Shared receiver duplication occurs only inside the pinned
ten-meta CPI. Authenticate current pool/Mint/Clock facts and the existing full
Jito identity. Use the official stored ratio, separately track current Mint supply
and recorded pool supply through legitimate direct-burn lag, and preserve all
protocol bytes except the exact expected deposit effects. Require permissionless
SOL entry (`sol_deposit_authority = None`). Only the protected SPL instruction may
execute; its minimum incorporates Config's immutable 0–1 bps tolerance. No new
dependency is planned for this narrow pinned instruction encoding.

The exact protected SPL encoding is variant 25 followed by the two little-endian
u64 values. Current Mint supply can trail recorded supply after direct burns;
require combined held units ≤ Mint supply ≤ recorded supply and increase both
supply observations separately by the same actual minted output. Preserve the
gap. Pool total and reserve native increase by the input. Only pool bytes
258–273, Mint supply and principal token amount may change in account data;
the entire remaining pool allocation, including its opaque tail, stays exact.
Valid zero-fee encodings include 0/0 and 0/N. Nonzero fees remain unsupported.

A phased crate-private accounting profile shares the accepted transition without
moving the legacy public observation checks or inventing model fields. Predict
and validate the final historical-unit update before CPI; verify actual complete
postconditions and reauthenticate identities before the sole Config/event commit.
The source and tests must substantiate these design requirements before closure.

Native Token funding may remain quarantined only under an explicit handler
contract preserving every lamport and excluding it from economic value. No
extraction, operational provenance classification or strict-accessor relaxation.
Preserve the public pure API and its validation/error order.

One delegated writer owns production code, tests and program README; a separate
reviewer checks design/source/tests/evidence. Root owns shared documentation,
locked/offline execution, static SBF gates and reviewed integration publication.
Require success, partial/repeated conversion, exact protocol/custody deltas,
fee/rounding/HWM, slippage, stale/supply/authority, pause/lifecycle, borrow/alias,
overflow and full partial-effect failure/discard/retry oracles. Host models do not
prove runtime signatures, nested SPL execution or Bank rollback. Actual production
lifecycle execution remains M4 work.

## Takeover and evidence

Evidence directory: `/tmp/piv1-t244-pilot-review`. Baseline: 242 prior inputs,
39 preserved records and 76 retained Task 2.43 logs, hash-verified without rerun.
Available capacity was 31,940,128,768 bytes; no recovery or installation is needed.
Preserve all four recovery archives, restored files and prior artifacts/logs.

Root verified local state.rs, processor.rs and instruction.rs against the retained
authenticated `spl-stake-pool-2.0.3.crate`, SHA-256
`6f0db03f091f43b5766296e80088718491b50949cd3eb4cce3e0cfed58fe2c18`,
revision `864ba3c1c564cc270ca62b6e6b558f57538ae092`. The first metadata lookup
encountered a mixed-entry KeyError before writing; the corrected read verified
the complete archive without extraction. No protocol execution is claimed.

The existing validation workflow has only task/base/path/focused-target
substitutions. Driver SHA-256:
`5e6922e442804af256406cac5ef71db0e037a37b8db8bb23486d08e09bf9879e`.
Separate driver and final source/test/README review passed. Root passed ten
focused tests on first execution (5.10 seconds including compilation), without
diagnostics. Root also passed 528 host tests +1 doctest/eight final gates on the
first execution without diagnostics; strict SBF also passed on its first execution.
The
109-input freeze SHA-256 is
`c8effa406ea3b255096b852fbf9e7c839d2bb8a25276a725015e9a4bb87eaf3d`.
Root and separate handler review identified an unchecked `data_len()` borrow on
the newly supplied withdraw authority. Before execution, the writer replaced it
with a fallible read returning `AccountBorrowFailed`; held-borrow and unsupported
multisig-shape regressions passed. No other source defect was found in review;
separate host and static artifact evidence reviews passed. The exact indexed
tree/document review is the final publication gate.
No main publication, founder acceptance, live operation, secrets access, key
creation, signing, fund movement, deployment or authority transfer is authorized.

## Actual host validation

Root executed the reviewed driver at
`/tmp/piv1-t244-pilot-review/validate.py` with `freeze`, `focused`, `final`, then
`sbf`. Host commands use pinned Rust 1.97.1 Cargo with `--locked --offline
--jobs 1`, `CARGO_INCREMENTAL=0` and the unchanged host target. The focused target
is `test -p piv1 --test principal_deposit_execution`. Final gates cover workspace
all-target tests, doctests, default/all-feature checks, `no-entrypoint`, `cpi`,
`idl-build` and warning-denied documentation. No writer/reviewer execution or
rustfmt run is claimed. Host compilation is not covered by the SBF resource guard.

All ten focused tests, 528 workspace tests and one doctest passed, with eight
final gates, zero failures/ignored tests/diagnostics and no execution-driven source
correction. Root and separate review independently verified all eighteen host
logs, 109 frozen inputs and 39 preserved records. Host evidence receipt SHA-256:
`f975d3000536b5817669f6d62498251ac15ecb7e021ad00d1345937f711f5511`.

The tests compare complete independently expected accounts, literal protected
instruction bytes/metas and PDA seeds. Both account topologies and partial/repeated
conversions preserve pending balances; direct-burn lag preserves the supply gap;
valid zero encodings and the paired-empty pool work. Historical completed state,
next-cycle carry, KIF liabilities and native quarantine remain exact. Fee, rounding,
HWM, caller minimum, pause/lifecycle, supply/epoch/authority, rent/alias/borrow and
overflow failures reject before effects. Legacy unused-field/receipt validation
and error precedence remain. Each of four internal effect prefixes in both
topologies is preserved on modeled CPI error before explicit discard and retry;
twenty account-lamport and seventeen data-field corruptions reject before Config
commit. The actual pinned Token MintTo processor runs; the surrounding stake-pool,
System transfer, signatures and transaction discard remain host models.

## Static SBF artifact and publication

The strict first SBF build passed without diagnostics. Root and separate review
verified `/tmp/piv1-bank-smoke-principal-deposit-sbf-t244-a/artifacts/piv1.so`,
579,896 bytes, SHA-256
`a61d5ed270db02af2b74626d83eb2cb5f1cafe0a704e5b4d869b8e10bf609dde`.
Result receipt SHA-256:
`15acf5fcb663875aa72a21cd649a64816325dff4cc4cfdf830b9a974c93b0264`.
All 58 SBF logs match. The raw ELF is ELF64 little-endian ET_DYN, machine 263,
flags 0, entry 404,472, with the same fourteen unresolved imports as Task 2.43.
The immutable artifact copy equals the Cargo output. No VM/Bank execution occurred.

The build took 301.09 seconds with exit 0 and no guard stop. Sampled peak group
RSS+swap was 608,350,208 bytes; minimum free space was 31,667,941,376 bytes.
Unchanged guards: 2.5-GiB sampled RSS+swap, 4-GiB per-process address space,
2-GiB disk reserve, 1,800 seconds and denied network socket creation with local
UNIX socketpair allowed. These are build limits, not runtime resource proof.
All 109 source inputs, 39 preserved records and tool pins remain unchanged.

Changed files: the new production handler, bounded private deposit component,
focused test and custody fixture; phased principal-deposit core, narrow quarantine
accessor, strict ABI/dispatch/exports and factual event; program README; and shared
guidance, specification, plans, checkpoint, README and this report. No dependency,
serialized payload, error number or economics changed. No installation, cleanup
or archive restoration was needed. No post-test source correction occurred.

Root publishes only the separately reviewed integration tree by normal unsigned
commit and non-force push. Git and
`/tmp/piv1-t244-pilot-review/publication.json` record the actual commit, independent
remote-ref check, clean worktree and unchanged main/Task 2.3 refs. Main integration,
founder acceptance and live approval remain separate. No secrets access, key
creation, signing, Mainnet action, deployment, fund movement or authority transfer
occurred. Save and end this bounded session for economical usage.

## Remaining production path

The next coherent M2 dependency is production distribution preparation: current
official valuation, authenticated Clock/guardian snapshot, pending-first funding,
cadence/insufficiency rules and conservative withdrawal-target/minimum evidence.
Close any demonstrated real withdrawal-adapter dependency alongside that boundary;
do not substitute caller/model revision or capacity facts. Delayed leg initiation,
finalization, settlement and post-settlement pending integration remain open,
followed by the complete M4 production lifecycle and exact Testnet package.
No existing historical runtime artifact proves this new handler's compute, heap,
nested CPI execution or Bank rollback. General fee/rounding-loss support remains
OPEN. No new economic or governance decision is inferred.
