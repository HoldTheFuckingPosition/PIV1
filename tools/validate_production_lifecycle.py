#!/usr/bin/python3
"""Complete production lifecycle through real external programs in unsigned local Bank.

Compose hash-bound existing guards, preserving historical evidence. Only the fixed
Task 2.38 compiler cache may change; each command log and runnable binary is new.
No signing, deployment, secrets or live operations.
"""
import argparse
import importlib.util
import hashlib
import json
import os
from pathlib import Path
import re
import shutil

ROOT = Path('/home/jerem/piv1')
SCRIPT = ROOT / 'tools/validate_production_lifecycle.py'
PINS = ROOT / 'tools/production_lifecycle_pins.json'
OLD_BUILD = Path('/tmp/piv1-bank-smoke-genesis-build-t238-20260928-a')
CACHE = OLD_BUILD / 'target'
TEST = 'complete_delayed_production_lifecycle'
TESTS = {'positive': TEST, 'cooldown-loss': 'cooldown_loss_recovery_production_lifecycle',
         'early-pool-loss': 'early_pool_loss_recovery_production_lifecycle',
         'settlement-pool-loss': 'settlement_pool_loss_recovery_production_lifecycle',
         'repeated-cycles': 'repeated_cycles_production_lifecycle',
         'valid-insufficiency': 'valid_insufficiency_production_lifecycle',
         'cooldown-reward': 'cooldown_reward_production_lifecycle',
         'post-cycle-compounding': 'post_cycle_compounding_production_lifecycle',
         'positive-withdrawal-fee': 'positive_withdrawal_fee_production_lifecycle'}
COMPLETIONS = {'positive': 'lifecycle_complete', 'cooldown-loss': 'cooldown_loss_complete',
               'early-pool-loss': 'early_pool_loss_complete',
               'settlement-pool-loss': 'settlement_pool_loss_complete',
               'repeated-cycles': 'repeated_cycles_complete',
               'valid-insufficiency': 'valid_insufficiency_complete',
               'cooldown-reward': 'cooldown_reward_complete',
               'post-cycle-compounding': 'post_cycle_compounding_complete',
               'positive-withdrawal-fee': 'positive_withdrawal_fee_complete'}
PROGRAMS = ['PIV1', 'System', 'Token8.0.0', 'SPLpool2.0.3', 'Stake5.1.0']
LOSS_COMPLETION = {
    'status': 'PASS', 'legs': 2, 'finalized_legs': 2, 'epoch': 14,
    'loss_lamports': 1, 'recovery_state': 4, 'recovery_flags': 1,
    'protected_hwm': 119_000_000_000, 'pending_sol': 300_000_000,
    'pending_tokens': 500_000_000, 'recovered_stake_rent': 4_565_760,
    'recovered_metadata_rent': 5_442_720, 'temporary_accounts_closed': 4,
    'beneficiary_paid': 0, 'kif_claimed': 0, 'synthetic_external_loss': True,
    'synthetic_loss_conservation': True, 'live_ready': False,
    'signature_verified': False, 'actual_programs': PROGRAMS,
}

SEVERE_COMMON = dict(status='PASS', legs=2, epoch=14, pool_total=4_565_760,
    pool_supply=1_092_108_750_001, pending_used=100_000_000, cooldown_loss_lamports=0,
    recovery_state=4, recovery_flags=2, protected_hwm=119_000_000_000,
    pending_sol=300_000_000, pending_tokens=500_000_000, beneficiary_paid=0,
    kif_claimed=0, synthetic_external_loss=True, synthetic_loss_conservation=True,
    live_ready=False, signature_verified=False, actual_programs=PROGRAMS)
SEVERE_COMPLETIONS = {
    'early-pool-loss': dict(SEVERE_COMMON, finalized_legs=1, residual_value=464_510,
        recovered_stake_rent=2_282_880, recovered_metadata_rent=2_721_360,
        temporary_accounts_closed=2, outstanding_leg=0, outstanding_stake_lamports=5_002_282_880,
        outstanding_metadata_lamports=2_721_360, companion_unchanged=True, zero_floor_executed=False),
    'settlement-pool-loss': dict(SEVERE_COMMON, finalized_legs=2, token_value=464_510,
        projected_retained=464_512, observed_protected_value=0, recovery_event_verified=True,
        zero_floor_executed=True, recovered_stake_rent=4_565_760,
        recovered_metadata_rent=5_442_720, temporary_accounts_closed=4),
}



