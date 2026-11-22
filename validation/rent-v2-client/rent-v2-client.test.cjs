'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const {spawnSync} = require('node:child_process');
const {PublicKey, IDS} = require('../../clients/cli/initializer-plan.cjs');
const {VersionedTransaction} = require(require('../../validation/genesis-transport/transport.cjs').SDK);
const {fixture, literal, publicPoints} = require('../../clients/cli/initializer-plan.test.cjs');
const {TEMPLATE} = require('./intake-plan-v2.cjs');
const state = require('./state-inspect-v2.cjs');
const client = require('./rent-v2-client.cjs');

const witness = name => fs.readFileSync(path.join(__dirname, 'fixtures', name));
const legacy = () => JSON.parse(witness('testnet-v1.json'));
const R = bytes => BigInt(bytes + 128) * 2575n;
const SIZES = [1014, 891, 210, 84, 84, 84, 84, 84, 84, 0, 0, 0, 0, 0, 165, 165];
const ROLES = ['pending_sol_vault', 'principal_sol_queue', 'operational_sol_vault', 'distribution_escrow', 'kif_sol_vault'];
const AMOUNTS = ['650240', '701123', '742777', '807111', '18446744073709551615'];

function snapshot(round) {
  const s = legacy();
  s.context = {sourceKind: 'synthetic-literal', sourceLabel: 'Rust layout fixture', slot: '42'};
  s.config.dataBase64 = witness('config-v2-absent.bin').toString('base64');
  s.distribution.dataBase64 = witness(`round-v${round}.bin`).toString('base64');
  s.distribution.lamports = R([0, 891, 924, 1199][round]).toString();
  return s;
}

function quoteInput(shared, prefunds) {
  const manifest = fixture({shared, vault: 0});
  const independentInitializer = literal(manifest);
  return {manifest, rentBySize: Object.fromEntries(SIZES.map(size => [size, R(size).toString()])),
    targets: independentInitializer.targets.map((key, slot) => ({address: key.toBase58(), owner: IDS.system,
      dataBytes: 0, executable: false, lamports: (prefunds[slot] ?? 0n).toString()}))};
}

test('sealed inputs bind the exact previous SBF and candidate source manifest', () => {
  const binding = client.verifyBindings();
  assert.equal(binding.elfBytes, 967696);
  assert.equal(binding.elfSha256, '2ea454e9eac013bab063651c6691bf1cf84fad3e5d3e0b67375f2603e4138109');
});

test('legacy capture stays V1; independent Rust fixtures verify all V2 storage fields and option offsets', () => {
  const current = state.inspectSnapshot(legacy());
  assert.equal(current.inspection.decoded.config.version, 1);
  assert.equal(current.inspection.decoded.distribution.version, 3);
  assert.equal(current.inspection.summary.config.technicalStorageReservesLamports, null);
  assert.deepEqual(current.inspection.decoded.config.migration_reserve, Array(64).fill(0));
  for (const [optionCount, name] of ['absent', 'one', 'two'].entries()) {
    const bytes = witness(`config-v2-${name}.bin`);
    const {value, layout} = state.decodeConfig(bytes);
    assert.equal(value.version, 2);
    assert.equal(layout.allocationBytes, 1014);
    assert.equal(layout.trailingZeroPaddingBytes, 16 - 8 * optionCount);
    assert.deepEqual(value.technical_storage_reserves_lamports, Object.fromEntries(ROLES.map((role, index) => [role, AMOUNTS[index]])));
    assert.equal(value.last_successful_preparation_at, optionCount ? '101' : null);
    assert.equal(value.last_valid_insufficient_attempt_at, optionCount === 2 ? '202' : null);
    assert.equal(value.guardian_registry_revision, current.inspection.decoded.config.guardian_registry_revision);
    assert.equal(value.protected_principal_hwm_lamports, current.inspection.decoded.config.protected_principal_hwm_lamports);
    assert.equal(state.inspectSnapshot({...legacy(), config: {...legacy().config, dataBase64: bytes.toString('base64')}}).readiness.custodyVerified, false);
  }
});

test('strict Config discriminator/version, reserved bytes, padding and provenance metadata refuse invalid declarations', () => {
  const bytes = witness('config-v2-absent.bin');
  const mutations = [
    [0, 0, 'state_discriminator'], [8, 3, 'unsupported_state_header'], [8, 1, 'nonzero_migration_reserve'],
    [9, 2, 'invalid_bool_tag'], [974, 1, 'nonzero_migration_reserve'], [1013, 1, 'nonzero_trailing_padding'],
  ];
  for (const [index, value, expected] of mutations) {
    const changed = Buffer.from(bytes);
    changed[index] = value;
    assert.throws(() => state.decodeConfig(changed), {'message': expected});
  }
  assert.throws(() => state.decodeConfig(bytes.subarray(1)), /account_size/);
  const s = snapshot(1);
  s.config.owner = IDS.token;
  assert.throws(() => state.inspectSnapshot(s), /declared_owner_mismatch/);
  s.config.owner = s.programId;
  s.config.address = IDS.token;
  assert.throws(() => state.inspectSnapshot(s), /canonical_state_pda/);
});

