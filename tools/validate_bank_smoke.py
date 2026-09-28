#!/usr/bin/python3
"""Pinned, offline Bank smoke build/run with retained logs and sampled resource guards.

No deployment or signing. New network sockets are denied; local UNIX socketpairs
remain available for compiler IPC. This is not a
general filesystem sandbox. Resource samples are conservative process-group sums,
not atomic cgroup limits. Existing outputs are never deleted or reused.
"""
import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import re
import resource
import signal
import stat
import subprocess
import time

ROOT = Path('/home/jerem/piv1')
WORK = ROOT / 'validation/genesis-bank-runtime'
SCRIPT = ROOT / 'tools/validate_bank_smoke.py'
PINS = ROOT / 'tools/bank_smoke_pins.json'
TOOLCHAIN = Path('/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin')
CACHE = Path('/tmp/piv1-t236-preparation-20260928-a/cargo-home')
MIN_FREE = 2 * 1024**3
MAX_MEMORY = 2560 * 1024**2


def require(value, message):
    if not value:
        raise RuntimeError(message)


def digest(path):
    require(stat.S_ISREG(path.lstat().st_mode), f'nonregular input: {path}')
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def write(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write('\n')
        stream.flush()
        os.fsync(stream.fileno())


def tree_digest(path):
    records = []
    for item in sorted(path.rglob('*')):
        require(not item.is_symlink(), f'unexpected public-library symlink: {item}')
        if item.is_file():
            records.append([str(item.relative_to(path)), digest(item)])
    require(records, 'empty public-library tree')
    return hashlib.sha256(json.dumps(records, separators=(',', ':')).encode()).hexdigest()


def checked_output(path, fresh=False):
    require(path.parent == Path('/tmp') and
            re.fullmatch(r'piv1-bank-smoke-[a-z0-9-]+', path.name), 'unexpected output path')
    if fresh:
        require(not os.path.lexists(path), 'existing output preserved; choose fresh path')
        path.mkdir(mode=0o700)
    require(path == path.resolve(strict=True) and path.is_dir(), 'output symlink/non-directory')
    require(path.stat().st_uid == os.getuid(), 'output owner mismatch')


def environment(output):
    return {
        'PATH': '/usr/bin:/bin', 'LC_ALL': 'C',
        'CARGO_HOME': str(CACHE), 'CARGO_NET_OFFLINE': 'true',
        'CARGO_TARGET_DIR': str(output / 'target'), 'CARGO_TERM_COLOR': 'never',
        'CARGO_BUILD_JOBS': '1', 'CARGO_INCREMENTAL': '0',
        'CARGO_PROFILE_TEST_DEBUG': '0', 'CARGO_PROFILE_DEV_DEBUG': '0',
        'RUSTC': str(TOOLCHAIN / 'rustc'), 'RUSTDOC': str(TOOLCHAIN / 'rustdoc'),
        'CC': '/usr/bin/cc', 'AR': '/usr/bin/ar', 'RANLIB': '/usr/bin/ranlib',
        'PERL': '/usr/bin/perl', 'OPENSSL_CONFIG_DIR': str(output / 'openssl-config'),
        'GIT_CONFIG_GLOBAL': '/dev/null', 'GIT_CONFIG_NOSYSTEM': '1',
        'CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER': '/usr/bin/cc',
        'TMPDIR': str(output / 'tmp'), 'RAYON_NUM_THREADS': '1',
        'PIV1_BANK_ACCOUNTS_DIR': str(output / 'accounts'),
    }


def child_limits():
    """Linux x86_64 only: deny network creation and x32/foreign ABIs."""
    require(os.uname().machine == 'x86_64', 'unsupported syscall architecture')
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    resource.setrlimit(resource.RLIMIT_FSIZE, (1024**3, 1024**3))
    resource.setrlimit(resource.RLIMIT_AS, (4 * 1024**3, 4 * 1024**3))

    class Filter(ctypes.Structure):
        _fields_ = [('code', ctypes.c_ushort), ('jt', ctypes.c_ubyte),
                    ('jf', ctypes.c_ubyte), ('k', ctypes.c_uint)]

    class Program(ctypes.Structure):
        _fields_ = [('length', ctypes.c_ushort), ('filters', ctypes.POINTER(Filter))]

    instructions = [(0x20, 0, 0, 4), (0x15, 1, 0, 0xc000003e),
                    (0x06, 0, 0, 0x80000000), (0x20, 0, 0, 0),
                    (0x35, 0, 1, 0x40000000), (0x06, 0, 0, 0x00050001)]
    # Rust's subprocess spawn uses an AF_UNIX SOCK_SEQPACKET socketpair.
    # Check both halves of arg0 before permitting only that local domain.
    instructions.extend([(0x15, 0, 7, 53), (0x20, 0, 0, 20),
                         (0x15, 1, 0, 0), (0x06, 0, 0, 0x00050001),
                         (0x20, 0, 0, 16), (0x15, 1, 0, 1),
                         (0x06, 0, 0, 0x00050001), (0x06, 0, 0, 0x7fff0000)])
    # Data operations may use the inherited local pairs; no socket, connect,
    # bind, listen or accept can create an external endpoint. Popen closes
    # inherited descriptors except the explicitly supplied ordinary log files.
    for number in [41, 42, 43, 49, 50, 288, 425, 426, 427]:
        # io_uring is also refused to avoid indirect socket operations. This
        # smoke does not authorize io_uring; an attempted use fails closed.
        instructions.extend([(0x15, 0, 1, number), (0x06, 0, 0, 0x00050001)])
    instructions.append((0x06, 0, 0, 0x7fff0000))
    filters = (Filter * len(instructions))(*(Filter(*row) for row in instructions))
    program = Program(len(instructions), filters)
    libc = ctypes.CDLL(None, use_errno=True)
    require(libc.prctl(38, 1, 0, 0, 0) == 0, 'no_new_privs failed')
    require(libc.prctl(22, 2, ctypes.byref(program), 0, 0) == 0, 'socket filter failed')


def process_memory(group):
    total = 0
    for path in Path('/proc').iterdir():
        if not path.name.isdecimal():
            continue
        try:
            fields = (path / 'stat').read_text().rsplit(')', 1)[1].split()
            if int(fields[2]) != group:
                continue
            for line in (path / 'status').read_text().splitlines():
                if line.startswith(('VmRSS:', 'VmSwap:')):
                    total += int(line.split()[1]) * 1024
        except (FileNotFoundError, ProcessLookupError):
            continue
    return total


def kill_group(group):
    try:
        os.killpg(group, signal.SIGKILL)
    except ProcessLookupError:
        pass


def monitored(command, output, label, timeout):
    env = environment(output)
    started = time.monotonic()
    peak = 0
    minimum = os.statvfs(output).f_bavail * os.statvfs(output).f_frsize
    require(minimum >= MIN_FREE, 'free-space reserve unavailable')
    record = {'command': command, 'cwd': str(WORK), 'environment': env,
              'timeout_seconds': timeout, 'network_socket_creation_denied': True,
              'local_unix_socketpair_allowed': True,
              'memory_guard_bytes': MAX_MEMORY, 'disk_reserve_bytes': MIN_FREE,
              'per_process_address_space_limit': 4 * 1024**3}
    write(output / (label + '.started.json'), record)
    reason = None
    with (output / (label + '.stdout')).open('xb') as out, (output / (label + '.stderr')).open('xb') as err:
        child = subprocess.Popen(command, cwd=WORK, env=env, stdin=subprocess.DEVNULL,
                                 stdout=out, stderr=err, start_new_session=True,
                                 preexec_fn=child_limits)
        try:
            while child.poll() is None:
                memory = process_memory(child.pid)
                free = os.statvfs(output).f_bavail * os.statvfs(output).f_frsize
                peak = max(peak, memory)
                minimum = min(minimum, free)
                if memory > MAX_MEMORY:
                    reason = 'sampled process-group RSS+swap exceeded limit'
                elif free < MIN_FREE:
                    reason = 'sampled disk reserve exhausted'
                elif time.monotonic() - started > timeout:
                    reason = 'wall timeout'
                if reason:
                    kill_group(child.pid)
                    break
                time.sleep(0.25)
        finally:
            if child.poll() is None:
                kill_group(child.pid)
            child.wait()
    record.update(exit=child.returncode, guard_stop=reason,
                  elapsed_seconds=time.monotonic() - started,
                  sampled_peak_group_rss_swap_bytes=peak,
                  sampled_minimum_free_bytes=minimum,
                  logs={name: digest(output / (label + '.' + name)) for name in ['stdout', 'stderr']})
    write(output / (label + '.json'), record)
    require(child.returncode == 0 and reason is None, f'{label} failed; retained complete evidence')
    return record


def verify(pins):
    require(os.getuid() == 1001, 'run as jerem')
    require(pins['schema'] == 1, 'unsupported pin schema')
    for relative, expected in pins['sources'].items():
        require(digest(ROOT / relative) == expected, f'input drift: {relative}')
    for absolute, expected in pins['tools'].items():
        require(digest(Path(absolute)) == expected, f'tool drift: {absolute}')
    for absolute, expected in pins['aliases'].items():
        require(str(Path(absolute).resolve(strict=True)) == expected, f'alias drift: {absolute}')
    for absolute, expected in pins['public_library_trees'].items():
        require(tree_digest(Path(absolute)) == expected, f'public-library drift: {absolute}')
    require(CACHE == CACHE.resolve(strict=True), 'cache symlink refused')
    registry = 'index.crates.io-1949cf8c6b5b557f'
    for package in pins['packages']:
        name = package['name'] + '-' + package['version']
        archive = CACHE / 'registry/cache' / registry / (name + '.crate')
        require(digest(archive) == package['checksum'], f'archive drift: {name}')
        source = CACHE / 'registry/src' / registry / name
        records = []
        for item in sorted(source.rglob('*')):
            require(not item.is_symlink(), f'source symlink: {item}')
            if item.is_file():
                relative = str(item.relative_to(source))
                if relative not in ['.cargo-ok', '.cargo-checksum.json']:
                    records.append({'path': relative, 'bytes': item.stat().st_size,
                                    'sha256': digest(item)})
        records.sort(key=lambda row: row['path'])
        actual = hashlib.sha256(json.dumps(records, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
        require(actual == package['source_tree_sha256'], f'archive-derived source tree drift: {name}')
    for path in [WORK, *WORK.parents, CACHE]:
        names = ['config', 'config.toml', 'credentials', 'credentials.toml'] if path == CACHE else ['.cargo/config', '.cargo/config.toml']
        for name in names:
            require(not os.path.lexists(path / name), f'configuration refused: {path/name}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('stage', choices=['build', 'run'])
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--build-output', type=Path)
    parser.add_argument('--approved-sha256')
    args = parser.parse_args()
    pins = json.loads(PINS.read_text())
    verify(pins)
    if args.stage == 'run':
        require(args.build_output is not None and re.fullmatch('[a-f0-9]{64}', args.approved_sha256 or ''), 'reviewed build/hash required')
        checked_output(args.build_output)
        built = json.loads((args.build_output / 'result.json').read_text())
        require(built['status'] == 'BUILD_PASS' and built['runner_sha256'] == digest(SCRIPT)
                and built['pins_sha256'] == digest(PINS), 'build input/status mismatch')
        binary = Path(built['binary']['path'])
        require(binary == binary.resolve(strict=True) and binary.is_relative_to(args.build_output / 'target'), 'untrusted executable path')
        require(digest(binary) == args.approved_sha256 == built['binary']['sha256'], 'unreviewed executable')
    checked_output(args.output, fresh=True)
    for name in ['tmp', 'openssl-config']:
        (args.output / name).mkdir(mode=0o700)
    result = {'stage': args.stage, 'runner_sha256': digest(SCRIPT), 'pins_sha256': digest(PINS)}
    try:
        if args.stage == 'build':
            monitored([str(TOOLCHAIN / 'cargo'), 'metadata', '--manifest-path', str(WORK/'Cargo.toml'), '--locked', '--offline', '--format-version', '1', '--filter-platform', 'x86_64-unknown-linux-gnu'], args.output, 'metadata', 60)
            metadata = json.loads((args.output / 'metadata.stdout').read_text())
            require(metadata['resolve'] == pins['resolve'], 'resolved features changed')
            monitored([str(TOOLCHAIN / 'cargo'), 'test', '--manifest-path', str(WORK/'Cargo.toml'), '--locked', '--offline', '--jobs', '1', '--test', 'bank', '--no-run', '--message-format=json'], args.output, 'build', 1800)
            messages = [json.loads(line) for line in (args.output / 'build.stdout').read_text().splitlines() if line.startswith('{')]
            diagnostics = [m for m in messages if m.get('reason') == 'compiler-message' and m['message']['level'] in ['warning', 'error', 'failure-note']]
            raw = [line for line in (args.output / 'build.stderr').read_text().splitlines() if re.search(r'(^|\s)(warning|error)(\[|:)', line, re.I)]
            write(args.output/'diagnostics.json', {'messages': diagnostics, 'stderr_diagnostics': raw})
            require(not diagnostics and not raw, 'diagnostics require review')
            executables = [Path(m['executable']) for m in messages if m.get('reason') == 'compiler-artifact' and m['target']['name'] == 'bank' and m.get('executable')]
            require(len(executables) == 1, 'expected one bank test executable')
            binary = executables[0]
            require(binary == binary.resolve(strict=True) and binary.is_relative_to(args.output/'target'), 'unexpected build executable')
            result.update(status='BUILD_PASS', binary={'path': str(binary), 'sha256': digest(binary)})
        else:
            try:
                monitored([str(binary), '--test-threads=1', '--nocapture'], args.output, 'runtime', 60)
            finally:
                require(digest(binary) == args.approved_sha256, 'binary changed during runtime')
            result.update(status='RUN_PASS', binary={'path': str(binary), 'sha256': digest(binary)})
    except BaseException as error:
        result.update(status='FAIL', error=str(error))
        raise
    finally:
        write(args.output/'result.json', result)


if __name__ == '__main__':
    main()