CYCLE_COMPLETION = dict(status='PASS', rounds=3, periods=[0, 1, 2], active_counts=[0, 0, 3],
    final_hwm=27_159_000_000, pending_sol=0, pending_tokens=0, kif_carry=2,
    kif_liability=66_499_998, zero_active_compound=47_500_000, cadence_seconds=864_000,
    cadence_error=6038, active_round_error=6034, synthetic_clock=True, real_walltime=False,
    live_ready=False, signature_verified=False, stake_executed=False,
    actual_programs=['PIV1', 'System', 'Token8.0.0', 'SPLpool2.0.3', 'synthetic-caller'],
    loaded_programs=['PIV1', 'synthetic-caller', 'Token8.0.0', 'SPLpool2.0.3', 'Stake5.1.0'])



INSUFFICIENCY_COMPLETION = dict(status='PASS', completed_rounds=1, final_round_state=1,
    final_sequence=1, next_sequence=2, prepared_at=1_814_401, insufficient_at=1_814_400,
    prior_prepared_at=864_000, retry_seconds=86_400, cadence_seconds=864_000,
    insufficiency_target=665_000_000, insufficiency_minimum=869_565_218,
    sufficient_target=1_274_583_333, sufficient_minimum=833_333_334, round_minimum=1_529_347_049,
    final_hwm=22_389_500_000, proposed_hwm=22_760_000_000, pending_sol=1, pending_tokens=0,
    escrow=1, kif_carry=19_000_000, withdrawal_legs_created=0, stake_minimum=1_000_000_000,
    synthetic_clock=True, real_walltime=False, live_ready=False, signature_verified=False,
    actual_programs=['PIV1', 'System', 'Token8.0.0', 'SPLpool2.0.3', 'Stake5.1.0', 'synthetic-caller'])


COMPOUND_COMPLETION = dict(status='PASS', completed_rounds=3, prior_sequence=2, next_sequence=3,
    deposited_sol=2_458_999_998, minted_tokens=1_891_538_460, historical_sol=2,
    historical_tokens=20_891_538_460, final_hwm=27_159_000_000, historical_value=27_159_000_000,
    pool_total=1_302_458_999_998, pool_supply=1_001_891_538_460, kif_liability=66_499_998,
    kif_carry=2, pending_sol=0, pending_tokens=0, next_yield=0, prior_summary_unchanged=True,
    rent_unchanged=True, final_state=0, synthetic_clock=True, real_walltime=False,
    live_ready=False, signature_verified=False, stake_executed=False,
    actual_programs=['PIV1', 'System', 'Token8.0.0', 'SPLpool2.0.3', 'synthetic-caller'],
    loaded_programs=['PIV1', 'synthetic-caller', 'Token8.0.0', 'SPLpool2.0.3', 'Stake5.1.0'])


def verify_compound_records(records, completion):
    verify_cycle_records(records, completion)
    def one(kind, expected):
        rows = [(i, row) for i, row in enumerate(records) if row['kind'] == kind]
        require(len(rows) == 1 and all(exact_typed(rows[0][1].get(key), value)
            for key, value in expected.items()), 'missing or invalid ' + kind)
        return rows[0][0]
    ready = one('compound_ready', dict(completed_rounds=3, prior_sequence=2, next_sequence=3,
        pool_total=1_300_000_000_000, pool_supply=1_000_000_000_000, historical_sol=2_459_000_000,
        historical_tokens=19_000_000_000, hwm=27_159_000_000, kif_liability=66_499_998,
        kif_carry=2, full_account_oracle=True))
    rejects = [(i, row) for i, row in enumerate(records) if row['kind'] == 'compound_rejection']
    require(len(rejects) == 2, 'missing compound rejections')
    for (_, row), (case, amount, minimum, code) in zip(rejects, [
            ('compound-minimum-reject', 2_458_999_998, 1_891_538_461, 6032),
            ('compound-queue-reject', 2_459_000_001, 0, 6028)]):
        expected = dict(case=case, amount=amount, minimum=minimum, error=code,
            no_cpi=True, no_events=True, fee_only=True, identical_replay_no_fee=True)
        require(all(exact_typed(row.get(key), value) for key, value in expected.items()),
            'invalid compound rejection')
    attempts = [(i, row) for i, row in enumerate(records) if row['kind'] == 'compound_attempt']
    require(len(attempts) == 2, 'missing compound attempts')
    for (_, row), (case, late) in zip(attempts, [('compound-late-failure', True), ('compound-success', False)]):
        expected = dict(case=case, late_outer=late, committed=not late, event_verified=True,
            protected_cpi_verified=True, full_account_oracle=True, prior_summary_unchanged=True,
            identical_replay_no_fee=late)
        require(all(exact_typed(row.get(key), value) for key, value in expected.items()),
            'invalid compound attempt')
        payload = row.get('event_data_hex')
        require(isinstance(payload, str) and re.fullmatch('[0-9a-f]{144}', payload), 'invalid compound event encoding')
        payload = bytes.fromhex(payload)
        require(payload[:8] == hashlib.sha256(b'event:PendingSolStaked').digest()[:8]
            and [int.from_bytes(payload[i:i + 8], 'little') for i in range(40, 72, 8)]
                == [2_458_999_998, 1_891_538_460, 27_159_000_000, 27_159_000_000],
            'invalid compound event amounts')
    order = [ready] + [i for i, _ in rejects] + [i for i, _ in attempts]
    last_cycle = max(i for i, row in enumerate(records) if row['kind'] == 'cycle_complete')
    final = next(i for i, row in enumerate(records) if row['kind'] == 'post_cycle_compounding_complete')
    require(last_cycle < ready and order == sorted(order) and order[-1] < final,
        'compound lifecycle out of order')
    require(attempts[0][1]['event_data_hex'] == attempts[1][1]['event_data_hex'], 'compound event identity drift')


