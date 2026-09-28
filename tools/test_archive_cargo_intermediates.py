#!/usr/bin/python3
"""Recovery regressions use disposable synthetic build roots only."""
import fcntl
import importlib.util
import json
import os
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('cargo_archive', Path(__file__).with_name('archive_cargo_intermediates.py'))
archive = importlib.util.module_from_spec(spec)
spec.loader.exec_module(archive)


class Recovery(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='piv1-intermediate-unit-')
        self.addCleanup(self.temp.cleanup)
        self.parent = Path(self.temp.name)
        self.root = self.parent / 'build'
        self.deps = self.root / 'target/debug/deps'
        self.deps.mkdir(parents=True)
        (self.deps.parent / '.cargo-lock').write_bytes(b'')
        self.executable = self.deps / 'runtime'
        self.executable.write_bytes(b'retained executable marker')
        self.executable.chmod(0o700)
        (self.root / 'result.json').write_text(json.dumps({
            'environment': {'CARGO_TARGET_DIR': str(self.root / 'target')},
            'executable': {'path': str(self.executable)}}))
        self.files = [self.deps / name for name in ('a.rlib', 'b.rmeta', 'c.o')]
        for index, path in enumerate(self.files):
            path.write_bytes(b'synthetic public intermediate' * (50 if index < 2 else 70))
            path.chmod(0o640)
            os.utime(path, ns=(1700000000000000123, 1700000000000000456))
        self.output = self.parent / 'archive'

    def plan(self):
        manifest = archive.inventory([self.root], [])
        return manifest, archive.encode(manifest)

    def prepared(self):
        manifest, raw = self.plan()
        archive.archive_all(manifest, raw, self.output, 1)
        return manifest, raw

    def test_archive_deduplicates_and_preserves_all_originals(self):
        manifest, raw = self.prepared()
        self.assertEqual(manifest['unique_objects'], 2)
        self.assertEqual(len(list((self.output / 'objects').iterdir())), 2)
        self.assertTrue(all(path.is_file() for path in self.files))
        self.assertEqual(self.executable.read_bytes(), b'retained executable marker')
        archive.verify_archive(manifest, raw, self.output)

    def test_prune_and_restore_all_exact_bytes_modes_owners_and_nanosecond_mtime(self):
        manifest, raw = self.prepared()
        archive.prune(manifest, raw, self.output)
        self.assertTrue(all(not path.exists() for path in self.files))
        archive.restore(manifest, raw, self.output, 'all')
        for record in manifest['records']:
            path = Path(record['path'])
            self.assertEqual(archive.digest(path), record['sha256'])
            info = path.stat()
            for key in ('size', 'mode', 'uid', 'gid', 'mtime_ns'):
                self.assertEqual(getattr(info, 'st_' + key), record[key])
            self.assertEqual(info.st_nlink, 1)
        self.assertTrue((self.output / 'ready.json').is_file())
        self.assertTrue(self.executable.is_file())

    def test_inventory_excludes_multilinks_and_evidence_paths(self):
        linked = self.deps / 'alias.o'
        os.link(self.files[2], linked)
        pin = self.parent / 'public-pins.json'
        pin.write_text(json.dumps({'path': str(self.files[0])}))
        manifest = archive.inventory([self.root], [pin])
        self.assertEqual([r['path'] for r in manifest['records']], [str(self.files[1])])
        self.assertEqual(len(manifest['excluded']), 3)

    def test_candidate_symlink_and_directory_symlink_reject(self):
        (self.deps / 'unsafe.o').symlink_to('/must-not-follow')
        with self.assertRaises(RuntimeError):
            self.plan()
        # Separate check targets an ancestor rather than relying on file suffix.
        link = self.parent / 'linked-build'
        link.symlink_to(self.root, target_is_directory=True)
        with self.assertRaises(OSError):
            archive.inventory([link], [])

    def test_wrong_owner_or_extended_metadata_rejects(self):
        with patch.object(archive.os, 'getuid', return_value=os.getuid() + 1):
            with self.assertRaises(RuntimeError):
                self.plan()
        with patch.object(archive.os, 'listxattr', return_value=['user.synthetic']):
            with self.assertRaisesRegex(RuntimeError, 'extended'):
                self.plan()

    def test_low_space_stops_before_archive_creation(self):
        manifest, raw = self.plan()
        with patch.object(archive.os, 'fstatvfs', return_value=SimpleNamespace(f_bavail=0, f_frsize=4096, f_bsize=4096)):
            with self.assertRaisesRegex(RuntimeError, 'headroom'):
                archive.archive_all(manifest, raw, self.output, 1024)
        self.assertFalse(self.output.exists())
        self.assertTrue(all(p.exists() for p in self.files))

    def test_mid_archive_space_loss_retains_originals_and_partial_archive(self):
        manifest, raw = self.plan()
        enough = SimpleNamespace(f_bavail=1_000_000, f_frsize=4096, f_bsize=4096)
        low = SimpleNamespace(f_bavail=0, f_frsize=4096, f_bsize=4096)
        with patch.object(archive.os, 'fstatvfs', side_effect=[enough, enough, low]):
            with self.assertRaisesRegex(RuntimeError, 'space changed'):
                archive.archive_all(manifest, raw, self.output, 1024)
        self.assertTrue(self.output.is_dir())
        self.assertFalse((self.output / 'ready.json').exists())
        self.assertTrue(all(p.exists() for p in self.files))

    def test_cargo_lock_contention_prevents_archive(self):
        manifest, raw = self.plan()
        with (self.deps.parent / '.cargo-lock').open('rb') as locked:
            fcntl.flock(locked.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            with self.assertRaises(BlockingIOError):
                archive.archive_all(manifest, raw, self.output, 1)
        self.assertFalse(self.output.exists())

    def test_archive_lock_contention_prevents_prune(self):
        manifest, raw = self.prepared()
        with (self.output / 'archive.lock').open('rb') as locked:
            fcntl.flock(locked.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            with self.assertRaises(BlockingIOError):
                archive.prune(manifest, raw, self.output)
        self.assertTrue(all(p.exists() for p in self.files))

    def test_changed_source_or_new_link_prevents_all_pruning(self):
        manifest, raw = self.prepared()
        self.files[-1].write_bytes(b'changed')
        with self.assertRaisesRegex(RuntimeError, 'original metadata'):
            archive.prune(manifest, raw, self.output)
        self.assertTrue(all(p.exists() for p in self.files))
        os.link(self.files[0], self.parent / 'new-link')
        with self.assertRaisesRegex(RuntimeError, 'linkage'):
            archive.prune(manifest, raw, self.output)

    def test_changed_receipt_and_manifest_are_rejected(self):
        manifest, raw = self.prepared()
        (self.root / 'result.json').write_text('{}')
        with self.assertRaisesRegex(RuntimeError, 'receipt or pin'):
            archive.prune(manifest, raw, self.output)
        with self.assertRaisesRegex(RuntimeError, 'manifest mismatch'):
            archive.verify_archive(manifest, raw + b' ', self.output)
        self.assertTrue(all(p.exists() for p in self.files))

    def test_path_traversal_duplicates_and_protected_records_are_rejected(self):
        manifest, _ = self.plan()
        for mutation in ('outside', 'duplicate', 'protected'):
            value = json.loads(archive.encode(manifest))
            if mutation == 'outside':
                value['records'][0]['path'] = str(self.parent / 'outside.o')
            elif mutation == 'duplicate':
                value['records'].append(value['records'][0])
            else:
                value['protected'].append(value['records'][0]['path'])
            with self.subTest(mutation=mutation), self.assertRaises(RuntimeError):
                archive.validate(value)

    def test_archive_corruption_blocks_prune_before_first_unlink(self):
        manifest, raw = self.prepared()
        obj = next((self.output / 'objects').iterdir())
        obj.write_bytes(b'not gzip')
        with self.assertRaises((OSError, RuntimeError, EOFError)):
            archive.prune(manifest, raw, self.output)
        self.assertTrue(all(p.exists() for p in self.files))

    def test_extra_object_or_missing_ready_receipt_rejects(self):
        manifest, raw = self.prepared()
        (self.output / 'objects/extra.gz').write_bytes(b'unexpected')
        with self.assertRaisesRegex(RuntimeError, 'membership'):
            archive.verify_archive(manifest, raw, self.output)
        other = self.parent / 'incomplete-archive'
        other.mkdir(mode=0o700)
        (other / 'manifest.json').write_bytes(raw)
        with self.assertRaises(FileNotFoundError):
            archive.verify_archive(manifest, raw, other)

    def test_existing_archive_or_symlink_is_never_overwritten(self):
        manifest, raw = self.prepared()
        with self.assertRaises(FileExistsError):
            archive.archive_all(manifest, raw, self.output, 1)
        link = self.parent / 'linked-archive'
        link.symlink_to(self.output, target_is_directory=True)
        with self.assertRaises(OSError):
            with archive.archive_lock(link):
                self.fail('symlink traversal allowed')

    def test_restore_refuses_existing_even_identical_destination(self):
        manifest, raw = self.prepared()
        with self.assertRaisesRegex(RuntimeError, 'existing destination'):
            archive.restore(manifest, raw, self.output, str(self.files[0]))
        self.assertTrue(all(p.exists() for p in self.files))

    def test_interrupted_prune_is_resumable_only_with_durable_matching_journal(self):
        manifest, raw = self.prepared()
        actual = archive.journal
        def interrupt(path, event):
            if event['operation'] == 'pruned':
                raise RuntimeError('synthetic interruption after first unlink')
            actual(path, event)
        with patch.object(archive, 'journal', side_effect=interrupt):
            with self.assertRaisesRegex(RuntimeError, 'synthetic interruption'):
                archive.prune(manifest, raw, self.output)
        self.assertFalse(self.files[0].exists())
        self.assertTrue(self.files[1].exists())
        result = archive.prune(manifest, raw, self.output)
        self.assertEqual(result, {'status': 'PRUNED', 'files': 2, 'already_pruned': 1})
        archive.restore(manifest, raw, self.output, 'all')

    def test_interrupted_prune_recovers_missing_subset_without_overwriting_survivors(self):
        manifest, raw = self.prepared()
        actual = archive.journal
        def interrupt(path, event):
            if event['operation'] == 'pruned':
                raise RuntimeError('synthetic interruption')
            actual(path, event)
        with patch.object(archive, 'journal', side_effect=interrupt):
            with self.assertRaises(RuntimeError):
                archive.prune(manifest, raw, self.output)
        survivor = self.files[1].stat().st_ino
        self.assertEqual(archive.restore(manifest, raw, self.output, 'missing')['files'], 1)
        self.assertEqual(self.files[1].stat().st_ino, survivor)
        self.assertTrue(all(p.exists() for p in self.files))

    def test_unexplained_missing_source_never_counts_as_pruned(self):
        manifest, raw = self.prepared()
        self.files[0].unlink()  # Only this disposable fixture is intentionally removed.
        with self.assertRaisesRegex(RuntimeError, 'unexplained missing'):
            archive.prune(manifest, raw, self.output)
        self.assertTrue(self.files[1].exists())

    def test_interrupted_restore_after_publish_recovers_only_matching_temporary_link(self):
        manifest, raw = self.prepared()
        archive.prune(manifest, raw, self.output)
        unlink = archive.os.unlink
        def interrupt(name, *args, **kwargs):
            if '.piv1-restore-' in str(name):
                raise RuntimeError('synthetic interruption after publish')
            return unlink(name, *args, **kwargs)
        with patch.object(archive.os, 'unlink', side_effect=interrupt):
            with self.assertRaisesRegex(RuntimeError, 'after publish'):
                archive.restore(manifest, raw, self.output, str(self.files[0]))
        self.assertEqual(self.files[0].stat().st_nlink, 2)
        archive.restore(manifest, raw, self.output, str(self.files[0]))
        self.assertEqual(self.files[0].stat().st_nlink, 1)
        self.assertEqual(archive.digest(self.files[0]), manifest['records'][0]['sha256'])
        self.assertFalse(any('.piv1-restore-' in p.name for p in self.deps.iterdir()))

    def test_approved_hashes_are_required_before_any_action(self):
        with patch.object(archive, 'digest', return_value='a' * 64), patch.object(archive, 'read_file') as read:
            with self.assertRaisesRegex(RuntimeError, 'tool differs'):
                archive.main(['prune', '--manifest', str(self.parent / 'manifest.json'),
                              '--approved-tool-sha256', 'b' * 64, '--approved-manifest-sha256', 'c' * 64])
            read.assert_not_called()

    def test_interrupted_restore_after_temporary_unlink_records_exact_completed_destination(self):
        manifest, raw = self.prepared()
        archive.prune(manifest, raw, self.output)
        actual = archive.journal
        def interrupt(path, event):
            if event['operation'] == 'restored':
                raise RuntimeError('synthetic interruption before completion journal')
            actual(path, event)
        with patch.object(archive, 'journal', side_effect=interrupt):
            with self.assertRaisesRegex(RuntimeError, 'completion journal'):
                archive.restore(manifest, raw, self.output, str(self.files[0]))
        inode = self.files[0].stat().st_ino
        self.assertEqual(self.files[0].stat().st_nlink, 1)
        archive.restore(manifest, raw, self.output, str(self.files[0]))
        self.assertEqual(self.files[0].stat().st_ino, inode)
        self.assertEqual(archive.history(self.output, manifest)[str(self.files[0])], 'restored')


if __name__ == '__main__':
    unittest.main()
