'use strict';

// Offline client adapter for the retained rent-resilience ELF. No RPC or signer API.
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');
const init = require('../../clients/cli/initializer-plan.cjs');
const state = require('./state-inspect-v2.cjs');
const intake = require('./intake-plan-v2.cjs');
const {PublicKey} = init;

const FIXED_ROLES = [
  'config', 'active_distribution', 'pending_sol_vault', 'principal_sol_queue', 'operational_sol_vault',
  'distribution_escrow', 'kif_sol_vault', 'principal_jito_vault', 'pending_jito_vault',
];
const TARGET_SIZES = [1014, 891, 210, 84, 84, 84, 84, 84, 84, 0, 0, 0, 0, 0, 165, 165];
const CLOCK = 'SysvarC1ock11111111111111111111111111111111';
const U64 = (1n << 64n) - 1n;
const repo = path.resolve(__dirname, '../..');
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');

function uint(value, field) {
  assert(typeof value === 'string' && /^(0|[1-9][0-9]{0,19})$/.test(value), `${field}: canonical u64 required`);
  const n = BigInt(value);
  assert(n <= U64, `${field}: u64 overflow`);
  return n;
}

function checked(n, field) {
  assert(n >= 0n && n <= U64, `${field}: u64 overflow`);
  return n;
}

function canonicalKey(text) {
  assert.equal(typeof text, 'string');
  const key = new PublicKey(text);
  assert.equal(key.toBase58(), text);
  return key;
}

function asU64(n) {
  const bytes = Buffer.alloc(8);
  bytes.writeBigUInt64LE(n);
  return bytes;
}

function manifestContext(input) {
  const checked = init.validateManifest(input);
  assert.deepEqual(checked.missing, [], 'complete unsigned initializer manifest required');
  assert.equal(checked.manifest.governance.vaultIndex, 0, 'founder profile requires canonical Vault 0');
  return init.deriveContext(checked.manifest);
}

function describe(program, data, roles) {
  assert.equal(new Set(roles.map(({address}) => address)).size, roles.length, 'instruction role alias');
  return {programId: program, dataHex: data.toString('hex'), accounts: roles};
}

function genesisQuote(input) {
  const {manifest, rentBySize, targets} = input;
  const context = manifestContext(manifest);
  assert(Array.isArray(targets) && targets.length === 16, 'exactly 16 targets in native order required');
  const rent = Object.fromEntries([...new Set(TARGET_SIZES)].map(size => [size, uint(rentBySize[size], `rent[${size}]`)]));
  const rows = targets.map((target, index) => {
    const address = context.targets[index].toBase58();
    const role = context.roleNames[index + 7];
    assert.equal(target.address, address, `${role}: canonical address`);
    assert.equal(target.owner, init.IDS.system, `${role}: unexpected owner`);
    assert(target.dataBytes === 0 && target.executable === false, `${role}: expected empty unallocated System target`);
    const size = TARGET_SIZES[index];
    const before = uint(target.lamports, `${role}.lamports`);
    const floor = rent[size];
    // The five SOL vaults always receive a full authenticated advance. Other
    // targets get only their own shortfall. Existing SOL is not that advance.
    const funded = size === 0 ? floor : before < floor ? floor - before : 0n;
    const total = checked(before + funded, `${role}: final balance before normalization`);
    const normalized = index >= 14 ? total - floor : 0n;
    return {role, address, size, before, floor, funded, total, normalized};
  });
  const tokenPrefund = checked(rows[14].normalized + rows[15].normalized, 'Token prefund normalization');
  const pending = rows[9];
  pending.total = checked(pending.total + tokenPrefund, 'Pending SOL including normalized Token prefunds');
  for (const row of rows.slice(14)) row.total -= row.normalized;
  const debit = checked(rows.reduce((sum, row) => sum + row.funded, 0n), 'initializer funding total');
  const payerMinimum = checked(debit + rent[0], 'payer including residual rent');
  const advisory = [];
  if (context.m.initialization.initiallyPaused) advisory.push('Unpause through existing governance before MR02.');
  if (rows[9].total > rent[0] || [10, 12, 13].some(index => rows[index].before > 0n)) {
    advisory.push('Economic SOL above the technical basis must be reconciled under existing policy before MR03 and DJ01 → IB01.');
  }
  const roles = context.roles.map((meta, index) => ({role: context.roleNames[index], address: meta.pubkey.toBase58(),
    writable: meta.isWritable, signer: meta.isSigner}));
  const decimalFields = row => Object.fromEntries(Object.entries(row).map(([key, value]) => [key, typeof value === 'bigint' ? value.toString() : value]));
  return {status: 'OFFLINE_DECLARED_INPUTS_ONLY', deploymentReady: false,
    instruction: describe(context.program.toBase58(), init.encodeNative(context), roles),
    balances: rows.map(decimalFields), payerDebitLamports: debit.toString(),
    minimumPayerBalanceBeforeFeesLamports: payerMinimum.toString(), tokenPrefundToPendingLamports: tokenPrefund.toString(),
    operationalSolPrefundLamports: rows[11].before.toString(), initialStorageBasisPerSolVaultLamports: rent[0].toString(), advisory};
}