REWARD_COMPLETION = dict(status='PASS', completed_rounds=2, epoch=14, legs=2,
    synthetic_reward=10_000, inflation_executed=False, first_hwm=121_955_000_003,
    final_hwm=121_955_001_953, later_gross=10_000, later_htfp=5900, later_compound=1950,
    later_team=1950, later_kif=200, next_yield=0, cumulative_reward=10_000,
    pending_sol=0, pending_tokens=0, kif_liability=216_363_836, kif_carry=0,
    recovered_stake_rent=4_565_760, recovered_metadata_rent=5_442_720, final_state=0,
    synthetic_clock=True, live_ready=False, signature_verified=False, actual_programs=PROGRAMS)
REWARD_ROWS = {
    'external_cooldown_reward': dict(synthetic=True, inflation_executed=False,
        credits_observed_unchanged=True, lamports=10_000, delegation_increment=10_000,
        donor_debit=10_000, epoch=0, before_cooldown=True, original_delegated=5_000_000_000,
        updated_delegated=5_000_010_000, stake_rent=2_282_880,
        all_piv_accounts_unchanged=True, capitalization_unchanged=True, full_account_oracle=True),
    'cooldown_reward_finalized': dict(epoch=14, reward=10_000, original_total=8_608_636_361,
        recovered_stake_rent=4_565_760, recovered_metadata_rent=5_442_720,
        fixed_snapshot_unchanged=True, full_account_oracle=True),
    'cooldown_reward_retained': dict(next_yield=10_000, cumulative_reward=10_000,
        historical_value=121_955_000_003, hwm=121_955_000_003, historical_sol=200_000_002,
        principal_sol=200_010_002, paid_htfp=6_382_727_270, paid_team=2_109_545_453,
        paid_kif=216_363_636, full_account_oracle=True),
    'cooldown_reward_later_split': dict(gross=10_000, historical_surplus=0,
        prior_next_yield=10_000, prior_yield_used=8050, htfp=5900, compound=1950,
        team=1950, kif=200, dust=0, active_count=1, period=1, new_legs=0, full_account_oracle=True),
}


def verify_reward_records(records, completion):
    require(type(completion.get('transactions')) is int and completion['transactions'] > 0,
            'missing reward transaction count')
    require(not any(row['kind'] in {'external_cooldown_loss', 'external_pool_loss'} for row in records),
            'loss stimulus in reward profile')
    positions = []
    for kind, expected in REWARD_ROWS.items():
        found = [(i, row) for i, row in enumerate(records) if row['kind'] == kind]
        require(len(found) == 1 and all(exact_typed(found[0][1].get(key), value)
            for key, value in expected.items()), 'missing or invalid ' + kind)
        positions.append(found[0][0])
    require(positions == sorted(positions), 'reward lifecycle out of order')
    clocks = [row for row in records if row['kind'] == 'cycle_clock']
    expected_clock = dict(case='reward-next-cycle-time', timestamp=2_592_000, epoch=14,
        synthetic=True, real_walltime=False, only_clock_timestamp_changed=True,
        economic_accounts_unchanged=True)
    require(len(clocks) == 1 and all(exact_typed(clocks[0].get(key), value)
        for key, value in expected_clock.items()), 'missing reward next-cycle clock')


