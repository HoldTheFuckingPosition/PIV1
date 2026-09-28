#!/usr/bin/python3
"""Losslessly archive only reviewed historical PIV1 Cargo intermediates.

Plan is read-only. Archive never prunes. Prune requires a complete verified archive.
No compiler, dependency, network, signing or external command is invoked.
"""
import argparse
from contextlib import contextmanager, ExitStack
import fcntl
import gzip
import hashlib
import json
import os
from pathlib import Path
import stat
import sys
import uuid
import zlib

REPO = Path('/home/jerem/piv1')
TOOL = REPO / 'tools/archive_cargo_intermediates.py'
ARCHIVE = Path('/home/jerem/piv1-evidence/task-2.35-intermediates-20260928-a')
ROOTS = tuple(Path('/tmp') / name for name in (
    'piv1-sbf-claims-build-20260909-a', 'piv1-sbf-claims-build-20260909-b',
    'piv1-sbf-claims-build-20260909-c', 'piv1-sbf-claims-build-t214-20260909-a',
    'piv1-sbf-claims-build-t228-20260921-a', 'piv1-genesis-runtime-build-t230-20260925-a',
    'piv1-genesis-runtime-build-t230-20260925-b',
    'piv1-genesis-initialization-runtime-build-t232-20260926-a',
    'piv1-genesis-initialization-runtime-build-t232-20260926-b',
    'piv1-genesis-initialization-runtime-build-t232-20260926-c',
    'piv1-genesis-initialization-failure-runtime-build-t233-20260927-a',
    'piv1-genesis-initialization-failure-runtime-build-t233-20260927-b'))
PINS = tuple(REPO / 'tools' / name for name in (
    'keyless_sbf_pins.json', 'sbf_claims_pins.json', 'genesis_preflight_probe_pins.json',
    'genesis_preflight_runtime_pins.json', 'genesis_initialization_probe_pins.json',
    'genesis_initialization_runtime_pins.json', 'genesis_initialization_failure_runtime_pins.json'))
EXTENSIONS = {'.rlib', '.rmeta', '.o'}
CHUNK = 1024 * 1024


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


@contextmanager
def directory(path):
    path = Path(path)
    require(path.is_absolute() and '..' not in path.parts, 'absolute normal directory required')
    fd = os.open('/', os.O_RDONLY | os.O_DIRECTORY)
    try:
        for part in path.parts[1:]:
            child = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
            os.close(fd)
            fd = child
        yield fd
    finally:
        os.close(fd)


@contextmanager
def file_open(path, flags=os.O_RDONLY, mode=0o600):
    path = Path(path)
    with directory(path.parent) as parent:
        fd = os.open(path.name, flags | os.O_NOFOLLOW, mode, dir_fd=parent)
        try:
            info = os.fstat(fd)
            require(stat.S_ISREG(info.st_mode) and info.st_uid == os.getuid()
                    and info.st_gid == os.getgid() and info.st_nlink == 1, 'unsafe file ownership/type/linkage')
            yield fd
        finally:
            os.close(fd)


def metadata(info):
    return {key: getattr(info, 'st_' + key) for key in (
        'dev', 'ino', 'mode', 'uid', 'gid', 'size', 'mtime_ns', 'ctime_ns', 'nlink', 'blocks')}


def hash_fd(fd):
    os.lseek(fd, 0, os.SEEK_SET)
    digest = hashlib.sha256()
    while chunk := os.read(fd, CHUNK):
        digest.update(chunk)
    return digest.hexdigest()


def read_file(path):
    with file_open(path) as fd:
        result = bytearray()
        while chunk := os.read(fd, CHUNK):
            result.extend(chunk)
        return bytes(result)


def digest(path):
    with file_open(path) as fd:
        return hash_fd(fd)


def encode(value):
    return (json.dumps(value, sort_keys=True, indent=2) + '\n').encode()


def write_new(path, data):
    with file_open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL) as fd:
        with os.fdopen(os.dup(fd), 'wb') as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
    with directory(Path(path).parent) as parent:
        os.fsync(parent)