test('IN02 313-byte stored data and 34/35 native roles match the independent literal wire oracle', () => {
  for (const shared of [true, false]) {
    const input = quoteInput(shared, []);
    const quote = client.genesisQuote(input);
    const expected = literal(input.manifest);
    assert.equal(quote.instruction.dataHex, expected.data.toString('hex'));
    assert.deepEqual(quote.instruction.accounts.map(role => role.address), expected.keys.map(key => key.toBase58()));
    assert.equal(quote.instruction.accounts.length, shared ? 34 : 35);
    assert.deepEqual(quote.instruction.accounts.flatMap(({signer}, i) => signer ? [i] : []), [5, shared ? 29 : 30]);
    assert.deepEqual(quote.instruction.accounts.flatMap(({writable}, i) => writable ? [i] : []),
      [...Array.from({length: 16}, (_, i) => i + 7), shared ? 29 : 30]);
    assert.equal(quote.payerDebitLamports, SIZES.reduce((sum, size) => sum + R(size), 0n).toString());
    assert.equal(quote.minimumPayerBalanceBeforeFeesLamports, (BigInt(quote.payerDebitLamports) + R(0)).toString());
    assert(quote.advisory.some(x => x.startsWith('Unpause')));
  }
});

test('prefunded SOL still receives the whole technical basis; Token surplus remains outside that basis', () => {
  const prefund = [R(1014) - 2n, 1n, 0n, R(84) + 5n, 0n, 0n, 0n, 0n, 0n,
    67n, 8n, 100000n, 0n, 0n, R(165) + 3n, R(165) + 11n];
  const quote = client.genesisQuote(quoteInput(true, prefund));
  assert.deepEqual(quote.balances.slice(9, 14).map(row => row.funded), Array(5).fill('329600'));
  assert.equal(quote.balances[9].total, (329600n + 67n + 3n + 11n).toString());
  assert.equal(quote.balances[10].total, '329608');
  assert.equal(quote.balances[11].total, '429600');
  assert.equal(quote.balances[14].total, R(165).toString());
  assert.equal(quote.balances[15].total, R(165).toString());
  assert.equal(quote.tokenPrefundToPendingLamports, '14');
  assert(quote.advisory.some(x => x.startsWith('Economic SOL')));
  const wrong = quoteInput(true, []);
  wrong.targets[9].owner = IDS.token;
  assert.throws(() => client.genesisQuote(wrong), /unexpected owner/);
  wrong.targets[9].owner = IDS.system;
  wrong.targets[9].lamports = '18446744073709551615';
  assert.throws(() => client.genesisQuote(wrong), /final balance.*overflow/);
  wrong.targets[9].lamports = '0';
  wrong.targets[14].lamports = '18446744073709551615';
  wrong.targets[15].lamports = '18446744073709551615';
  assert.throws(() => client.genesisQuote(wrong), /Token prefund normalization.*overflow/);
});

test('MR02 and MR03 bind the round layouts, payer roles and exact runtime storage shortfalls', () => {
  const first = {snapshot: snapshot(1), nextRoundRentLamports: R(924).toString(),
    operationalLamports: '1000000', liveSolRentLamports: R(0).toString()};
  const migration2 = client.roundV2(first);
  assert.equal(migration2.instruction.dataHex, Buffer.from('PIV1MR02\x01', 'binary').toString('hex'));
  assert.equal(migration2.shortfallLamports, '84975');
  assert.deepEqual(migration2.instruction.accounts.map(x => x.role), [
    'config', 'active_distribution', 'pending_sol_vault', 'principal_sol_queue', 'operational_sol_vault',
    'distribution_escrow', 'kif_sol_vault', 'principal_jito_vault', 'pending_jito_vault', 'system_program',
  ]);
  assert.deepEqual(migration2.instruction.accounts.flatMap(({writable}, i) => writable ? [i] : []), [1, 4]);
  assert(!migration2.instruction.accounts.some(({signer}) => signer));
  assert.throws(() => client.roundV2({...first, operationalLamports: '827751'}), /preserve operational storage/);
  assert.throws(() => client.roundV2({...first, liveSolRentLamports: '650241'}), /current rent exceeds/);

  const manifest = fixture({vault: 0});
  const second = {snapshot: snapshot(2), nextRoundRentLamports: R(1199).toString(), liveSolRentLamports: R(0).toString(),
    multisigAddress: manifest.governance.multisigAddress, transactionIndex: '84',
    currentGuardianKeys: manifest.governance.guardians, payerAddress: manifest.rentPayer,
    payerLamports: '1037725', payerCapLamports: '708125'};
  const migration3 = client.roundV3(second);
  const bytes = Buffer.from(migration3.instruction.dataHex, 'hex');
  assert.equal(bytes.length, 50);
  assert.equal(bytes.subarray(0, 8).toString(), 'PIV1MR03');
  assert.deepEqual([...bytes.subarray(8, 10)], [1, 0]);
  assert.deepEqual(bytes.subarray(10, 42), crypto.createHash('sha256').update(witness('round-v2.bin')).digest());
  assert.equal(bytes.readBigUInt64LE(42), 708125n);
  assert.equal(migration3.shortfallLamports, '708125');
  const keys = migration3.instruction.accounts;
  assert.equal(keys.length, 26);
  assert.equal(keys[16].address, second.snapshot.distribution.address);
  assert.equal(keys[25].address, manifest.rentPayer);
  assert.deepEqual(keys.flatMap(({writable}, i) => writable ? [i] : []), [16, 25]);
  assert.deepEqual(keys.flatMap(({signer}, i) => signer ? [i] : []), [5, 25]);
  const [vault0] = PublicKey.findProgramAddressSync([Buffer.from('multisig'), new PublicKey(second.multisigAddress).toBuffer(),
    Buffer.from('vault'), Buffer.from([0])], new PublicKey(IDS.squads));
  assert.equal(keys[5].address, vault0.toBase58());
  assert.throws(() => client.roundV3({...second, snapshot: snapshot(3)}), /round version/);
  assert.throws(() => client.roundV3({...second, snapshot: legacy()}), /Config V2/);
  assert.throws(() => client.roundV3({...second, payerCapLamports: '708124'}), /external funding insufficient/);
  assert.throws(() => client.roundV3({...second, payerLamports: '1037724'}), /external funding insufficient/);
  const completed = state.inspectSnapshot(snapshot(3));
  assert.equal(completed.inspection.accounts.distribution.layout.allocationBytes, 1199);
  assert.equal(completed.inspection.decoded.distribution.lifecycle, 'Idle');
});

