"""Reject incomplete lifecycle evidence and altered executable inputs."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


SPEC = importlib.util.spec_from_file_location(
    'lifecycle_runner', Path(__file__).with_name('validate_production_lifecycle.py'))
RUNNER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RUNNER)


def module(name):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(name + '.py'))
    loaded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded


PROGRAMS = module('build_lifecycle_programs')
SBF = module('build_keyless_sbf')


class LifecycleEvidenceTests(unittest.TestCase):
    def completion(self, **changes):
        record = dict(kind='lifecycle_complete', status='PASS', legs=2,
                      pending_sol=0, pending_tokens=0, live_ready=False,
                      signature_verified=False,
                      actual_programs=['PIV1', 'System', 'Token8.0.0',
                                       'SPLpool2.0.3', 'Stake5.1.0'])
        record.update(changes)
        return 'PIV1_LIFECYCLE_EVIDENCE ' + json.dumps(record) + '\n'

    def transcript(self, record):
        return ('test complete_delayed_production_lifecycle ... ' + record
                + 'test result: ok. 1 passed; 0 failed;\n')

    def test_complete_record_requires_actual_test_success(self):
        with self.assertRaises(RuntimeError):
            RUNNER.verify_runtime_output(self.completion())

    def test_success_without_lifecycle_completion_rejects(self):
        with self.assertRaises(RuntimeError):
            RUNNER.verify_runtime_output(self.transcript(''))

    def test_incomplete_or_overclaimed_lifecycle_rejects(self):
        for change in [dict(legs=1), dict(pending_sol=1), dict(pending_tokens=1),
                       dict(live_ready=True), dict(signature_verified=True),
                       dict(actual_programs=['PIV1', 'synthetic Token'])]:
            with self.subTest(change=change), self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(self.transcript(self.completion(**change)))

    def test_ambiguous_duplicate_completion_rejects(self):
        with self.assertRaises(RuntimeError):
            RUNNER.verify_runtime_output(self.transcript(self.completion() * 2))

    def test_complete_scoped_evidence_parses(self):
        self.assertEqual(RUNNER.verify_runtime_output(
            self.transcript(self.completion()))['status'], 'PASS')

    def test_artifact_substitution_and_missing_program_reject(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'program.so'
            path.write_bytes(b'reviewed bytes')
            entry = dict(path=str(path), bytes=path.stat().st_size,
                         sha256=RUNNER.digest(path))
            pins = dict(artifacts={role: entry.copy()
                                  for role in ['piv', 'caller', 'token', 'pool', 'stake']})
            RUNNER.verify_artifacts(pins)
            pins['artifacts'].pop('stake')
            with self.assertRaises(RuntimeError):
                RUNNER.verify_artifacts(pins)
            pins['artifacts']['stake'] = entry.copy()
            path.write_bytes(b'altered  bytes')
            with self.assertRaises(RuntimeError):
                RUNNER.verify_artifacts(pins)


class CooldownLossEvidenceTests(unittest.TestCase):
    def records(self):
        # Independent literal contract, not copied from the runner's constants.
        completion = dict(kind='cooldown_loss_complete', status='PASS', transactions=90,
            legs=2, finalized_legs=2, epoch=14, loss_lamports=1, recovery_state=4,
            recovery_flags=1, protected_hwm=119_000_000_000, pending_sol=300_000_000,
            pending_tokens=500_000_000, recovered_stake_rent=4_565_760,
            recovered_metadata_rent=5_442_720, temporary_accounts_closed=4,
            beneficiary_paid=0, kif_claimed=0, synthetic_external_loss=True,
            synthetic_loss_conservation=True, live_ready=False, signature_verified=False,
            actual_programs=['PIV1', 'System', 'Token8.0.0', 'SPLpool2.0.3', 'Stake5.1.0'])
        shock = dict(kind='external_cooldown_loss', synthetic=True,
            production_instruction=False, live_slashing_proof=False, lamports=1,
            epoch=0, capitalization_before=1_000_000_000_000,
            capitalization_after=1_000_000_000_000,
            all_data_unchanged=True, all_piv_accounts_unchanged=True)
        return [shock, completion]

    def transcript(self, records, test='cooldown_loss_recovery_production_lifecycle'):
        return ('test ' + test + ' ... ' + '\n'.join(
            'PIV1_LIFECYCLE_EVIDENCE ' + json.dumps(row) for row in records)
            + '\ntest result: ok. 1 passed; 0 failed;\n')

    def test_fixed_profiles_select_exact_tests_and_loss_evidence(self):
        self.assertEqual(RUNNER.parse_args(['bank-build', '--output', '/tmp/unused']).profile, 'positive')
        self.assertEqual(RUNNER.parse_args(['bank-run', '--output', '/tmp/unused', '--profile', 'cooldown-loss']).profile, 'cooldown-loss')
        self.assertEqual(RUNNER.selected_test('positive'), 'complete_delayed_production_lifecycle')
        self.assertEqual(RUNNER.selected_test('cooldown-loss'), 'cooldown_loss_recovery_production_lifecycle')
        self.assertEqual(RUNNER.verify_runtime_output(self.transcript(self.records()),
            'cooldown-loss')['recovery_state'], 4)
        for profile in ['', 'loss', '../cooldown-loss', 'positive --ignored']:
            with self.subTest(profile=profile), self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(self.transcript(self.records()), profile)

    def test_cross_profile_missing_duplicate_or_wrong_test_rejects(self):
        records = self.records()
        for transcript in [self.transcript(records, 'complete_delayed_production_lifecycle'),
                           self.transcript(records[:1]), self.transcript(records + records[1:]),
                           self.transcript(records + [dict(kind='lifecycle_complete')]),
                           self.transcript(records + [dict(kind='invented_complete')])]:
            with self.subTest(transcript=transcript), self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(transcript, 'cooldown-loss')
        with self.assertRaises(RuntimeError):
            RUNNER.verify_runtime_output(self.transcript(records))
        positive = LifecycleEvidenceTests().completion()
        with self.assertRaises(RuntimeError):
            RUNNER.verify_runtime_output(LifecycleEvidenceTests().transcript(positive +
                'PIV1_LIFECYCLE_EVIDENCE ' + json.dumps(records[0]) + '\n'))

    def test_loss_economic_or_provenance_overclaims_and_wrong_types_reject(self):
        changes = [dict(loss_lamports=0), dict(recovery_state=2), dict(recovery_flags=3),
            dict(finalized_legs=1), dict(protected_hwm=118_999_999_999), dict(pending_sol=0),
            dict(pending_tokens=0), dict(recovered_stake_rent=4_565_759),
            dict(recovered_metadata_rent=0), dict(temporary_accounts_closed=3),
            dict(beneficiary_paid=1), dict(kif_claimed=1), dict(synthetic_external_loss=False),
            dict(synthetic_loss_conservation=False), dict(live_ready=True),
            dict(signature_verified=True), dict(loss_lamports=True), dict(epoch=14.0),
            dict(transactions=True), dict(transactions=0)]
        for change in changes:
            records = self.records(); records[1].update(change)
            with self.subTest(change=change), self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(self.transcript(records), 'cooldown-loss')

    def test_external_shock_must_be_unique_balanced_and_explicitly_synthetic(self):
        for change in [dict(synthetic=False), dict(production_instruction=True),
            dict(live_slashing_proof=True), dict(lamports=2), dict(epoch=14),
            dict(capitalization_after=999), dict(capitalization_before=0, capitalization_after=0),
            dict(all_data_unchanged=False), dict(all_piv_accounts_unchanged=False)]:
            records = self.records(); records[0].update(change)
            with self.subTest(change=change), self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(self.transcript(records), 'cooldown-loss')
        records = self.records()
        for altered in [records[1:], records + records[:1]]:
            with self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(self.transcript(altered), 'cooldown-loss')

    def test_malformed_evidence_rejects(self):
        for bad in [None, [], 42, {}, dict(kind=12)]:
            with self.subTest(bad=bad), self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(self.transcript(self.records() + [bad]), 'cooldown-loss')
        with self.assertRaises(ValueError):
            RUNNER.verify_runtime_output('PIV1_LIFECYCLE_EVIDENCE {bad json', 'cooldown-loss')


class SeverePoolLossEvidenceTests(unittest.TestCase):
    def records(self, profile):
        # Independent expected outcomes; do not import the runner's contracts.
        completion = dict(status='PASS', transactions=90, legs=2, epoch=14,
            pool_total=4_565_760, pool_supply=1_092_108_750_001, pending_used=100_000_000,
            cooldown_loss_lamports=0, recovery_state=4, recovery_flags=2,
            protected_hwm=119_000_000_000, pending_sol=300_000_000, pending_tokens=500_000_000,
            beneficiary_paid=0, kif_claimed=0, synthetic_external_loss=True,
            synthetic_loss_conservation=True, live_ready=False, signature_verified=False,
            actual_programs=['PIV1', 'System', 'Token8.0.0', 'SPLpool2.0.3', 'Stake5.1.0'])
        shock = dict(kind='external_pool_loss', synthetic=True, production_instruction=False,
            live_slashing_proof=False, lamports=1_191_386_797_879, epoch=14,
            all_data_unchanged=True, all_piv_accounts_unchanged=True, withdrawn_legs_unchanged=True,
            retained_rent_per_source=2_282_880, capitalization_before=2_000_000_000_000,
            capitalization_after=2_000_000_000_000)
        events = []
        if profile == 'early-pool-loss':
            completion.update(kind='early_pool_loss_complete', finalized_legs=1, residual_value=464_510,
                recovered_stake_rent=2_282_880, recovered_metadata_rent=2_721_360,
                temporary_accounts_closed=2, outstanding_leg=0, outstanding_stake_lamports=5_002_282_880,
                outstanding_metadata_lamports=2_721_360, companion_unchanged=True, zero_floor_executed=False)
        else:
            completion.update(kind='settlement_pool_loss_complete', finalized_legs=2, token_value=464_510,
                projected_retained=464_512, observed_protected_value=0, recovery_event_verified=True,
                recovered_stake_rent=4_565_760, recovered_metadata_rent=5_442_720,
                temporary_accounts_closed=4, zero_floor_executed=True)
            payload = (hashlib.sha256(b'event:DistributionSettlementRecovery').digest()[:8]
                       + bytes([17]) * 32 + bytes(16) + bytes([2]))
            for case, committed in [('severe-settlement-late-failure', False), ('severe-settlement', True)]:
                events.append(dict(kind='settlement_recovery_event', case=case,
                    transaction_succeeded=committed, observed_protected_value=0,
                    recovery_flags=2, event_data_hex=payload.hex()))
        return [shock, *events, completion]

    def transcript(self, profile, records):
        names = {'early-pool-loss': 'early_pool_loss_recovery_production_lifecycle',
                 'settlement-pool-loss': 'settlement_pool_loss_recovery_production_lifecycle'}
        return CooldownLossEvidenceTests().transcript(records, names[profile])

    def rejects(self, profile, records):
        with self.assertRaises(RuntimeError):
            RUNNER.verify_runtime_output(self.transcript(profile, records), profile)

    def test_exact_severe_profiles_accept_distinct_outcomes(self):
        for profile, test in [('early-pool-loss', 'early_pool_loss_recovery_production_lifecycle'),
                              ('settlement-pool-loss', 'settlement_pool_loss_recovery_production_lifecycle')]:
            with self.subTest(profile=profile):
                self.assertEqual(RUNNER.selected_test(profile), test)
                self.assertEqual(RUNNER.parse_args(['bank-run', '--output', '/tmp/unused', '--profile', profile]).profile, profile)
                result = RUNNER.verify_runtime_output(self.transcript(profile, self.records(profile)), profile)
                self.assertEqual(result['zero_floor_executed'], profile == 'settlement-pool-loss')

    def test_missing_duplicate_cross_profile_and_wrong_test_reject(self):
        for profile in ['early-pool-loss', 'settlement-pool-loss']:
            rows = self.records(profile)
            other = 'settlement-pool-loss' if profile == 'early-pool-loss' else 'early-pool-loss'
            for records in [rows[:-1], rows + rows[-1:], rows + self.records(other)[-1:],
                            rows + [dict(kind='invented_complete')], rows + CooldownLossEvidenceTests().records()[:1]]:
                with self.subTest(profile=profile, records=records):
                    self.rejects(profile, records)
            with self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(self.transcript(other, rows), profile)
            with self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(self.transcript(profile, rows))

    def test_exact_economics_rent_custody_and_types_reject_overclaims(self):
        common = [dict(pool_total=4_565_761), dict(pool_supply=1_092_108_750_000),
            dict(pending_used=0), dict(cooldown_loss_lamports=1), dict(recovery_state=2),
            dict(recovery_flags=1), dict(protected_hwm=0), dict(pending_sol=0), dict(pending_tokens=0),
            dict(beneficiary_paid=1), dict(kif_claimed=1), dict(epoch=14.0), dict(transactions=True),
            dict(transactions=0), dict(live_ready=True), dict(signature_verified=True)]
        special = {
            'early-pool-loss': [dict(finalized_legs=2), dict(residual_value=100_000_000),
                dict(recovered_stake_rent=4_565_760), dict(recovered_metadata_rent=0),
                dict(temporary_accounts_closed=4), dict(outstanding_leg=1),
                dict(outstanding_stake_lamports=0), dict(outstanding_metadata_lamports=0),
                dict(companion_unchanged=False), dict(zero_floor_executed=True)],
            'settlement-pool-loss': [dict(finalized_legs=1), dict(token_value=464_512),
                dict(projected_retained=100_000_000), dict(observed_protected_value=1),
                dict(recovery_event_verified=False), dict(recovered_stake_rent=0),
                dict(recovered_metadata_rent=0), dict(temporary_accounts_closed=2), dict(zero_floor_executed=False)]}
        for profile, changes in special.items():
            for change in common + changes:
                records = self.records(profile); records[-1].update(change)
                with self.subTest(profile=profile, change=change):
                    self.rejects(profile, records)

    def test_severe_shock_requires_exact_balanced_external_provenance(self):
        changes = [dict(lamports=1_191_386_797_878), dict(synthetic=False),
            dict(production_instruction=True), dict(live_slashing_proof=True), dict(epoch=0),
            dict(all_data_unchanged=False), dict(all_piv_accounts_unchanged=False),
            dict(withdrawn_legs_unchanged=False), dict(retained_rent_per_source=0),
            dict(capitalization_after=1), dict(capitalization_before=0, capitalization_after=0)]
        for profile in ['early-pool-loss', 'settlement-pool-loss']:
            for change in changes:
                records = self.records(profile); records[0].update(change)
                with self.subTest(profile=profile, change=change):
                    self.rejects(profile, records)
            records = self.records(profile)
            self.rejects(profile, records[1:])
            self.rejects(profile, records + records[:1])

    def test_actual_event_requires_attempt_and_commit_with_exact_payload(self):
        profile = 'settlement-pool-loss'
        changes = [dict(case='invented'), dict(transaction_succeeded=True),
            dict(transaction_succeeded=0), dict(observed_protected_value=1),
            dict(observed_protected_value=False), dict(recovery_flags=1),
            dict(event_data_hex='00'), dict(event_data_hex='x' * 114)]
        for change in changes:
            records = self.records(profile); records[1].update(change)
            with self.subTest(change=change):
                self.rejects(profile, records)
        # Alter discriminator, Config, sequence, protected value, or flags in the
        # actual serialized payload while keeping the synthetic summary intact.
        for offset in [0, 8, 40, 48, 56]:
            records = self.records(profile)
            data = bytearray.fromhex(records[1]['event_data_hex']); data[offset] ^= 1
            records[1]['event_data_hex'] = data.hex()
            with self.subTest(offset=offset):
                self.rejects(profile, records)
        records = self.records(profile)
        self.rejects(profile, [records[0], records[2], records[3]])
        self.rejects(profile, records + records[1:2])
        self.rejects('early-pool-loss', self.records('early-pool-loss') + records[1:2])


class IsolatedTargetAuditTests(unittest.TestCase):
    def fingerprint(self, root, prefix='target-token', package='solana-sysvar-id-0123456789abcdef',
                    filename='lib-solana_sysvar_id.json'):
        return root / prefix / SBF.TARGET / 'release' / '.fingerprint' / package / filename

    def test_exact_metadata_allowed_in_each_isolated_target(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for prefix in ['target-token', 'target-pool']:
                path = self.fingerprint(root, prefix)
                path.parent.mkdir(parents=True)
                path.write_text('{}')
            self.assertEqual(len([p for p in PROGRAMS.audit_outputs(SBF, root)
                                  if p.endswith('.json')]), 2)

    def test_other_prefix_or_metadata_identity_rejects(self):
        for changes in [dict(prefix='target-other'), dict(prefix='nested/target-token'),
                        dict(package='solana-sysvar-id-not-a-hash'), dict(filename='id.json')]:
            with self.subTest(changes=changes), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                path = self.fingerprint(root, **changes)
                path.parent.mkdir(parents=True)
                path.write_text('{}')
                with self.assertRaises(RuntimeError):
                    PROGRAMS.audit_outputs(SBF, root)

    def test_directory_or_symlink_cannot_use_metadata_exception(self):
        for symlink in [False, True]:
            with self.subTest(symlink=symlink), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                path = self.fingerprint(root)
                path.parent.mkdir(parents=True)
                if symlink:
                    path.symlink_to(root / 'does-not-exist')
                else:
                    path.mkdir()
                with self.assertRaises(RuntimeError):
                    PROGRAMS.audit_outputs(SBF, root)


class RepeatedCycleEvidenceTests(unittest.TestCase):
    def records(self):
        # Literal independent expected values, never imported runner contracts.
        records = []
        clocks = [('cycle-0-time', 100, 1_728_000),
            ('cycle-delayed-settlement', 1_728_000, 1_814_400),
            ('cycle-cadence-clock-before', 1_814_400, 2_591_999),
            ('cycle-1-time', 2_591_999, 2_592_000), ('cycle-2-time', 2_592_000, 5_184_000)]
        for case, prior, timestamp in clocks:
            records.append(dict(kind='cycle_clock', case=case, prior_timestamp=prior, timestamp=timestamp,
                slot=1, epoch=0, synthetic=True, real_walltime=False,
                only_clock_timestamp_changed=True, economic_accounts_unchanged=True))
        for case, code in [('cycle-already-active', 6034), ('cycle-cadence-before', 6038)]:
            records.append(dict(kind='cycle_rejection', case=case, error=code, no_cpi=True,
                fee_only=True, identical_replay_no_fee=True))
        for i in range(3):
            records.append(dict(kind='cycle_complete', sequence=i, period=i,
                prepared_at=[1_728_000, 2_592_000, 5_184_000][i],
                completed_at=[1_814_400, 2_592_000, 5_184_000][i], active_count=[0, 0, 3][i],
                gross=1_900_000_000, kif_current=38_000_000,
                kif_carry_input=[0, 19_000_000, 28_500_000][i],
                kif_zero_compound=[19_000_000, 28_500_000, 0][i],
                kif_carry_next=[19_000_000, 28_500_000, 2][i], new_liability=[0, 0, 66_499_998][i],
                hwm=[22_389_500_000, 24_788_500_000, 27_159_000_000][i],
                pending_sol=0, pending_tokens=0, full_account_oracle=True))
        records.append(dict(kind='repeated_cycles_complete', status='PASS', transactions=30, rounds=3,
            periods=[0, 1, 2], active_counts=[0, 0, 3], final_hwm=27_159_000_000,
            pending_sol=0, pending_tokens=0, kif_carry=2, kif_liability=66_499_998,
            zero_active_compound=47_500_000, cadence_seconds=864_000, cadence_error=6038,
            active_round_error=6034, synthetic_clock=True, real_walltime=False,
            live_ready=False, signature_verified=False, stake_executed=False,
            actual_programs=['PIV1', 'System', 'Token8.0.0', 'SPLpool2.0.3', 'synthetic-caller'],
            loaded_programs=['PIV1', 'synthetic-caller', 'Token8.0.0', 'SPLpool2.0.3', 'Stake5.1.0']))
        return records

    def transcript(self, records, test='repeated_cycles_production_lifecycle'):
        return ('test ' + test + ' ... ' + '\n'.join(
            'PIV1_LIFECYCLE_EVIDENCE ' + json.dumps(row) for row in records)
            + '\ntest result: ok. 1 passed; 0 failed;\n')

    def test_exact_repeated_cycle_profile(self):
        self.assertEqual(RUNNER.selected_test('repeated-cycles'), 'repeated_cycles_production_lifecycle')
        self.assertEqual(RUNNER.parse_args(['bank-run', '--output', '/tmp/unused',
            '--profile', 'repeated-cycles']).profile, 'repeated-cycles')
        self.assertEqual(RUNNER.verify_runtime_output(self.transcript(self.records()),
            'repeated-cycles')['kif_carry'], 2)

    def test_carry_periods_clock_and_guard_evidence_cannot_be_omitted(self):
        for kind in ['cycle_complete', 'cycle_clock', 'cycle_rejection', 'repeated_cycles_complete']:
            records = self.records()
            records.pop(next(i for i, row in enumerate(records) if row['kind'] == kind))
            with self.subTest(kind=kind), self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(self.transcript(records), 'repeated-cycles')

    def test_wrong_accounting_or_invented_activity_rejects(self):
        changes = [('cycle_complete', 'kif_carry_next', 0), ('cycle_complete', 'period', 1),
            ('cycle_complete', 'active_count', 1), ('cycle_complete', 'kif_zero_compound', 0),
            ('cycle_complete', 'prepared_at', 1_814_400), ('cycle_complete', 'hwm', 1),
            ('repeated_cycles_complete', 'kif_liability', 66_500_000),
            ('repeated_cycles_complete', 'active_counts', [False, 0, 3]),
            ('repeated_cycles_complete', 'periods', [0, True, 2]),
            ('repeated_cycles_complete', 'stake_executed', True)]
        for kind, key, value in changes:
            records = self.records();next(row for row in records if row['kind'] == kind)[key] = value
            with self.subTest(kind=kind, key=key), self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(self.transcript(records), 'repeated-cycles')

    def test_clock_and_rejection_provenance_rejects(self):
        changes = [('cycle_clock', 'real_walltime', True), ('cycle_clock', 'epoch', 1),
            ('cycle_clock', 'economic_accounts_unchanged', False), ('cycle_clock', 'timestamp', 99),
            ('cycle_rejection', 'error', 6038), ('cycle_rejection', 'no_cpi', False),
            ('cycle_rejection', 'identical_replay_no_fee', False)]
        for kind, key, value in changes:
            records = self.records();next(row for row in records if row['kind'] == kind)[key] = value
            with self.subTest(kind=kind, key=key), self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(self.transcript(records), 'repeated-cycles')

    def test_cross_profile_duplicate_and_wrong_test_reject(self):
        for extra in [dict(kind='lifecycle_complete'), dict(kind='pause_profile_complete'),
                      dict(kind='external_pool_loss'), self.records()[-1]]:
            with self.subTest(extra=extra), self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(self.transcript(self.records() + [extra]), 'repeated-cycles')
        with self.assertRaises(RuntimeError):
            RUNNER.verify_runtime_output(self.transcript(self.records(),
                test='complete_delayed_production_lifecycle'), 'repeated-cycles')


class ValidInsufficiencyEvidenceTests(unittest.TestCase):
    def records(self):
        # Independent literal timing/accounting contract; no imported expected data.
        rows = []
        for case, prior, timestamp in [('primer-time', 100, 864_000),
                ('cadence-before', 864_000, 1_727_999), ('first-time', 1_727_999, 1_728_000),
                ('before-retry', 1_728_000, 1_814_399), ('at-retry', 1_814_399, 1_814_400),
                ('sufficient-time', 1_814_400, 1_814_401)]:
            rows.append(dict(kind='cycle_clock', case='insufficiency-' + case,
                prior_timestamp=prior, timestamp=timestamp, slot=1, epoch=0,
                synthetic=True, real_walltime=False, only_clock_timestamp_changed=True,
                economic_accounts_unchanged=True))
        for case, status, timestamp, queries, insufficient in [
                ('ten-day-reject', 'Custom(6038)', 1_727_999, 0, None),
                ('malformed-initial', 'InvalidInstructionData', 1_728_000, 0, None),
                ('first', 'Ok', 1_728_000, 1, 1_728_000),
                ('immediate', 'Custom(6039)', 1_728_000, 1, 1_728_000),
                ('before-boundary', 'Custom(6039)', 1_814_399, 1, 1_728_000),
                ('malformed-boundary', 'InvalidInstructionData', 1_814_400, 0, 1_728_000),
                ('exact-retry', 'Ok', 1_814_400, 1, 1_814_400)]:
            rows.append(dict(kind='insufficiency_attempt', case='insufficiency-' + case,
                status=status, timestamp=timestamp, minimum_query_cpis=queries, economic_cpis=0,
                full_account_oracle=True, identical_replay_no_fee=True,
                prepared_at=864_000, insufficient_at=insufficient))
        for case, amount, pending, total in [
                ('primer-yield', 100_000_000_000, 2_000_000_000, 1_100_000_000_000),
                ('small-yield', 50_000_000_000, 0, 1_150_000_000_000),
                ('sufficient-yield', 50_000_000_000, 1, 1_200_000_000_000)]:
            rows.append(dict(kind='insufficiency_yield_input', case='insufficiency-' + case,
                external_reserve_transfer=amount, explicit_pending_sol=pending, pool_total=total,
                pool_supply=1_000_000_000_000, real_pool_update=True,
                live_yield_proof=False, full_account_oracle=True))
        for case, timestamp in [('first', 1_728_000), ('exact-retry', 1_814_400)]:
            payload = (hashlib.sha256(b'event:DistributionPreparationInsufficient').digest()[:8]
                + bytes([17]) * 32 + timestamp.to_bytes(8, 'little')
                + (665_000_000).to_bytes(8, 'little') + (869_565_218).to_bytes(8, 'little'))
            rows.append(dict(kind='valid_insufficiency_event', case='insufficiency-' + case,
                attempted_at=timestamp, target=665_000_000, minimum=869_565_218,
                event_data_hex=payload.hex(), config_only=True, prepared_at=864_000, round_idle=True))
        rows.append(dict(kind='valid_insufficiency_complete', status='PASS', transactions=18,
            completed_rounds=1, final_round_state=1, final_sequence=1, next_sequence=2,
            prepared_at=1_814_401, insufficient_at=1_814_400, prior_prepared_at=864_000,
            retry_seconds=86_400, cadence_seconds=864_000, insufficiency_target=665_000_000,
            insufficiency_minimum=869_565_218, sufficient_target=1_274_583_333,
            sufficient_minimum=833_333_334, round_minimum=1_529_347_049,
            final_hwm=22_389_500_000, proposed_hwm=22_760_000_000,
            pending_sol=1, pending_tokens=0, escrow=1, kif_carry=19_000_000,
            withdrawal_legs_created=0, stake_minimum=1_000_000_000, synthetic_clock=True,
            real_walltime=False, live_ready=False, signature_verified=False,
            actual_programs=['PIV1', 'System', 'Token8.0.0', 'SPLpool2.0.3', 'Stake5.1.0', 'synthetic-caller']))
        return rows

    def transcript(self, records, test='valid_insufficiency_production_lifecycle'):
        return ('test ' + test + ' ... ' + '\n'.join(
            'PIV1_LIFECYCLE_EVIDENCE ' + json.dumps(row) for row in records)
            + '\ntest result: ok. 1 passed; 0 failed;\n')

    def reject(self, records):
        with self.assertRaises(RuntimeError):
            RUNNER.verify_runtime_output(self.transcript(records), 'valid-insufficiency')

    def test_exact_valid_insufficiency_profile(self):
        self.assertEqual(RUNNER.selected_test('valid-insufficiency'), 'valid_insufficiency_production_lifecycle')
        self.assertEqual(RUNNER.parse_args(['bank-run', '--output', '/tmp/unused',
            '--profile', 'valid-insufficiency']).profile, 'valid-insufficiency')
        self.assertEqual(RUNNER.verify_runtime_output(self.transcript(self.records()),
            'valid-insufficiency')['insufficient_at'], 1_814_400)

    def test_each_timing_query_event_and_stimulus_is_required_once(self):
        for kind in ['cycle_clock', 'insufficiency_attempt', 'insufficiency_yield_input',
                'valid_insufficiency_event', 'valid_insufficiency_complete']:
            for duplicate in [False, True]:
                records = self.records(); index = next(i for i, row in enumerate(records) if row['kind'] == kind)
                if duplicate:
                    records.append(records[index].copy())
                else:
                    records.pop(index)
                with self.subTest(kind=kind, duplicate=duplicate):
                    self.reject(records)

    def test_retry_boundary_cadence_cpi_and_malformed_clock_claims_reject(self):
        changes = [('insufficiency_attempt', 'prepared_at', 1_728_000),
            ('insufficiency_attempt', 'minimum_query_cpis', 1),
            ('insufficiency_attempt', 'economic_cpis', 1),
            ('insufficiency_attempt', 'insufficient_at', 1_727_999),
            ('insufficiency_attempt', 'identical_replay_no_fee', False),
            ('cycle_clock', 'synthetic', False), ('cycle_clock', 'economic_accounts_unchanged', False),
            ('cycle_clock', 'timestamp', 864_001), ('cycle_clock', 'slot', True)]
        for kind, field, value in changes:
            records = self.records(); next(row for row in records if row['kind'] == kind)[field] = value
            with self.subTest(kind=kind, field=field):
                self.reject(records)
        for case in ['before-boundary', 'malformed-boundary', 'exact-retry']:
            records = self.records(); next(row for row in records if row.get('case') == 'insufficiency-' + case)['insufficient_at'] = 1_814_399
            self.reject(records)

    def test_actual_insufficiency_event_payload_must_match_outcome(self):
        for offset in [0, 8, 40, 48, 56]:
            records = self.records(); row = next(row for row in records if row['kind'] == 'valid_insufficiency_event')
            payload = bytearray.fromhex(row['event_data_hex']); payload[offset] ^= 1; row['event_data_hex'] = payload.hex()
            with self.subTest(offset=offset):
                self.reject(records)
        for value in ['00', 'x' * 128, None]:
            records = self.records(); next(row for row in records if row['kind'] == 'valid_insufficiency_event')['event_data_hex'] = value
            self.reject(records)

    def test_sufficient_amounts_and_declared_one_lamport_cannot_drift(self):
        for field, value in [('pending_sol', 0), ('sufficient_target', 1_274_583_334),
                ('round_minimum', 1_529_347_050), ('prepared_at', 1_900_800),
                ('insufficient_at', 1_814_401), ('final_hwm', 22_760_000_000),
                ('withdrawal_legs_created', 1), ('stake_minimum', True), ('signature_verified', True)]:
            records = self.records(); records[-1][field] = value
            with self.subTest(field=field):
                self.reject(records)
        records = self.records(); next(row for row in records if row.get('case') == 'insufficiency-sufficient-yield')['explicit_pending_sol'] = 0
        self.reject(records)

    def test_cross_profile_false_completion_and_missing_test_success_reject(self):
        for extra in [dict(kind='repeated_cycles_complete'), dict(kind='cycle_complete'),
                dict(kind='external_pool_loss'), dict(kind='external_cooldown_loss')]:
            self.reject(self.records() + [extra])
        for stdout in [self.transcript(self.records(), 'repeated_cycles_production_lifecycle'),
                self.transcript(self.records()).replace('test result: ok. 1 passed; 0 failed;', '')]:
            with self.assertRaises(RuntimeError):
                RUNNER.verify_runtime_output(stdout, 'valid-insufficiency')


class CooldownRewardEvidenceTests(unittest.TestCase):
    def records(self):
        # Independent literal contract, never imported from the runner's oracle.
        return [
            dict(kind='external_cooldown_reward', synthetic=True, inflation_executed=False,
                credits_observed_unchanged=True, lamports=10_000, delegation_increment=10_000,
                donor_debit=10_000, epoch=0, before_cooldown=True, original_delegated=5_000_000_000,
                updated_delegated=5_000_010_000, stake_rent=2_282_880, all_piv_accounts_unchanged=True,
                capitalization_unchanged=True, full_account_oracle=True),
            dict(kind='cooldown_reward_finalized', epoch=14, reward=10_000, original_total=8_608_636_361,
                recovered_stake_rent=4_565_760, recovered_metadata_rent=5_442_720,
                fixed_snapshot_unchanged=True, full_account_oracle=True),
            dict(kind='cooldown_reward_retained', next_yield=10_000, cumulative_reward=10_000,
                historical_value=121_955_000_003, hwm=121_955_000_003, historical_sol=200_000_002,
                principal_sol=200_010_002, paid_htfp=6_382_727_270, paid_team=2_109_545_453,
                paid_kif=216_363_636, full_account_oracle=True),
            dict(kind='cycle_clock', case='reward-next-cycle-time', timestamp=2_592_000, epoch=14,
                synthetic=True, real_walltime=False, only_clock_timestamp_changed=True,
                economic_accounts_unchanged=True),
            dict(kind='cooldown_reward_later_split', gross=10_000, historical_surplus=0,
                prior_next_yield=10_000, prior_yield_used=8050, htfp=5900, compound=1950,
                team=1950, kif=200, dust=0, active_count=1, period=1, new_legs=0, full_account_oracle=True),
            dict(kind='cooldown_reward_complete', transactions=50, status='PASS', completed_rounds=2,
                epoch=14, legs=2, synthetic_reward=10_000, inflation_executed=False,
                first_hwm=121_955_000_003, final_hwm=121_955_001_953, later_gross=10_000,
                later_htfp=5900, later_compound=1950, later_team=1950, later_kif=200,
                next_yield=0, cumulative_reward=10_000, pending_sol=0, pending_tokens=0,
                kif_liability=216_363_836, kif_carry=0, recovered_stake_rent=4_565_760,
                recovered_metadata_rent=5_442_720, final_state=0, synthetic_clock=True,
                live_ready=False, signature_verified=False,
                actual_programs=['PIV1', 'System', 'Token8.0.0', 'SPLpool2.0.3', 'Stake5.1.0']),
        ]

    def transcript(self, rows, test='cooldown_reward_production_lifecycle'):
        return CooldownLossEvidenceTests().transcript(rows, test)

    def reject(self, rows):
        with self.assertRaises(RuntimeError):
            RUNNER.verify_runtime_output(self.transcript(rows), 'cooldown-reward')

    def test_exact_reward_profile(self):
        self.assertEqual(RUNNER.selected_test('cooldown-reward'), 'cooldown_reward_production_lifecycle')
        self.assertEqual(RUNNER.parse_args(['bank-run', '--output', '/tmp/unused',
            '--profile', 'cooldown-reward']).profile, 'cooldown-reward')
        result = RUNNER.verify_runtime_output(self.transcript(self.records()), 'cooldown-reward')
        self.assertEqual(result['final_hwm'], 121_955_001_953)

    def test_missing_duplicate_or_reordered_lifecycle_rejects(self):
        for index in range(6):
            rows = self.records(); self.reject(rows[:index] + rows[index+1:])
            rows = self.records(); self.reject(rows + [rows[index]])
        rows = self.records(); rows[0], rows[1] = rows[1], rows[0]; self.reject(rows)

    def test_donation_or_unbalanced_reward_rejects(self):
        for field, value in [('delegation_increment', 0), ('donor_debit', 0),
                ('epoch', 14), ('before_cooldown', False), ('inflation_executed', True),
                ('credits_observed_unchanged', False), ('capitalization_unchanged', False),
                ('lamports', True)]:
            rows = self.records(); rows[0][field] = value
            with self.subTest(field=field): self.reject(rows)

    def test_rent_current_distribution_and_principal_misclassification_rejects(self):
        for index, field, value in [(1, 'recovered_stake_rent', 4_575_760),
                (1, 'original_total', 8_608_646_361), (2, 'next_yield', 0),
                (2, 'historical_sol', 200_010_002), (2, 'paid_htfp', 6_382_737_270),
                (2, 'hwm', 121_955_010_003)]:
            rows = self.records(); rows[index][field] = value
            with self.subTest(field=field): self.reject(rows)

    def test_later_split_double_count_and_unearned_kif_rejects(self):
        for field, value in [('gross', 20_000), ('historical_surplus', 10_000),
                ('prior_yield_used', 10_000), ('compound', 10_000), ('period', 0),
                ('active_count', 0), ('kif', 0), ('new_legs', 1)]:
            rows = self.records(); rows[4][field] = value
            with self.subTest(field=field): self.reject(rows)
        for field, value in [('next_yield', 10_000), ('final_hwm', 121_955_010_003),
                ('cumulative_reward', 20_000), ('signature_verified', True), ('transactions', True)]:
            rows = self.records(); rows[-1][field] = value; self.reject(rows)

    def test_cross_profile_and_false_success_rejects(self):
        for extra in [dict(kind='external_pool_loss'), dict(kind='external_cooldown_loss'),
                dict(kind='lifecycle_complete'), dict(kind='valid_insufficiency_complete')]:
            self.reject(self.records() + [extra])
        for text in [self.transcript(self.records(), 'complete_delayed_production_lifecycle'),
                self.transcript(self.records()).replace('test result: ok. 1 passed; 0 failed;', '')]:
            with self.assertRaises(RuntimeError): RUNNER.verify_runtime_output(text, 'cooldown-reward')


class PostCycleCompoundingEvidenceTests(unittest.TestCase):
    def records(self):
        # Reuse only the earlier independent literal S fixture, never runner constants.
        rows = RepeatedCycleEvidenceTests().records()[:-1]
        rows.append(dict(kind='compound_ready', completed_rounds=3, prior_sequence=2,
            next_sequence=3, pool_total=1_300_000_000_000, pool_supply=1_000_000_000_000,
            historical_sol=2_459_000_000, historical_tokens=19_000_000_000,
            hwm=27_159_000_000, kif_liability=66_499_998, kif_carry=2, full_account_oracle=True))
        for case, amount, minimum, code in [('compound-minimum-reject', 2_458_999_998, 1_891_538_461, 6032),
                ('compound-queue-reject', 2_459_000_001, 0, 6028)]:
            rows.append(dict(kind='compound_rejection', case=case, amount=amount, minimum=minimum,
                error=code, no_cpi=True, no_events=True, fee_only=True, identical_replay_no_fee=True))
        # Synthetic parser-only event identity; actual runtime auditor binds the real Config key.
        event = hashlib.sha256(b'event:PendingSolStaked').digest()[:8] + bytes(32)
        event += b''.join(n.to_bytes(8, 'little') for n in [2_458_999_998, 1_891_538_460,
            27_159_000_000, 27_159_000_000])
        for case, late in [('compound-late-failure', True), ('compound-success', False)]:
            rows.append(dict(kind='compound_attempt', case=case, late_outer=late, committed=not late,
                event_data_hex=event.hex(), event_verified=True, protected_cpi_verified=True,
                full_account_oracle=True, prior_summary_unchanged=True, identical_replay_no_fee=late))
        rows.append(dict(kind='post_cycle_compounding_complete', status='PASS', transactions=29,
            completed_rounds=3, prior_sequence=2, next_sequence=3, deposited_sol=2_458_999_998,
            minted_tokens=1_891_538_460, historical_sol=2, historical_tokens=20_891_538_460,
            final_hwm=27_159_000_000, historical_value=27_159_000_000,
            pool_total=1_302_458_999_998, pool_supply=1_001_891_538_460,
            kif_liability=66_499_998, kif_carry=2, pending_sol=0, pending_tokens=0,
            next_yield=0, prior_summary_unchanged=True, rent_unchanged=True, final_state=0,
            synthetic_clock=True, real_walltime=False, live_ready=False, signature_verified=False,
            stake_executed=False, actual_programs=['PIV1', 'System', 'Token8.0.0', 'SPLpool2.0.3', 'synthetic-caller'],
            loaded_programs=['PIV1', 'synthetic-caller', 'Token8.0.0', 'SPLpool2.0.3', 'Stake5.1.0']))
        return rows

    def transcript(self, rows, test='post_cycle_compounding_production_lifecycle'):
        return RepeatedCycleEvidenceTests().transcript(rows, test)

    def reject(self, rows):
        with self.assertRaises(RuntimeError):
            RUNNER.verify_runtime_output(self.transcript(rows), 'post-cycle-compounding')

    def test_exact_post_cycle_profile(self):
        self.assertEqual(RUNNER.selected_test('post-cycle-compounding'), 'post_cycle_compounding_production_lifecycle')
        self.assertEqual(RUNNER.parse_args(['bank-run', '--output', '/tmp/unused',
            '--profile', 'post-cycle-compounding']).profile, 'post-cycle-compounding')
        result = RUNNER.verify_runtime_output(self.transcript(self.records()), 'post-cycle-compounding')
        self.assertEqual((result['historical_sol'], result['minted_tokens']), (2, 1_891_538_460))

    def test_missing_or_duplicate_required_rows_reject(self):
        for kind in ['compound_ready', 'compound_rejection', 'compound_attempt',
                'post_cycle_compounding_complete', 'cycle_complete', 'cycle_clock', 'cycle_rejection']:
            rows = self.records(); index = next(i for i, row in enumerate(rows) if row['kind'] == kind)
            with self.subTest(kind=kind, operation='missing'):
                self.reject(rows[:index] + rows[index + 1:])
            with self.subTest(kind=kind, operation='duplicate'):
                self.reject(rows + [rows[index]])

    def test_accounting_and_retained_state_drift_reject(self):
        for kind, field, wrong in [('compound_ready', 'historical_sol', 2_459_000_001),
                ('compound_ready', 'prior_sequence', 1), ('compound_ready', 'kif_liability', 0),
                ('cycle_complete', 'hwm', 0), ('post_cycle_compounding_complete', 'final_hwm', 27_159_000_001),
                ('post_cycle_compounding_complete', 'historical_sol', 0),
                ('post_cycle_compounding_complete', 'minted_tokens', 2_458_999_998),
                ('post_cycle_compounding_complete', 'prior_summary_unchanged', False),
                ('post_cycle_compounding_complete', 'kif_carry', 0),
                ('post_cycle_compounding_complete', 'pending_sol', 1),
                ('post_cycle_compounding_complete', 'rent_unchanged', False),
                ('post_cycle_compounding_complete', 'next_sequence', True)]:
            rows = self.records(); next(row for row in rows if row['kind'] == kind)[field] = wrong
            with self.subTest(kind=kind, field=field): self.reject(rows)

    def test_guard_and_rollback_provenance_reject(self):
        for kind, field, wrong in [('compound_rejection', 'error', 6034),
                ('compound_rejection', 'minimum', 1_891_538_460), ('compound_rejection', 'no_cpi', False),
                ('compound_rejection', 'no_events', False), ('compound_rejection', 'fee_only', False),
                ('compound_attempt', 'committed', True), ('compound_attempt', 'event_verified', False),
                ('compound_attempt', 'protected_cpi_verified', False),
                ('compound_attempt', 'identical_replay_no_fee', False),
                ('compound_attempt', 'full_account_oracle', 1)]:
            rows = self.records(); next(row for row in rows if row['kind'] == kind)[field] = wrong
            with self.subTest(kind=kind, field=field): self.reject(rows)

    def test_events_and_order_reject(self):
        for bad in ['', '00' * 72, 'GG' * 72, '00' * 71]:
            rows = self.records(); next(row for row in rows if row['kind'] == 'compound_attempt')['event_data_hex'] = bad
            with self.subTest(event=bad): self.reject(rows)
        for left, right in [('compound_ready', 'cycle_complete'), ('compound_rejection', 'compound_attempt'),
                ('compound_attempt', 'post_cycle_compounding_complete')]:
            rows = self.records(); a = next(i for i, row in enumerate(rows) if row['kind'] == left)
            b = next(i for i, row in enumerate(rows) if row['kind'] == right); rows[a], rows[b] = rows[b], rows[a]
            with self.subTest(left=left, right=right): self.reject(rows)

    def test_cross_profile_wrong_test_and_unexecuted_reject(self):
        for extra in [dict(kind='repeated_cycles_complete'), dict(kind='cooldown_reward_complete'),
                dict(kind='external_pool_loss'), dict(kind='external_cooldown_loss')]:
            self.reject(self.records() + [extra])
        for text in [self.transcript(self.records(), 'repeated_cycles_production_lifecycle'),
                self.transcript(self.records()).replace('test result: ok. 1 passed; 0 failed;', '')]:
            with self.assertRaises(RuntimeError): RUNNER.verify_runtime_output(text, 'post-cycle-compounding')


if __name__ == '__main__':
    unittest.main()