def absolute_values(value):
    if isinstance(value, str) and value.startswith('/'):
        yield value
    elif isinstance(value, dict):
        for item in value.values():
            yield from absolute_values(item)
    elif isinstance(value, list):
        for item in value:
            yield from absolute_values(item)


def inventory(roots, pins):
    """Return a complete read-only plan; tests supply private synthetic roots."""
    bindings, protected, excluded, records = {}, set(), [], []
    for path in pins:
        raw = read_file(path)
        bindings[str(path)] = hashlib.sha256(raw).hexdigest()
        protected.update(absolute_values(json.loads(raw)))
    for root in roots:
        with directory(root) as fd:
            require(os.fstat(fd).st_uid == os.getuid(), 'build root not owned by current user')
        receipt = root / 'result.json'
        raw = read_file(receipt)
        data = json.loads(raw)
        require(data['environment']['CARGO_TARGET_DIR'] == str(root / 'target'), 'receipt target mismatch')
        bindings[str(receipt)] = hashlib.sha256(raw).hexdigest()
        if data.get('executable'):
            protected.add(data['executable']['path'])
    for root in roots:
        target = root / 'target'
        with directory(target):
            pass
        for parent, dirs, names in os.walk(target, followlinks=False):
            for name in dirs:
                with directory(Path(parent) / name):
                    pass
            for name in sorted(names):
                path = Path(parent) / name
                if path.suffix not in EXTENSIONS:
                    continue
                with directory(path.parent) as fd:
                    info = os.stat(path.name, dir_fd=fd, follow_symlinks=False)
                require(stat.S_ISREG(info.st_mode), 'nonregular candidate')
                require(info.st_uid == os.getuid() and info.st_gid == os.getgid(), 'candidate owner mismatch')
                if info.st_mode & 0o111 or info.st_nlink != 1 or str(path) in protected:
                    excluded.append({'path': str(path), 'reason': 'executable_multilink_or_protected'})
                    continue
                with file_open(path) as fd:
                    require(not os.listxattr(fd), 'extended source metadata requires separate preservation')
                    before = metadata(os.fstat(fd))
                    sha = hash_fd(fd)
                    require(metadata(os.fstat(fd)) == before, 'candidate changed while hashing')
                records.append({'path': str(path), 'sha256': sha, **before})
    records.sort(key=lambda record: record['path'])
    unique = {record['sha256']: record['size'] for record in records}
    return {'schema': 1, 'roots': [str(root) for root in roots], 'bindings': bindings,
            'protected': sorted(protected), 'excluded': sorted(excluded, key=lambda item: item['path']),
            'records': records, 'logical_bytes': sum(r['size'] for r in records),
            'allocated_bytes': sum(r['blocks'] * 512 for r in records),
            'unique_objects': len(unique), 'unique_bytes': sum(unique.values())}


def validate(manifest, roots=None):
    require(manifest['schema'] == 1, 'unsupported manifest')
    actual_roots = [Path(root) for root in manifest['roots']]
    require(actual_roots and len(actual_roots) == len(set(actual_roots)), 'duplicate or empty roots')
    if roots is not None:
        require(actual_roots == list(roots), 'unapproved roots')
    seen, sizes = set(), {}
    for record in manifest['records']:
        path = Path(record['path'])
        require(path.is_absolute() and '..' not in path.parts and path.suffix in EXTENSIONS,
                'unsafe candidate path')
        require(sum(path.is_relative_to(root / 'target') for root in actual_roots) == 1,
                'candidate outside target')
        require(record['path'] not in seen and record['path'] not in manifest['protected'], 'duplicate/protected candidate')
        seen.add(record['path'])
        require(stat.S_ISREG(record['mode']) and not record['mode'] & 0o111 and record['nlink'] == 1
                and record['uid'] == os.getuid() and record['gid'] == os.getgid(), 'unsafe candidate metadata')
        sha = record['sha256']
        require(len(sha) == 64 and all(c in '0123456789abcdef' for c in sha) and record['size'] >= 0,
                'invalid content identity')
        require(sha not in sizes or sizes[sha] == record['size'], 'conflicting content size')
        sizes[sha] = record['size']
    require(bool(seen), 'empty archive plan')
    return sizes