test('direct-JitoSOL-first uses native DJ01 → IB01 roles in one unsigned legacy packet', () => {
  for (const shared of [true, false]) {
    const initFixture = fixture({shared, vault: 0});
    const context = {...TEMPLATE, programId: initFixture.programId, feePayer: initFixture.feePayer,
      transport: {recentBlockhash: initFixture.transport.recentBlockhash, lastValidBlockHeight: '67', referenceSlot: '42'},
      resources: {computeUnits: 500000, heapBytes: 262144, loadedAccountBytes: 8388608}};
    const contribution = {...context, action: 'contribute-jitosol',
      jitosol: {owner: initFixture.governance.creator, sourceToken: publicPoints()[17], amountUnits: '9007199254740993'}};
    const bootstrap = {...context, action: 'bootstrap-initial', bootstrap: {...initFixture.protocol}};
    const pair = client.jitoGenesisPair({contribution, bootstrap});
    const dj = Buffer.from(pair.contribution.dataHex, 'hex');
    assert.equal(dj.subarray(0, 8).toString(), 'PIV1DJ01');
    assert.equal(dj[8], 1);
    assert.equal(dj.readBigUInt64LE(9), 9007199254740993n);
    assert.deepEqual(pair.contribution.accounts.map(role => role.role), ['config','active_distribution','pending_sol','pending_token','source_token','owner','mint','token_program']);
    assert.equal(pair.bootstrap.dataHex, Buffer.from('PIV1IB01\x01', 'binary').toString('hex'));
    assert.equal(pair.bootstrap.accounts.length, shared ? 18 : 19);
    assert.equal(pair.bootstrap.accounts.filter(role => role.signer).length, 0);
    const transaction = VersionedTransaction.deserialize(Buffer.from(pair.packet.wireBase64, 'base64'));
    assert.equal(transaction.signatures.length, 2);
    assert(transaction.signatures.every(signature => signature.every(byte => byte === 0)));
    assert.deepEqual(pair.packet.wireInstructions.slice(-2).map(ix => ix.dataHex), [pair.contribution.dataHex, pair.bootstrap.dataHex]);
    assert(pair.packet.wireBytes <= 1232);
    assert.throws(() => client.jitoGenesisPair({contribution, bootstrap: {...bootstrap, cluster: 'mainnet'}}), /unsupported_cluster/);
    assert.throws(() => client.jitoGenesisPair({contribution: {...contribution, jitosol: {...contribution.jitosol, sourceToken: bootstrap.bootstrap.reserve}}, bootstrap}), /aliases/);
  }
});

test('CLI reads the retained legacy snapshot and refuses unsupported operational flags', () => {
  const run = (...args) => spawnSync(process.execPath, [path.join(__dirname, 'rent-v2-client.cjs'), ...args], {encoding: 'utf8'});
  const result = run('--inspect', path.join(__dirname, 'fixtures', 'testnet-v1.json'));
  assert.equal(result.status, 0, result.stderr);
  assert.equal(JSON.parse(result.stdout).inspection.decoded.config.version, 1);
  for (const flag of ['--rpc', '--sign', '--send', '--upgrade', '--migrate']) {
    const outcome = run(flag);
    assert.equal(outcome.status, 1);
    assert.equal(outcome.stdout, '');
  }
});