def verify_insufficiency_records(records, completion):
    require(type(completion.get('transactions')) is int and completion['transactions'] > 0,
            'missing insufficiency transaction count')
    require(not any(row['kind'] in {'external_cooldown_loss', 'external_pool_loss'} for row in records),
            'loss stimulus in normal insufficiency profile')
    def exact_rows(kind, expected):
        rows = [row for row in records if row['kind'] == kind]
        require(len(rows) == len(expected), 'missing or duplicate ' + kind)
        for row, fields in zip(rows, expected):
            require(all(exact_typed(row.get(key), value) for key, value in fields.items()),
                    'invalid ' + kind)
        return rows
    clocks = [('primer-time', 100, 864_000), ('cadence-before', 864_000, 1_727_999),
        ('first-time', 1_727_999, 1_728_000), ('before-retry', 1_728_000, 1_814_399),
        ('at-retry', 1_814_399, 1_814_400), ('sufficient-time', 1_814_400, 1_814_401)]
    exact_rows('cycle_clock', [dict(case='insufficiency-' + case, prior_timestamp=prior,
        timestamp=time, slot=1, epoch=0, synthetic=True, real_walltime=False,
        only_clock_timestamp_changed=True, economic_accounts_unchanged=True) for case, prior, time in clocks])
    attempts = [('ten-day-reject', 'Custom(6038)', 1_727_999, 0, None),
        ('malformed-initial', 'InvalidInstructionData', 1_728_000, 0, None),
        ('first', 'Ok', 1_728_000, 1, 1_728_000),
        ('immediate', 'Custom(6039)', 1_728_000, 1, 1_728_000),
        ('before-boundary', 'Custom(6039)', 1_814_399, 1, 1_728_000),
        ('malformed-boundary', 'InvalidInstructionData', 1_814_400, 0, 1_728_000),
        ('exact-retry', 'Ok', 1_814_400, 1, 1_814_400)]
    exact_rows('insufficiency_attempt', [dict(case='insufficiency-' + case, status=status,
        timestamp=time, minimum_query_cpis=queries, economic_cpis=0,
        full_account_oracle=True, identical_replay_no_fee=True, prepared_at=864_000,
        insufficient_at=last) for case, status, time, queries, last in attempts])
    exact_rows('insufficiency_yield_input', [dict(case='insufficiency-' + case,
        external_reserve_transfer=amount, explicit_pending_sol=pending, pool_total=total,
        pool_supply=1_000_000_000_000, real_pool_update=True, live_yield_proof=False,
        full_account_oracle=True) for case, amount, pending, total in [
        ('primer-yield', 100_000_000_000, 2_000_000_000, 1_100_000_000_000),
        ('small-yield', 50_000_000_000, 0, 1_150_000_000_000),
        ('sufficient-yield', 50_000_000_000, 1, 1_200_000_000_000)]])
    events = exact_rows('valid_insufficiency_event', [dict(case='insufficiency-' + case,
        attempted_at=time, target=665_000_000, minimum=869_565_218,
        config_only=True, prepared_at=864_000, round_idle=True)
        for case, time in [('first', 1_728_000), ('exact-retry', 1_814_400)]])
    config = None
    for row in events:
        encoded = row.get('event_data_hex')
        require(isinstance(encoded, str) and re.fullmatch('[0-9a-f]{128}', encoded),
                'malformed insufficiency event')
        payload = bytes.fromhex(encoded)
        require(payload[:8] == hashlib.sha256(b'event:DistributionPreparationInsufficient').digest()[:8]
            and payload[8:40] != bytes(32)
            and payload[40:] == b''.join(value.to_bytes(8, 'little') for value in
                [row['attempted_at'], 665_000_000, 869_565_218])
            and (config is None or payload[8:40] == config), 'invalid insufficiency event payload')
        config = payload[8:40]


def exact_typed(actual, expected):
    if type(actual) is not type(expected):
        return False
    if isinstance(expected, list):
        return len(actual) == len(expected) and all(exact_typed(a, e) for a, e in zip(actual, expected))
    return actual == expected


def verify_cycle_records(records, completion):
    require(type(completion.get('transactions')) is int and completion['transactions'] > 0,
            'missing repeated-cycle transaction count')
    require(not any(row['kind'] in {'external_cooldown_loss', 'external_pool_loss'} for row in records),
            'loss stimulus in repeated-cycle profile')
    cycles = [row for row in records if row['kind'] == 'cycle_complete']
    require(len(cycles) == 3, 'missing repeated-cycle summaries')
    for index, row in enumerate(cycles):
        expected = dict(sequence=index, period=index, prepared_at=[1_728_000, 2_592_000, 5_184_000][index],
            completed_at=[1_814_400, 2_592_000, 5_184_000][index], active_count=[0, 0, 3][index],
            gross=1_900_000_000, kif_current=38_000_000, kif_carry_input=[0, 19_000_000, 28_500_000][index],
            kif_zero_compound=[19_000_000, 28_500_000, 0][index], kif_carry_next=[19_000_000, 28_500_000, 2][index],
            new_liability=[0, 0, 66_499_998][index], hwm=[22_389_500_000, 24_788_500_000, 27_159_000_000][index],
            pending_sol=0, pending_tokens=0, full_account_oracle=True)
        require(all(exact_typed(row.get(key), value) for key, value in expected.items()),
                'invalid repeated-cycle accounting')
    clocks = [row for row in records if row['kind'] == 'cycle_clock']
    expected_clocks = [('cycle-0-time', 100, 1_728_000),
        ('cycle-delayed-settlement', 1_728_000, 1_814_400),
        ('cycle-cadence-clock-before', 1_814_400, 2_591_999),
        ('cycle-1-time', 2_591_999, 2_592_000), ('cycle-2-time', 2_592_000, 5_184_000)]
    require(len(clocks) == len(expected_clocks), 'missing declared Clock stimulus')
    for row, (case, prior, timestamp) in zip(clocks, expected_clocks):
        expected = dict(case=case, prior_timestamp=prior, timestamp=timestamp, slot=1, epoch=0,
            synthetic=True, real_walltime=False, only_clock_timestamp_changed=True, economic_accounts_unchanged=True)
        require(all(exact_typed(row.get(key), value) for key, value in expected.items()),
                'invalid Clock stimulus provenance')
    rejections = [row for row in records if row['kind'] == 'cycle_rejection']
    require(len(rejections) == 2, 'missing cadence/active-round rejections')
    for row, (case, code) in zip(rejections, [('cycle-already-active', 6034), ('cycle-cadence-before', 6038)]):
        expected = dict(case=case, error=code, no_cpi=True, fee_only=True, identical_replay_no_fee=True)
        require(all(exact_typed(row.get(key), value) for key, value in expected.items()),
                'invalid cadence/active-round guard')