def check_bindings(manifest):
    for path, sha in manifest['bindings'].items():
        require(digest(Path(path)) == sha, 'receipt or pin changed')


@contextmanager
def cargo_locks(manifest):
    with ExitStack() as stack:
        for root in manifest['roots']:
            fd = stack.enter_context(file_open(Path(root) / 'target/debug/.cargo-lock', os.O_RDWR))
            fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        yield


@contextmanager
def archive_lock(archive):
    with directory(archive) as fd:
        info = os.fstat(fd)
        require(info.st_uid == os.getuid() and stat.S_IMODE(info.st_mode) == 0o700, 'archive must be private and owned')
    with file_open(archive / 'archive.lock', os.O_RDWR) as fd:
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        yield


def checked_original(record):
    with file_open(Path(record['path'])) as fd:
        require(not os.listxattr(fd), 'extended source metadata changed')
        require(metadata(os.fstat(fd)) == {key: record[key] for key in metadata(os.fstat(fd))}, 'original metadata changed')
        require(hash_fd(fd) == record['sha256'], 'original content changed')
        require(metadata(os.fstat(fd)) == {key: record[key] for key in metadata(os.fstat(fd))}, 'original changed while reading')


def object_path(archive, sha):
    return archive / 'objects' / (sha + '.gz')


def verify_object(path, expected_sha, expected_size):
    with file_open(path) as fd, os.fdopen(os.dup(fd), 'rb') as raw:
        before = metadata(os.fstat(fd))
        with gzip.GzipFile(fileobj=raw, mode='rb') as stream:
            sha, count = hashlib.sha256(), 0
            while chunk := stream.read(CHUNK):
                count += len(chunk)
                require(count <= expected_size, 'archive expands beyond expected size')
                sha.update(chunk)
        require(count == expected_size and sha.hexdigest() == expected_sha, 'archive content mismatch')
        require(metadata(os.fstat(fd)) == before, 'archive changed during verification')
    return digest(path)


