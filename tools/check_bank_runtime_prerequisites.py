#!/usr/bin/python3
"""Inspect local Bank preparation prerequisites without executing or changing anything.

Presence and free space are planning observations, not authenticated dependency
resolution, compilation readiness, runtime evidence or permission to proceed.
"""
import argparse
import json
import os
from pathlib import Path
import re
import stat
import sys

CACHE_ROOT = Path('/tmp/piv1-t213-preparation-20260909-a/cargo-home')
REGISTRY = 'index.crates.io-1949cf8c6b5b557f'
CANDIDATE_VERSION = '4.2.0'
PACKAGES = ('solana-runtime', 'solana-accounts-db', 'solana-svm')


class UsageParser(argparse.ArgumentParser):
    def error(self, message):
        self.print_usage(sys.stderr)
        self.exit(64, f'{self.prog}: error: {message}\n')


def positive_decimal(value):
    if not re.fullmatch(r'[1-9][0-9]*', value):
        raise argparse.ArgumentTypeError('minimum free bytes must be a positive decimal integer')
    try:
        return int(value)
    except ValueError:
        raise argparse.ArgumentTypeError('minimum free bytes exceeds the supported integer input length') from None


def observe(path, kind):
    """Reject symlinks observed while checking each path component."""
    path = Path(path)
    result = {'path': str(path), 'kind': kind}
    if not path.is_absolute() or '..' in path.parts or '\0' in str(path):
        return {**result, 'status': 'unsafe', 'reason': 'absolute_normal_path_required'}
    current = Path(path.anchor)
    for index, part in enumerate(path.parts):
        if index:
            current /= part
        try:
            mode = current.lstat().st_mode
        except FileNotFoundError:
            return {**result, 'status': 'missing', 'reason': 'missing_component', 'component': str(current)}
        except OSError as error:
            return {**result, 'status': 'unavailable', 'reason': 'metadata_unavailable',
                    'component': str(current), 'errno': error.errno}
        if stat.S_ISLNK(mode):
            return {**result, 'status': 'unsafe', 'reason': 'symlink_component', 'component': str(current)}
        last = index == len(path.parts) - 1
        expected = stat.S_ISREG(mode) if last and kind == 'archive' else stat.S_ISDIR(mode)
        if not expected:
            return {**result, 'status': 'unsafe', 'reason': 'wrong_component_type', 'component': str(current)}
    return {**result, 'status': 'present'}


def check(cache_root, output_parent, minimum_free_bytes):
    if type(minimum_free_bytes) is not int or minimum_free_bytes <= 0:
        raise ValueError('minimum_free_bytes must be a positive integer selected by the caller')
    cache_root, output_parent = Path(cache_root), Path(output_parent)
    source_root = cache_root / 'registry/src' / REGISTRY
    archive_root = cache_root / 'registry/cache' / REGISTRY
    packages = [
        {'name': name, 'version': CANDIDATE_VERSION,
         'source': observe(source_root / f'{name}-{CANDIDATE_VERSION}', 'directory'),
         'archive': observe(archive_root / f'{name}-{CANDIDATE_VERSION}.crate', 'archive')}
        for name in PACKAGES
    ]
    modular = observe(source_root / f'solana-program-runtime-{CANDIDATE_VERSION}', 'directory')
    parent = observe(output_parent, 'directory')
    available = None
    space = {'status': 'unavailable', 'reason': 'output_parent_not_usable'}
    if parent['status'] == 'present':
        try:
            usage = os.statvfs(output_parent)
            if usage.f_bavail < 0 or usage.f_frsize <= 0:
                space = {'status': 'unavailable', 'reason': 'invalid_space_metadata'}
            else:
                available = usage.f_bavail * usage.f_frsize
                space = {'status': 'sufficient' if available >= minimum_free_bytes else 'insufficient'}
                if available < minimum_free_bytes:
                    space['reason'] = 'below_selected_planning_minimum'
        except OSError as error:
            space = {'status': 'unavailable', 'reason': 'space_metadata_unavailable', 'errno': error.errno}
    ready = (all(item[role]['status'] == 'present' for item in packages for role in ('source', 'archive'))
             and modular['status'] == 'present' and parent['status'] == 'present'
             and space['status'] == 'sufficient')
    return {
        'schema': 1, 'status': 'LOCAL_PREREQUISITES_PRESENT' if ready else 'NOT_READY',
        'ready': ready, 'candidate_agave_version': CANDIDATE_VERSION,
        'candidate_version_authenticated': False, 'cache_root': str(cache_root),
        'packages': packages, 'modular_runtime_source': modular, 'output_parent': parent,
        'minimum_free_bytes': minimum_free_bytes, 'available_bytes': available, 'space': space,
        'scope': 'metadata_only; caller_selected_planning_minimum; no_commands_executed',
        'limitations': [
            'Package presence does not authenticate source bytes, archives or version provenance.',
            'Direct packages are not a complete resolved dependency closure.',
            'The selected space minimum is not a measured Bank build requirement.',
            'A pass is not build/runtime readiness, Bank rollback evidence or authorization.',
            'This metadata observation is not an atomic filesystem snapshot or an execution guard.',
        ],
    }


def main(argv=None):
    parser = UsageParser(description=__doc__)
    parser.add_argument('--output-parent', type=Path, required=True,
                        help='existing real directory for the proposed future build output')
    parser.add_argument('--minimum-free-bytes', type=positive_decimal, required=True,
                        help='explicit planning minimum; no Bank build-size estimate is inferred')
    args = parser.parse_args(argv)
    report = check(CACHE_ROOT, args.output_parent, args.minimum_free_bytes)
    print(json.dumps(report, sort_keys=True, indent=2))
    return 0 if report['ready'] else 2


if __name__ == '__main__':
    raise SystemExit(main())