def require(value, message):
    if not value:
        raise RuntimeError(message)


def digest(path):
    import hashlib
    require(path.is_file() and not path.is_symlink(), f'nonregular input: {path}')
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def load(name, pins):
    path = ROOT / ('tools/' + name + '.py')
    require(digest(path) == pins['helpers'][str(path.relative_to(ROOT))], 'helper drift')
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def verify(pins):
    for relative, expected in pins['sources'].items():
        require(digest(ROOT / relative) == expected, f'source drift: {relative}')
    for relative, expected in pins['helpers'].items():
        require(digest(ROOT / relative) == expected, f'helper/pin drift: {relative}')
    for entry in pins['preserved']:
        path = Path(entry['path'])
        require(path == path.resolve(strict=True) and digest(path) == entry['sha256']
                and path.stat().st_size == entry['bytes'], f'historical evidence drift: {path}')
    for path in [CACHE, OLD_BUILD / 'openssl-config']:
        require(path.is_dir() and path == path.resolve(strict=True)
                and path.stat().st_uid == os.getuid(), 'untrusted compiler cache/config path')
    require(not list((OLD_BUILD / 'openssl-config').iterdir()), 'old OpenSSL config must remain empty')


def bank_profile(pins):
    old = json.loads((ROOT / 'tools/bank_genesis_pins.json').read_text())
    old['sources'] = pins['sources']
    return old


def strict_diagnostics(output):
    for path in sorted(output.glob('*.stderr')):
        require(not re.search(r'(^|\s)(warning|error)(\[|:)', path.read_text(), re.I),
                f'diagnostics require review: {path}')


def cached_command(command):
    return ['/usr/bin/env', f'CARGO_TARGET_DIR={CACHE}',
            f'OPENSSL_CONFIG_DIR={OLD_BUILD / "openssl-config"}', *command]


def build_bank(h, output, pins, result):
    prior = bank_profile(pins)
    cargo = str(h.TOOLCHAIN / 'cargo')
    common = ['--manifest-path', str(h.WORK / 'Cargo.toml'), '--locked', '--offline']
    h.monitored(cached_command([cargo, 'metadata', *common, '--format-version', '1',
        '--filter-platform', 'x86_64-unknown-linux-gnu']), output, 'metadata', 60)
    require(json.loads((output / 'metadata.stdout').read_text())['resolve'] == prior['resolve'],
            'Bank dependency/features changed')
    h.monitored(cached_command([cargo, 'test', *common, '--jobs', '1', '--test',
        'production_lifecycle', '--no-run', '--message-format=json']), output, 'build', 1800)
    messages = [json.loads(line) for line in (output / 'build.stdout').read_text().splitlines() if line.startswith('{')]
    require(not [m for m in messages if m.get('reason') == 'compiler-message'
        and m['message']['level'] in ['warning', 'error', 'failure-note']], 'compiler diagnostics')
    strict_diagnostics(output)
    paths = [Path(m['executable']) for m in messages if m.get('reason') == 'compiler-artifact'
        and m['target']['name'] == 'production_lifecycle' and m.get('executable')]
    require(len(paths) == 1, 'expected one production Bank binary')
    binary = paths[0]
    require(binary == binary.resolve(strict=True) and binary.parent == CACHE / 'debug/deps'
            and re.fullmatch('production_lifecycle-[a-f0-9]{16}', binary.name), 'untrusted cached executable')
    fresh = output / 'production_lifecycle'
    with binary.open('rb') as source, fresh.open('xb') as target:
        shutil.copyfileobj(source, target)
    fresh.chmod(0o700)
    require(digest(binary) == digest(fresh), 'binary copy mismatch')
    result.update(status='BUILD_PASS', binary={'path': str(fresh), 'sha256': digest(fresh)},
        reused_cache=str(CACHE), limits='Cached dependencies reused; no full fresh dependency rebuild claim.')