def archive_all(manifest, manifest_bytes, archive, reserve):
    sizes = validate(manifest)
    require(type(reserve) is int and reserve > 0, 'positive explicit archive reserve required')
    require(manifest_bytes == encode(manifest), 'noncanonical manifest bytes')
    require(zlib.ZLIB_RUNTIME_VERSION == '1.3' and zlib.MAX_WBITS == 15 and zlib.DEF_MEM_LEVEL == 8,
            'unreviewed compressor parameters or runtime')
    check_bindings(manifest)
    with directory(archive.parent) as fd:
        usage = os.fstatvfs(fd)
        require(usage.f_frsize > 0 and usage.f_bavail >= 0, 'invalid filesystem space metadata')
        # zlib v1.3 deflateBound, default window15/mem8, raw stream plus the
        # deterministic gzip header/trailer: n+(n>>12)+(n>>14)+(n>>25)+25.
        # One percent +8KiB dominates that bound. Round each object allocation
        # and retain manifest/ready/journal metadata slack; assume no compression.
        block = max(usage.f_frsize, usage.f_bsize)
        worst_case = sum(((size + size // 100 + 8192 + block - 1) // block) * block
                         for size in sizes.values()) + 3 * len(manifest_bytes) + 2 * CHUNK
        require(usage.f_bavail * usage.f_frsize >= worst_case + reserve, 'insufficient unique worst-case headroom')
        info = os.fstat(fd)
        require(info.st_uid == os.getuid() and info.st_gid == os.getgid()
                and stat.S_IMODE(info.st_mode) == 0o700, 'archive parent must be private and owned')
        with cargo_locks(manifest):
            for record in manifest['records']:
                checked_original(record)
            os.mkdir(archive.name, mode=0o700, dir_fd=fd)
            os.fsync(fd)
            write_new(archive / 'archive.lock', b'')
            with archive_lock(archive):
                write_new(archive / 'manifest.json', manifest_bytes)
                with directory(archive) as archive_fd:
                    os.mkdir('objects', mode=0o700, dir_fd=archive_fd)
                    os.fsync(archive_fd)
                objects = {}
                for record in manifest['records']:
                    sha = record['sha256']
                    if sha in objects:
                        continue
                    with directory(archive) as archive_fd:
                        current = os.fstatvfs(archive_fd)
                        next_bound = record['size'] + record['size'] // 100 + 8192 + max(current.f_frsize, current.f_bsize)
                        require(current.f_bavail * current.f_frsize >= next_bound + reserve, 'space changed: preserve partial archive and all originals')
                    checked_original(record)
                    path = object_path(archive, sha)
                    with file_open(Path(record['path'])) as source_fd, file_open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL) as out_fd:
                        with os.fdopen(os.dup(out_fd), 'wb') as raw:
                            with gzip.GzipFile(filename='', mode='wb', fileobj=raw, compresslevel=6, mtime=0) as stream:
                                while chunk := os.read(source_fd, CHUNK):
                                    stream.write(chunk)
                            raw.flush()
                            os.fsync(raw.fileno())
                    objects[sha] = {'size': record['size'], 'archive_sha256': verify_object(path, sha, record['size'])}
                    with directory(path.parent) as parent:
                        os.fsync(parent)
                    if len(objects) % 250 == 0:
                        print(json.dumps({'archived_objects': len(objects), 'total_objects': len(sizes)}), file=sys.stderr, flush=True)
                check_bindings(manifest)
                write_new(archive / 'ready.json', encode({'manifest_sha256': hashlib.sha256(manifest_bytes).hexdigest(), 'objects': objects}))
    return {'status': 'ARCHIVE_READY', 'objects': len(sizes), 'originals_removed': 0}


def verify_archive(manifest, manifest_bytes, archive):
    sizes = validate(manifest)
    require(read_file(archive / 'manifest.json') == manifest_bytes == encode(manifest), 'archive manifest mismatch')
    ready = json.loads(read_file(archive / 'ready.json'))
    require(ready['manifest_sha256'] == hashlib.sha256(manifest_bytes).hexdigest()
            and set(ready['objects']) == set(sizes), 'incomplete archive')
    with directory(archive / 'objects') as fd:
        require(set(os.listdir(fd)) == {sha + '.gz' for sha in sizes}, 'unexpected archive object membership')
    for sha, size in sizes.items():
        require(ready['objects'][sha]['size'] == size, 'archive size metadata mismatch')
        require(verify_object(object_path(archive, sha), sha, size) == ready['objects'][sha]['archive_sha256'], 'archive compressed bytes changed')
    return ready


def journal(archive, entry):
    with file_open(archive / 'journal.jsonl', os.O_WRONLY | os.O_APPEND | os.O_CREAT) as fd:
        data = json.dumps(entry, sort_keys=True).encode() + b'\n'
        require(os.write(fd, data) == len(data), 'incomplete journal write')
        os.fsync(fd)
    with directory(archive) as parent:
        os.fsync(parent)


def history(archive, manifest, detailed=False):
    try:
        raw = read_file(archive / 'journal.jsonl')
    except FileNotFoundError:
        return {}
    require(not raw or raw.endswith(b'\n'), 'incomplete journal: preserve archive and inspect before resumption')
    expected = {record['path']: record['sha256'] for record in manifest['records']}
    states = {}
    for line in raw.splitlines():
        event = json.loads(line)
        require(event['operation'] in ('prune-ready', 'pruned', 'restore-temp', 'restore-ready', 'restored')
                and expected.get(event['path']) == event['sha256'], 'journal differs from manifest')
        if 'temporary' in event:
            require(Path(event['temporary']).parent == Path(event['path']).parent
                    and Path(event['temporary']).name.startswith(Path(event['path']).name + '.piv1-restore-'), 'unsafe journal temporary')
        states[event['path']] = event if detailed else event['operation']
    return states


def missing(record):
    path = Path(record['path'])
    with directory(path.parent) as parent:
        try:
            os.stat(path.name, dir_fd=parent, follow_symlinks=False)
        except FileNotFoundError:
            return True
    return False


def prune(manifest, manifest_bytes, archive):
    with cargo_locks(manifest), archive_lock(archive):
        check_bindings(manifest)
        verify_archive(manifest, manifest_bytes, archive)
        states = history(archive, manifest)
        remaining = []
        # Only a durable matching pre-unlink journal explains a missing source.
        for record in manifest['records']:
            if missing(record):
                require(states.get(record['path']) in ('prune-ready', 'pruned'), 'unexplained missing original')
            else:
                checked_original(record)
                remaining.append(record)
        for index, record in enumerate(remaining):
            checked_original(record)
            journal(archive, {'operation': 'prune-ready', 'path': record['path'], 'sha256': record['sha256']})
            path = Path(record['path'])
            with directory(path.parent) as parent:
                info = os.stat(path.name, dir_fd=parent, follow_symlinks=False)
                require(metadata(info) == {key: record[key] for key in metadata(info)}, 'original changed before unlink')
                os.unlink(path.name, dir_fd=parent)
                os.fsync(parent)
            journal(archive, {'operation': 'pruned', 'path': record['path'], 'sha256': record['sha256']})
            if (index + 1) % 500 == 0:
                print(json.dumps({'pruned_this_run': index + 1, 'remaining_this_run': len(remaining) - index - 1}), file=sys.stderr, flush=True)
        check_bindings(manifest)
    return {'status': 'PRUNED', 'files': len(remaining), 'already_pruned': len(manifest['records']) - len(remaining)}


def restore(manifest, manifest_bytes, archive, selected):
    require(selected in ('all', 'missing') or selected in {r['path'] for r in manifest['records']}, 'explicit manifest restore selection required')
    with cargo_locks(manifest), archive_lock(archive):
        check_bindings(manifest)
        verify_archive(manifest, manifest_bytes, archive)
        states = history(archive, manifest, detailed=True)
        records = []
        for record in manifest['records']:
            if selected == 'missing':
                if missing(record):
                    require(states.get(record['path'], {}).get('operation') in ('prune-ready', 'pruned', 'restore-temp', 'restore-ready'), 'unexplained missing restore source')
                    records.append(record)
            elif selected == 'all' or record['path'] == selected:
                records.append(record)
        require(records, 'no explicitly selected missing records')
        recovered = []
        for record in records:
            path = Path(record['path'])
            with directory(path.parent) as parent:
                try:
                    os.stat(path.name, dir_fd=parent, follow_symlinks=False)
                except FileNotFoundError:
                    pass
                else:
                    event = states.get(record['path'], {})
                    require(event.get('operation') == 'restore-ready' and event.get('temporary'), 'restore refuses existing destination')
                    temporary = Path(event['temporary']).name
                    original = os.stat(path.name, dir_fd=parent, follow_symlinks=False)
                    try:
                        temp_info = os.stat(temporary, dir_fd=parent, follow_symlinks=False)
                    except FileNotFoundError:
                        temp_info = None
                    require(stat.S_ISREG(original.st_mode) and
                            (original.st_nlink == 1 if temp_info is None else
                             original.st_ino == temp_info.st_ino and original.st_dev == temp_info.st_dev
                             and original.st_nlink == temp_info.st_nlink == 2), 'unexplained existing restore destination')
                    fd = os.open(path.name, os.O_RDONLY | os.O_NOFOLLOW, dir_fd=parent)
                    try:
                        require(all(getattr(original, 'st_' + key) == record[key] for key in ('mode', 'uid', 'gid', 'size', 'mtime_ns'))
                                and not os.listxattr(fd) and hash_fd(fd) == record['sha256'], 'published restore mismatch')
                    finally:
                        os.close(fd)
                    if temp_info is not None:
                        os.unlink(temporary, dir_fd=parent)
                        os.fsync(parent)
                    journal(archive, {'operation': 'restored', 'path': record['path'], 'sha256': record['sha256']})
                    recovered.append(record['path'])
        for record in records:
            if record['path'] in recovered:
                continue
            path = Path(record['path'])
            with directory(path.parent) as parent:
                temporary = path.name + '.piv1-restore-' + uuid.uuid4().hex
                event = {'path': record['path'], 'sha256': record['sha256'], 'temporary': str(path.parent / temporary)}
                journal(archive, {**event, 'operation': 'restore-temp'})
                fd = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600, dir_fd=parent)
                try:
                    info = os.fstat(fd)
                    require(info.st_uid == record['uid'] and info.st_gid == record['gid'] and not os.listxattr(fd), 'restore inherited unexpected owner or extended metadata')
                    with os.fdopen(os.dup(fd), 'wb') as output, file_open(object_path(archive, record['sha256'])) as archive_fd:
                        with os.fdopen(os.dup(archive_fd), 'rb') as raw, gzip.GzipFile(fileobj=raw, mode='rb') as stream:
                            count, sha = 0, hashlib.sha256()
                            while chunk := stream.read(CHUNK):
                                count += len(chunk)
                                require(count <= record['size'], 'restore expands beyond expected size')
                                sha.update(chunk)
                                output.write(chunk)
                        require(count == record['size'] and sha.hexdigest() == record['sha256'], 'restore content mismatch')
                        output.flush()
                    os.fchmod(fd, stat.S_IMODE(record['mode']))
                    os.utime(fd, ns=(record['mtime_ns'], record['mtime_ns']))
                    os.fsync(fd)
                    journal(archive, {**event, 'operation': 'restore-ready'})
                    os.link(temporary, path.name, src_dir_fd=parent, dst_dir_fd=parent, follow_symlinks=False)
                    os.fsync(parent)
                    os.unlink(temporary, dir_fd=parent)
                    os.fsync(parent)
                finally:
                    os.close(fd)
            journal(archive, {'operation': 'restored', 'path': record['path'], 'sha256': record['sha256']})
            with file_open(path) as fd:
                info = os.fstat(fd)
                require(all(getattr(info, 'st_' + key) == record[key] for key in ('mode', 'uid', 'gid', 'size', 'mtime_ns'))
                        and not os.listxattr(fd) and hash_fd(fd) == record['sha256'], 'restored final path mismatch')
    return {'status': 'RESTORED', 'files': len(records), 'archive_retained': True}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('stage', choices=('plan', 'archive', 'verify', 'prune', 'restore'))
    parser.add_argument('--manifest', type=Path)
    parser.add_argument('--approved-manifest-sha256')
    parser.add_argument('--approved-tool-sha256')
    parser.add_argument('--reserve-bytes', type=int)
    parser.add_argument('--select')
    args = parser.parse_args(argv)
    require(Path(__file__) == TOOL, 'unexpected tool path')
    if args.stage == 'plan':
        print(encode(inventory(ROOTS, PINS)).decode(), end='')
        return 0
    require(args.manifest is not None and args.approved_manifest_sha256 and args.approved_tool_sha256,
            'reviewed manifest and tool hashes required')
    require(digest(TOOL) == args.approved_tool_sha256, 'tool differs from review')
    raw = read_file(args.manifest)
    require(hashlib.sha256(raw).hexdigest() == args.approved_manifest_sha256, 'manifest differs from review')
    manifest = json.loads(raw)
    validate(manifest, ROOTS)
    require(set(manifest['bindings']) == {str(p) for p in PINS} | {str(r / 'result.json') for r in ROOTS}, 'unapproved binding paths')
    if args.stage == 'archive':
        result = archive_all(manifest, raw, ARCHIVE, args.reserve_bytes)
    elif args.stage == 'prune':
        result = prune(manifest, raw, ARCHIVE)
    elif args.stage == 'restore':
        result = restore(manifest, raw, ARCHIVE, args.select)
    else:
        with cargo_locks(manifest), archive_lock(ARCHIVE):
            check_bindings(manifest)
            verify_archive(manifest, raw, ARCHIVE)
        result = {'status': 'ARCHIVE_VERIFIED', 'originals_removed': 0}
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (OSError, EOFError, RuntimeError, ValueError, KeyError) as error:
        print(json.dumps({'status': 'STOPPED', 'reason': str(error)}), file=sys.stderr)
        raise SystemExit(2)