function stateForMigration(snapshot, roundVersion, liveRent) {
  const result = state.inspectSnapshot(snapshot);
  assert.equal(result.status, 'OFFLINE_STATE_INSPECTION');
  const {config, distribution} = result.inspection.decoded;
  assert(config.version === 2 && !config.paused, 'unpaused authenticated Config V2 required');
  assert(distribution.version === roundVersion && distribution.lifecycle === 'Idle', 'unexpected round version or lifecycle');
  const live = uint(liveRent, 'liveSolRentLamports');
  for (const amount of Object.values(config.technical_storage_reserves_lamports)) {
    assert(BigInt(amount) >= live, 'current rent exceeds a declared technical basis');
  }
  const keys = state.deriveTopology(snapshot.programId);
  return {config, distribution, keys, live, program: snapshot.programId};
}

function rentDelta(snapshot, quotedFloor) {
  const balance = uint(snapshot.distribution.lamports, 'current round lamports');
  const floor = uint(quotedFloor, 'next round minimum');
  return floor > balance ? floor - balance : 0n;
}

function roundV2(input) {
  const {snapshot, nextRoundRentLamports, operationalLamports, liveSolRentLamports} = input;
  const {program, keys, config} = stateForMigration(snapshot, 1, liveSolRentLamports);
  const delta = rentDelta(snapshot, nextRoundRentLamports);
  const operational = uint(operationalLamports, 'operational vault lamports');
  const baseline = BigInt(config.technical_storage_reserves_lamports.operational_sol_vault);
  assert(operational >= baseline + delta, 'MR02 must preserve operational storage');
  const roles = FIXED_ROLES.map((role, index) => ({role, address: keys[role].address, signer: false, writable: index === 1 || index === 4}));
  roles.push({role: 'system_program', address: init.IDS.system, signer: false, writable: false});
  return {status: 'OFFLINE_DECLARED_INPUTS_ONLY', deploymentReady: false, payer: 'operational_sol_vault',
    shortfallLamports: delta.toString(), instruction: describe(program, Buffer.concat([Buffer.from('PIV1MR02'), Buffer.from([1])]), roles)};
}

function pda(program, ...seeds) {
  return PublicKey.findProgramAddressSync(seeds, canonicalKey(program))[0].toBase58();
}

function roundV3(input) {
  const {snapshot, nextRoundRentLamports, liveSolRentLamports, multisigAddress, transactionIndex,
    currentGuardianKeys, payerAddress, payerLamports, payerCapLamports} = input;
  const {program, keys, config, live} = stateForMigration(snapshot, 2, liveSolRentLamports);
  const multisig = canonicalKey(multisigAddress).toBuffer();
  const index = asU64(uint(transactionIndex, 'Squads transaction index'));
  const revision = asU64(BigInt(config.guardian_registry_revision));
  const cap = uint(payerCapLamports, 'MR03 cap');
  const delta = rentDelta(snapshot, nextRoundRentLamports);
  assert(delta <= cap && uint(payerLamports, 'MR03 payer lamports') >= checked(delta + live, 'MR03 payer requirement'), 'MR03 external funding insufficient');
  assert(PublicKey.isOnCurve(canonicalKey(payerAddress).toBytes()), 'external payer must sign');
  for (const field of ['piv_authority', 'htfp_recipient', 'team_owner_recipient', 'jitosol_mint', 'stake_pool', 'validator_list', 'reserve_stake', 'manager_fee_account', 'referrer_token_account']) {
    assert.notEqual(payerAddress, config[field], `MR03 payer may not alias ${field}`);
  }
  assert(Array.isArray(currentGuardianKeys) && currentGuardianKeys.length === 6 && new Set(currentGuardianKeys).size === 6, 'six distinct guardian keys required');
  const labels = ['program', 'program_data', 'multisig', 'proposal', 'transaction', 'governance_vault', 'instructions',
    'config', 'guardian_registry', ...Array.from({length: 6}, (_, slot) => `guardian_reward_${slot}`), 'clock',
    ...FIXED_ROLES.slice(1), 'system_program', 'rent_payer'];
  const addresses = [program, pda(init.IDS.loader, canonicalKey(program).toBuffer()), multisigAddress,
    pda(init.IDS.squads, Buffer.from('multisig'), multisig, Buffer.from('transaction'), index, Buffer.from('proposal')),
    pda(init.IDS.squads, Buffer.from('multisig'), multisig, Buffer.from('transaction'), index),
    pda(init.IDS.squads, Buffer.from('multisig'), multisig, Buffer.from('vault'), Buffer.from([0])),
    init.IDS.instructions, keys.config.address, keys.guardian_registry.address,
    ...currentGuardianKeys.map((key, slot) => pda(program, Buffer.from('guardian-reward'), canonicalKey(key).toBuffer(), revision, Buffer.from([slot]))),
    CLOCK, ...FIXED_ROLES.slice(1).map(role => keys[role].address), init.IDS.system, payerAddress];
  const roles = labels.map((role, i) => ({role, address: addresses[i], signer: i === 5 || i === 25, writable: i === 16 || i === 25}));
  const roundHash = sha(Buffer.from(snapshot.distribution.dataBase64, 'base64'));
  const data = Buffer.concat([Buffer.from('PIV1MR03'), Buffer.from([1, 0]), Buffer.from(roundHash, 'hex'), asU64(cap)]);
  return {status: 'OFFLINE_DECLARED_INPUTS_ONLY', deploymentReady: false, payer: 'external_rent_payer',
    shortfallLamports: delta.toString(), roundHashSha256: roundHash, instruction: describe(program, data, roles)};
}

