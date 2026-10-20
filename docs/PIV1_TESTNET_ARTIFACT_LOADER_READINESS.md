# Artifact/loader readiness — direct governed Deploy CPI blocked

**INVALID_DIRECT_DEPLOY_CPI_DO_NOT_EXECUTE.** The first isolated local runtime run
failed at the atomic Program-create/Squads-execute step. This is an actual recipe
failure, not a successful artifact-deployment test or an expected-map correction.
D-048's conditional deployment permission remains recorded, with **READINESS UNMET /
NO LIVE TRIGGER**. The single [M5 register](PIV1_M5_TESTNET_READINESS.md) owns readiness.

One guarded host-test build passed in 13.66 seconds without diagnostics. The single
Bank run exited 101 after 3.349 seconds, without a guard stop. Seven setup/governance
transactions succeeded; eight complete packets record 15 zero signature slots and
75,000 lamports of local feeDetails. These are unsigned local Bank entries, not live
signatures or network fees. No retry followed. Root and separate actual-evidence reviews confirm the failure/blocker,
not a passing runtime.

The successful prefix executes Buffer-only handover from G0 to canonical vault0,
then verifies every fixture-preloaded ELF byte/hash and authority before storing,
proposing and collecting four real retained-Squads approvals. The atomic step then
reports `Program BPFLoaderUpgradeab1e11111111111111111111111 not supported by inner
instructions` at outer instruction index 4, before invoking native loader Deploy.
There is no deploy-time ELF verification, deployment or later-slot heartbeat proof.
Eight pre-failure snapshots retain 323 account records and two CPIs. The last full map
after approve_4 has H2 144,591,280 and G0 999,920,000; these are not final balances.
No post-failure complete map was emitted: final balances or rollback are not proved.

The local Bank 4.2 CPI whitelist permits loader Upgrade, SetAuthority, feature-gated
SetAuthorityChecked and Close, but excludes DeployWithMaxDataLen. The reviewed official
4.4 source has the same relevant guard; this is not an assertion that the observed
4.4.0-beta.1 node binary equals that source. Reviewing only the loader's Deploy branch
was insufficient. Rust SDK account ordering differs from the archived JavaScript wire;
comparison is semantic instruction/privilege equivalence, not exact wire replay.

The raw-wire auditor initially assumed identical Rust/JavaScript key ordering; that
read-only audit guard was corrected and preserved without changing packets or rerunning
the application. Earlier source-only fixture corrections and guard-import bytecode
are also retained separately from the genuine runtime failure.

The unchanged 894,552-byte candidate remains SHA-256
`e29735e1467da98ecddb1eea9fd25abaffa4a450bd899ddc137f7d8c0da35b99`, under the
actual public PIV ID. It is **not deployment-selected**. No PIV build or ELF change
occurred. One host harness was compiled from the unchanged locked dependency graph;
198 retained source/dependency/toolchain/artifact inputs match.

Buffer bytes were preloaded, with Buffer rent debited from the same synthetic H2
balance; this does not test upload. Config/registry/reward canary accounts had separate
synthetic capital and were not exercised by a canary. Clock 100, normalized Rent
5080/threshold 1/burn 50 and selected GetSysvar/V2/V3 features are declared local
fixtures, not a full Testnet replay. Four official Testnet read-only RPC requests/two
HTTP and four official source GETs are separate observations; web source views are
separate background context. Named disable-old feature absence proves no whole-runtime
or ELF compatibility.

The original [loader proposal](PIV1_TESTNET_GOVERNED_LOADER_PROPOSAL.json) and its raw
host evidence stay immutable. Their direct-CPI deployment recipe and operational
packet/fee/budget projections are invalid and must not be executed. The current package
must not carry those projections into another authority architecture.

Next prepare a separately reviewed exact **top-level** atomic bootstrap deployment
plus handover, with an explicit temporary on-curve authority, only as an unselected,
unapproved, unexecuted source alternative. This changes first-instruction authority
and Buffer/pre-vote guarantees; it is no automatic fallback or chosen governance
policy. Exact four-approval/hash/funding controls, selected artifact/version limits,
initializer anchor frozen before votes, recipient provisioning/resources and the
separate M4 risk/scope decision remain open. Routine preparation continues; no live
permission is triggered while readiness is unmet.

M4 remains OPEN_UNWAIVED and stopped drafts remain untouched. D-041/D-043/D-045/D-047
stay consumed. No private access, keys, signing, RPC simulation, send, live funds,
live handover or live deployment occurred. The local Buffer handover did execute. Main 824342d/protected 3677fee and the 2 GiB reserve
are preserved; root owns reviewed integration publication. Exact raw receipts and
completed actual-evidence review are indexed in [the public result](PIV1_TESTNET_ARTIFACT_LOADER_READINESS.json).