def reviewed_result(path, pins_hash, status):
    require(path == path.resolve(strict=True), 'aliased result')
    value = json.loads(path.read_text())
    require(value['status'] == status and value['pins_sha256'] == pins_hash
            and value['runner_sha256'] == digest(SCRIPT), 'reviewed result/input mismatch')
    return value


FEE_COMPLETION = dict(status='PASS', transactions=50, completed_rounds=1, epoch=14, legs=2,
    fee_numerator=1, fee_denominator=1000, fixed_target=7_891_249_999,
    total_fee=7_891_251, aggregate_fee_ceiling=7_891_250, burned=7_883_358_748,
    delegated_proceeds=8_600_027_724, net=8_700_027_724, paid_htfp=6_376_417_834,
    paid_team=2_107_460_131, paid_kif=216_149_757, compound=2_109_545_454,
    split_dust=2, net_dust=2, external_reserve_appreciation=3,
    final_hwm=121_955_000_003, historical_value=121_955_000_003,
    historical_sol=200_000_002, historical_tokens=111_608_750_001,
    pool_total=1_191_399_972_279, pool_supply=1_092_116_641_252,
    manager_tokens=980_007_891_251, pending_sol=0, pending_tokens=0,
    next_yield=0, kif_liability=216_149_757, kif_carry=0, kif_claimed=0,
    recovered_stake_rent=4_565_760, recovered_metadata_rent=5_442_720,
    temporary_accounts_closed=4, final_state=0, full_account_oracle=True,
    live_ready=False, signature_verified=False, actual_programs=PROGRAMS)


def verify_fee_records(records, completion):
    require(not any(row['kind'] in {'external_cooldown_loss', 'external_pool_loss',
        'external_cooldown_reward', 'cycle_clock'} for row in records),
        'unrelated synthetic stimulus in positive withdrawal-fee profile')
    fixtures = [(i, row) for i, row in enumerate(records) if row['kind'] == 'fixture']
    require(len(fixtures) == 1, 'missing unique fee genesis provenance')
    require(all(exact_typed(fixtures[0][1].get(k), v) for k, v in dict(
        profile='positive-withdrawal-fee', synthetic_external_pool=True,
        synthetic_governance=True, unsigned=True, economic_piv_seeded=False,
        withdrawal_fee_numerator=1, withdrawal_fee_denominator=1000).items()),
        'invalid fee genesis provenance')
    legs = [(i, row) for i, row in enumerate(records) if row['kind'] == 'withdrawal_fee_leg']
    require(len(legs) == 2, 'missing unique fee-bearing legs')
    expected = [dict(index=0, input=4_587_921_256, fee=4_587_922, burn=4_583_333_334,
        native=5_000_000_000, before_total=1_200_000_000_000, before_supply=1_100_000_000_000,
        after_total=1_195_000_000_000, after_supply=1_095_416_666_666, manager_credit=4_587_922),
        dict(index=1, input=3_303_328_743, fee=3_303_329, burn=3_300_025_414,
        native=3_600_027_724, before_total=1_195_000_000_000, before_supply=1_095_416_666_666,
        after_total=1_191_399_972_276, after_supply=1_092_116_641_252, manager_credit=3_303_329)]
    for (_, actual), wanted in zip(legs, expected):
        wanted.update(fee_numerator=1, fee_denominator=1000, full_account_oracle=True)
        require(all(exact_typed(actual.get(k), v) for k, v in wanted.items()), 'fee leg accounting mismatch')
    final = next(i for i, row in enumerate(records) if row['kind'] == 'positive_withdrawal_fee_complete')
    require(fixtures[0][0] < legs[0][0] < legs[1][0] < final, 'fee lifecycle evidence out of order')


def selected_test(profile):
    require(profile in TESTS, 'unknown lifecycle profile')
    return TESTS[profile]