function jitoGenesisPair(input) {
  const {contribution, bootstrap} = input;
  const dj = intake.validateManifest(contribution);
  const ib = intake.validateManifest(bootstrap);
  assert(dj.missing.length === 0 && ib.missing.length === 0, 'complete unsigned intake declarations required');
  assert.equal(dj.manifest.action, 'contribute-jitosol');
  assert.equal(ib.manifest.action, 'bootstrap-initial');
  for (const field of ['cluster', 'programId', 'feePayer', 'transport', 'resources']) assert.deepEqual(dj.manifest[field], ib.manifest[field]);
  const d = intake.derived(dj.manifest.programId);
  const contributionAction = intake.actionInstruction(dj.manifest, d);
  const bootstrapAction = intake.actionInstruction(ib.manifest, d);
  for (const external of [dj.manifest.jitosol.sourceToken, dj.manifest.jitosol.owner]) {
    assert(!bootstrapAction.roles.some(meta => meta.address === external), 'intake source or owner aliases a bootstrap role');
  }
  const packet = init.packet({m: dj.manifest}, 'direct-jitosol-genesis', [...intake.budgets(dj.manifest.resources),
    contributionAction.instruction, bootstrapAction.instruction]);
  return {status: 'OFFLINE_DECLARED_INPUTS_ONLY', deploymentReady: false, packet,
    contribution: describe(dj.manifest.programId, contributionAction.data, contributionAction.roles),
    bootstrap: describe(ib.manifest.programId, bootstrapAction.data, bootstrapAction.roles)};
}

function verifyBindings() {
  const seals = require('./source-seal.json');
  assert.equal(process.version, 'v24.19.0');
  for (const [name, expected] of Object.entries(seals.client)) {
    assert.equal(sha(fs.readFileSync(path.join(__dirname, name))), expected, `client binding: ${name}`);
  }
  const elfPath = process.env.PIV1_V2_ELF;
  assert(elfPath && path.isAbsolute(elfPath), 'PIV1_V2_ELF must identify the selected local ELF');
  const elf = fs.readFileSync(elfPath);
  assert.equal(elf.length, seals.elfBytes);
  assert.equal(sha(elf), seals.elfSha256, 'retained ELF hash');
  const manifest = fs.readFileSync(path.join(repo, 'docs/evidence/piv1-rent-v2/production-sources.sha256'));
  assert.equal(sha(manifest), seals.productionSourceManifestSha256, 'retained production source manifest');
  for (const line of manifest.toString('utf8').trim().split('\n')) {
    const [expected, name] = line.split('  ');
    assert.equal(sha(fs.readFileSync(path.join(repo, name))), expected, `retained program source: ${name}`);
  }
  return {node: process.version, sdk: '1.98.4', elfSha256: seals.elfSha256, elfBytes: seals.elfBytes,
    productionSourceManifestSha256: seals.productionSourceManifestSha256};
}

const actions = {'--inspect': state.inspectSnapshot, '--genesis': genesisQuote, '--mr02': roundV2, '--mr03': roundV3, '--jito-pair': jitoGenesisPair};

function main(args) {
  const bindings = verifyBindings();
  if (args.length === 1 && args[0] === '--verify') return {bindings, status: 'OFFLINE_ONLY'};
  assert(args.length === 2 && Object.hasOwn(actions, args[0]), 'expected an offline action and a public fixture file');
  const stat = fs.statSync(args[1]);
  assert(stat.isFile() && stat.size <= 65536, 'public declaration must fit within 65,536 bytes');
  const bytes = fs.readFileSync(args[1]);
  assert(bytes.length <= 65536);
  return {...actions[args[0]](init.parseManifest(bytes.toString('utf8'))), bindings, declarationSha256: sha(bytes)};
}

module.exports = {verifyBindings, genesisQuote, roundV2, roundV3, jitoGenesisPair};
if (require.main === module) {
  try { process.stdout.write(`${JSON.stringify(main(process.argv.slice(2)), null, 2)}\n`); }
  catch (error) { process.stderr.write(`${error.message}\n`); process.exitCode = 1; }
}
