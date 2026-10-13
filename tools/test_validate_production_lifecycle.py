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


if __name__ == '__main__':
    unittest.main()