def verify_runtime_output(stdout, profile='positive'):
    test = selected_test(profile)
    records = [json.loads(line.split('PIV1_LIFECYCLE_EVIDENCE ', 1)[1])
        for line in stdout.splitlines() if 'PIV1_LIFECYCLE_EVIDENCE ' in line]
    require(all(isinstance(row, dict) and isinstance(row.get('kind'), str)
                for row in records), 'malformed lifecycle evidence')
    kind = COMPLETIONS[profile]
    allowed_completions = ({kind, 'pause_profile_complete'} if profile == 'positive' else
                           {kind, 'cycle_complete'} if profile in {'repeated-cycles', 'post-cycle-compounding'} else {kind})
    require(all(row['kind'] in allowed_completions for row in records
                if row['kind'].endswith('_complete')), 'cross-profile or unknown completion evidence')
    complete = [row for row in records if row['kind'] == kind]
    expected = (dict(status='PASS', legs=2, pending_sol=0, pending_tokens=0,
                     live_ready=False, signature_verified=False, actual_programs=PROGRAMS)
                if profile == 'positive' else LOSS_COMPLETION if profile == 'cooldown-loss'
                else CYCLE_COMPLETION if profile == 'repeated-cycles'
                else COMPOUND_COMPLETION if profile == 'post-cycle-compounding'
                else FEE_COMPLETION if profile == 'positive-withdrawal-fee'
                else INSUFFICIENCY_COMPLETION if profile == 'valid-insufficiency'
                else REWARD_COMPLETION if profile == 'cooldown-reward' else SEVERE_COMPLETIONS[profile])
    require('test result: ok. 1 passed; 0 failed;' in stdout and
            f'test {test} ...' in stdout and len(complete) == 1 and
            all(exact_typed(complete[0].get(key), value)
                for key, value in expected.items()), 'missing actual lifecycle completion')
    if profile == 'positive-withdrawal-fee':
        verify_fee_records(records, complete[0])
    if profile == 'post-cycle-compounding':
        verify_compound_records(records, complete[0])
    if profile == 'repeated-cycles':
        verify_cycle_records(records, complete[0])
    if profile == 'valid-insufficiency':
        verify_insufficiency_records(records, complete[0])
    if profile == 'cooldown-reward':
        verify_reward_records(records, complete[0])
    if profile == 'positive':
        require(not any(row['kind'] in {'external_cooldown_loss', 'external_pool_loss'} for row in records),
                'external loss stimulus in positive profile')
    if profile == 'cooldown-loss':
        require(not any(row['kind'] == 'external_pool_loss' for row in records), 'pool shock in cooldown-loss profile')
        require(type(complete[0].get('transactions')) is int and complete[0]['transactions'] > 0,
                'missing loss-profile transaction count')
        shocks = [row for row in records if row['kind'] == 'external_cooldown_loss']
        require(len(shocks) == 1, 'missing unique declared external loss stimulus')
        shock = shocks[0]
        for key, value in dict(synthetic=True, production_instruction=False,
                              live_slashing_proof=False, lamports=1, epoch=0,
                              all_data_unchanged=True, all_piv_accounts_unchanged=True).items():
            require(type(shock.get(key)) is type(value) and shock[key] == value,
                    'invalid external loss provenance')
        require(type(shock.get('capitalization_before')) is int and
                shock['capitalization_before'] > 0 and
                type(shock.get('capitalization_after')) is int and
                shock['capitalization_before'] == shock['capitalization_after'],
                'unbalanced external loss stimulus')
    if profile in SEVERE_COMPLETIONS:
        require(type(complete[0].get('transactions')) is int and complete[0]['transactions'] > 0,
                'missing severe-profile transaction count')
        require(not any(row['kind'] == 'external_cooldown_loss' for row in records),
                'withdrawn-leg shock in severe pool profile')
        shocks = [row for row in records if row['kind'] == 'external_pool_loss']
        require(len(shocks) == 1, 'missing unique external pool-loss stimulus')
        shock = shocks[0]
        expected_shock = dict(synthetic=True, production_instruction=False, live_slashing_proof=False,
            lamports=1_191_386_797_879, epoch=14, all_data_unchanged=True, all_piv_accounts_unchanged=True,
            withdrawn_legs_unchanged=True, retained_rent_per_source=2_282_880)
        require(all(type(shock.get(key)) is type(value) and shock[key] == value
                    for key, value in expected_shock.items()), 'invalid severe loss provenance')
        require(type(shock.get('capitalization_before')) is int and shock['capitalization_before'] > 0
                and type(shock.get('capitalization_after')) is int
                and shock['capitalization_after'] == shock['capitalization_before'], 'unbalanced pool-loss stimulus')
        events = [row for row in records if row['kind'] == 'settlement_recovery_event']
        if profile == 'early-pool-loss':
            require(not events, 'early finalization cannot claim settlement zero-floor execution')
        else:
            expected_cases = {'severe-settlement-late-failure': False, 'severe-settlement': True}
            require(len(events) == 2 and {row.get('case') for row in events} == set(expected_cases),
                    'missing attempted and committed recovery events')
            event_config = None
            for event in events:
                require(type(event.get('transaction_succeeded')) is bool
                        and event['transaction_succeeded'] is expected_cases[event['case']]
                        and type(event.get('observed_protected_value')) is int
                        and event['observed_protected_value'] == 0
                        and type(event.get('recovery_flags')) is int and event['recovery_flags'] == 2,
                        'invalid recovery event result')
                encoded = event.get('event_data_hex')
                require(isinstance(encoded, str) and re.fullmatch('[0-9a-f]{114}', encoded), 'malformed recovery event data')
                payload = bytes.fromhex(encoded)
                require(payload[:8] == hashlib.sha256(b'event:DistributionSettlementRecovery').digest()[:8]
                        and payload[40:48] == bytes(8) and payload[48:56] == bytes(8)
                        and payload[56] == 2 and payload[8:40] != bytes(32), 'wrong recovery event payload')
                require(event_config is None or payload[8:40] == event_config, 'recovery event Config mismatch')
                event_config = payload[8:40]
    return complete[0]


def verify_artifacts(pins):
    require(set(pins['artifacts']) == {'piv', 'caller', 'token', 'pool', 'stake'}, 'unexpected program set')
    for entry in pins['artifacts'].values():
        path = Path(entry['path'])
        require(path == path.resolve(strict=True) and digest(path) == entry['sha256']
                and path.stat().st_size == entry['bytes'], 'program artifact drift')

def parse_args(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('stage', choices=['bank-build', 'bank-run'])
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--build-output', type=Path)
    parser.add_argument('--approved-sha256')
    parser.add_argument('--profile', choices=tuple(TESTS), default='positive')
    return parser.parse_args(argv)


def main():
    args = parse_args()
    pins = json.loads(PINS.read_text()); verify(pins); verify_artifacts(pins)
    h = load('validate_bank_smoke', pins); h.verify(bank_profile(pins))
    git = load('build_keyless_sbf', pins)
    before = git.git_state()
    require(before['user'] == 'jerem' and before['branch'] == 'integration/piv1-testnet'
            and before['head'] == pins['baseline'], 'wrong repository state')
    h.checked_output(args.output, fresh=True)
    for name in ['tmp', 'openssl-config']:
        (args.output / name).mkdir(mode=0o700)
    result = {'stage': args.stage, 'profile': args.profile, 'runner_sha256': digest(SCRIPT), 'pins_sha256': digest(PINS)}
    try:
        if args.stage == 'bank-build':
            build_bank(h, args.output, pins, result)
        else:
            built = reviewed_result(args.build_output / 'result.json', digest(PINS), 'BUILD_PASS')
            binary = Path(built['binary']['path'])
            require(binary == args.build_output / 'production_lifecycle'
                and binary == binary.resolve(strict=True)
                and digest(binary) == args.approved_sha256 == built['binary']['sha256'], 'unreviewed binary')
            artifacts = pins['artifacts']
            command = ['/usr/bin/env']
            for role, entry in artifacts.items():
                for field, suffix in [('path', 'PATH'), ('sha256', 'SHA256'), ('bytes', 'BYTES')]:
                    command.append(f'PIV_LIFECYCLE_{role.upper()}_{suffix}={entry[field]}')
            command.extend([str(binary), '--exact', selected_test(args.profile), '--test-threads=1', '--nocapture'])
            try:
                h.monitored(command, args.output, 'runtime', 300)
            finally:
                require(digest(binary) == args.approved_sha256, 'binary drift')
                for entry in artifacts.values():
                    path = Path(entry['path'])
                    require(digest(path) == entry['sha256'] and path.stat().st_size == entry['bytes'], 'runtime artifact drift')
            stdout = (args.output / 'runtime.stdout').read_text()
            result['completion'] = verify_runtime_output(stdout, args.profile)
            require((args.output / 'runtime.stderr').stat().st_size == 0, 'unexpected runtime stderr')
            require(digest(binary) == args.approved_sha256, 'binary drift')
            result.update(status='RUN_PASS', binary=built['binary'], artifacts=artifacts)
    except BaseException as error:
        result.update(status='FAIL', error=str(error))
        raise
    finally:
        try:
            verify(pins); verify_artifacts(pins)
            require(digest(SCRIPT) == result['runner_sha256'] and digest(PINS) == result['pins_sha256'], 'runner/pin drift')
            after = git.git_state()
            require(all(before[k] == after[k] for k in ['user', 'branch', 'head', 'refs']), 'protected Git drift')
            result['preservation'] = 'PASS'
        except BaseException as error:
            result.update(status='FAIL', preservation=str(error))
            raise
        finally:
            result['logs'] = {p.name: {'sha256': digest(p), 'bytes': p.stat().st_size}
                for p in args.output.iterdir() if p.suffix in ['.stdout', '.stderr']}
            h.write(args.output / 'result.json', result)
    print(json.dumps({'status': result['status'], 'output': str(args.output)}))


if __name__ == '__main__':
    main()
