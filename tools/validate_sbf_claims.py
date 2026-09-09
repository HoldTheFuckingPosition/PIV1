#!/usr/bin/python3
"""Reviewed isolated host compilation and local SBF validation, never deployment.

Default stage is read-only preflight. Build only compiles; run requires the exact
reviewed executable hash from a successful build. All evidence uses fresh paths.
This is a bounded local runner, not an OS sandbox or a supply-chain audit.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import tarfile
import time

ROOT = Path('/home/jerem/piv1')
WORKSPACE = ROOT / 'validation/sbf-claims'
PINS = ROOT / 'tools/sbf_claims_pins.json'
TOOLCHAIN = Path('/home/jerem/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu')
PREPARATION = Path('/tmp/piv1-t213-preparation-20260909-a')
CARGO_HOME = PREPARATION / 'cargo-home'
ELF = Path('/tmp/piv1-keyless-sbf-build-20260909-c/target/sbpf-solana-solana/release/piv1.so')
ELF_SHA = '0392bb822a3e767674ccd75486ad2685320bce5ffadb426ea8a93b08625bb6c8'
REGISTRY = 'index.crates.io-1949cf8c6b5b557f'
BASELINE = 'fd48c3b1644faed6d30fdb92774c1bd8c03a2658'


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def regular(path):
    require(stat.S_ISREG(path.lstat().st_mode), f'not a regular non-symlink file: {path}')
    return path


def digest(path):
    with regular(path).open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def write_json(path, value):
    with path.open('x', encoding='utf-8') as output:
        json.dump(value, output, indent=2, sort_keys=True)
        output.write('\n')


def fresh_output(path):
    require(path.parent == Path('/tmp'), 'output must be an immediate /tmp child')
    require(re.fullmatch(r'piv1-sbf-claims-[a-z0-9-]+', path.name), 'unexpected output name')
    require(not path.exists() and not path.is_symlink(), 'output already exists')
    path.mkdir(mode=0o700)


def environment(output):
    # HOME is neither inherited nor reassigned. No user credentials/configuration
    # are needed. Explicit CARGO_HOME is the already-reviewed private public cache.
    return {
        'PATH': '/usr/bin:/bin', 'LC_ALL': 'C',
        'CARGO_HOME': str(CARGO_HOME), 'TMPDIR': str(output / 'tmp'),
        'CARGO_TARGET_DIR': str(output / 'target'),
        'RUSTC': str(TOOLCHAIN / 'bin/rustc'),
        'RUSTDOC': str(TOOLCHAIN / 'bin/rustdoc'),
        'CARGO_TERM_COLOR': 'never', 'CARGO_NET_OFFLINE': 'true',
        'CC': '/usr/bin/cc', 'AR': '/usr/bin/ar', 'RANLIB': '/usr/bin/ranlib',
        'CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER': '/usr/bin/cc',
    }


def reject_configs():
    directories = [WORKSPACE, *WORKSPACE.parents, CARGO_HOME]
    for directory in directories:
        names = ('config', 'config.toml', 'credentials', 'credentials.toml') if directory == CARGO_HOME else ('.cargo/config', '.cargo/config.toml')
        for name in names:
            path = directory / name
            require(not path.exists() and not path.is_symlink(), f'unexpected configuration: {path}')
    require(CARGO_HOME.is_dir() and not CARGO_HOME.is_symlink(), 'private Cargo home missing/nonregular')
    # libc 0.2.189 probes this optional tool even on Linux. Keep its reviewed
    # absence explicit; do not run an unreviewed executable found later in PATH.
    for path in [Path('/usr/bin/emcc'), Path('/bin/emcc')]:
        require(not os.path.lexists(path), f'unreviewed optional compiler appeared: {path}')


def source_files():
    paths = []
    for directory in [ROOT / 'programs', ROOT / 'crates', WORKSPACE]:
        for path in directory.rglob('*'):
            require(not path.is_symlink(), f'source symlink: {path}')
            if path.is_file():
                paths.append(path)
    paths.extend(ROOT / name for name in ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml',
        'Anchor.toml', 'tools/build_keyless_sbf.py', 'tools/keyless_sbf_pins.json',
        'tools/test_build_keyless_sbf.py', 'tools/test_validate_sbf_claims.py'])
    return {str(path.relative_to(ROOT)): digest(path) for path in sorted(set(paths))}


def verify_packages(packages):
    """Bind extracted source to the hash-verified public archive, not a mutable
    .cargo-checksum.json assertion. No archive extraction or script execution."""
    checked = []
    for package in packages:
        name = f"{package['name']}-{package['version']}"
        archive = CARGO_HOME / 'registry/cache' / REGISTRY / (name + '.crate')
        source = CARGO_HOME / 'registry/src' / REGISTRY / name
        require(digest(archive) == package['checksum'], f'archive changed: {name}')
        require(source.is_dir() and not source.is_symlink(), f'bad package source: {name}')
        expected = set()
        with tarfile.open(archive, mode='r:gz') as tar:
            for member in tar:
                relative = Path(member.name)
                require(relative.parts[0] == name and '..' not in relative.parts and not relative.is_absolute(), 'unsafe archive path')
                if member.isdir():
                    continue
                require(member.isfile(), f'nonregular archive member: {member.name}')
                relative = Path(*relative.parts[1:])
                require(str(relative) not in expected, 'duplicate archive member')
                expected.add(str(relative))
                with tar.extractfile(member) as contents:
                    expected_digest = hashlib.file_digest(contents, 'sha256').hexdigest()
                require(digest(source / relative) == expected_digest, f'extracted source changed: {name}/{relative}')
        actual = set()
        for path in source.rglob('*'):
            require(not path.is_symlink(), f'package symlink: {path}')
            if path.is_file():
                actual.add(str(path.relative_to(source)))
        require(actual - {'.cargo-ok', '.cargo-checksum.json'} == expected, f'package file set changed: {name}')
        checked.append({'name': name, 'checksum': package['checksum'], 'files': len(expected)})
    blst = CARGO_HOME / 'registry/src' / REGISTRY / 'blst-0.3.17'
    require(not (blst / 'libblst.a').exists(), 'unreviewed native archive override')
    require((blst / 'blst/src/server.c').is_file(), 'packaged blst source missing; fallback forbidden')
    return checked


class Run:
    def __init__(self, output):
        self.output = output
        self.env = environment(output)
        self.commands = []

    def command(self, label, argv, cwd=WORKSPACE):
        stdout = self.output / f'{label}.stdout'
        stderr = self.output / f'{label}.stderr'
        record = {'label': label, 'argv': list(map(str, argv)), 'cwd': str(cwd)}
        self.commands.append(record)
        record['started_utc'] = datetime.now(timezone.utc).isoformat()
        started = time.monotonic()
        with stdout.open('xb') as out, stderr.open('xb') as err:
            result = subprocess.run(argv, cwd=cwd, env=self.env, stdout=out, stderr=err)
        record.update(exit_code=result.returncode, stdout_sha256=digest(stdout), stderr_sha256=digest(stderr))
        record['finished_utc'] = datetime.now(timezone.utc).isoformat()
        record['duration_seconds'] = time.monotonic() - started
        return result.returncode

    def git_snapshot(self, label):
        value = {}
        for name, args in [('head', ['rev-parse', 'HEAD']), ('branch', ['branch', '--show-current']),
            ('refs', ['show-ref']), ('status', ['status', '--short'])]:
            require(self.command(f'{label}-{name}', ['/usr/bin/git', *args], ROOT) == 0, 'Git read failed')
            value[name] = (self.output / f'{label}-{name}.stdout').read_text()
        return value


def verify_artifact(artifact):
    require(isinstance(artifact, dict) and set(artifact) == {'path', 'sha256', 'bytes'}, 'reviewed artifact identity required')
    require(isinstance(artifact['sha256'], str) and re.fullmatch(r'[a-f0-9]{64}', artifact['sha256']), 'reviewed artifact hash required')
    require(type(artifact['bytes']) is int and artifact['bytes'] > 0, 'reviewed artifact size required')
    path = Path(artifact['path'])
    require(path.is_absolute() and path == path.resolve(strict=True), 'artifact path must resolve exactly')
    require(digest(path) == artifact['sha256'] and path.stat().st_size == artifact['bytes'], 'reviewed artifact changed')
    return path


def preflight(run, pins):
    reject_configs()
    require(pins['schema'] == 1 and pins['baseline_head'] == BASELINE, 'invalid pin schema/baseline')
    require(source_files() == pins['sources'], 'source set/hash changed')
    for path, expected in pins['tools'].items():
        require(digest(Path(path)) == expected, f'tool/library changed: {path}')
    for path, expected in pins['aliases'].items():
        require(str(Path(path).resolve(strict=True)) == expected, f'tool alias changed: {path}')
    require(digest(ELF) == ELF_SHA and ELF.stat().st_size == 176064, 'Task 2.12 artifact changed')
    artifact = pins['artifact']
    path = verify_artifact(artifact)
    run.env.update(PIV_VALIDATION_ELF_PATH=str(path), PIV_VALIDATION_ELF_SHA256=artifact['sha256'],
        PIV_VALIDATION_ELF_BYTES=str(artifact['bytes']))
    require(run.command('baseline-ancestry', ['/usr/bin/git', 'merge-base', '--is-ancestor', BASELINE, 'HEAD'], ROOT) == 0, 'baseline not ancestor')
    packages = verify_packages(pins['packages'])
    write_json(run.output / 'packages.json', packages)
    command = [str(TOOLCHAIN / 'bin/cargo'), 'metadata', '--manifest-path', str(WORKSPACE / 'Cargo.toml'),
        '--locked', '--offline', '--format-version', '1', '--filter-platform', 'x86_64-unknown-linux-gnu']
    require(run.command('metadata', command) == 0, 'metadata failed')
    metadata = json.loads((run.output / 'metadata.stdout').read_text())
    require(metadata['resolve'] == pins['resolve'], 'resolved feature/dependency graph changed')
    require(metadata['workspace_members'] == [metadata['resolve']['root']], 'workspace isolation changed')
    require(run.command('host-cpu', ['/usr/bin/lscpu']) == 0, 'CPU inventory failed')
    # CPU information is provenance, not a portability guarantee. blst may select ADX.
    write_json(run.output / 'preflight.json', {'status': 'PREFLIGHT_PASS',
        'package_count': len(packages), 'artifact': pins['artifact'], 'sources': pins['sources']})


def build(run):
    argv = [str(TOOLCHAIN / 'bin/cargo'), 'test', '--manifest-path', str(WORKSPACE / 'Cargo.toml'),
        '--locked', '--offline', '--jobs', '1', '--test', 'claims', '--no-run', '--message-format=json']
    status = run.command('build', argv)
    messages = [json.loads(line) for line in (run.output / 'build.stdout').read_text().splitlines() if line.startswith('{')]
    diagnostics = [m for m in messages if m.get('reason') == 'compiler-message']
    warnings = [m for m in diagnostics if m['message']['level'] in ('warning', 'error', 'failure-note')]
    stderr_warnings = [line for line in (run.output / 'build.stderr').read_text().splitlines()
        if re.search(r'(^|\s)(warning|error)(\[|:)', line, re.I)]
    write_json(run.output / 'diagnostics.json', {'cargo_exit': status, 'messages': diagnostics,
        'stderr_diagnostics': stderr_warnings})
    require(status == 0 and not warnings and not stderr_warnings, 'build failed or diagnostics require review')
    candidates = [m for m in messages if m.get('reason') == 'compiler-artifact'
        and m['target']['name'] == 'claims' and m.get('executable')]
    require(len(candidates) == 1, 'expected exactly one claims test executable')
    executable = Path(candidates[0]['executable'])
    require(executable.is_relative_to(run.output / 'target'), 'executable escaped target directory')
    require(os.access(regular(executable), os.X_OK), 'test output is not executable')
    with executable.open('rb') as f:
        header = f.read(20)
    require(header[:6] == b'\x7fELF\x02\x01' and int.from_bytes(header[18:20], 'little') == 62, 'not a host x86_64 ELF')
    return {'path': str(executable), 'sha256': digest(executable)}


def execute(run, build_output, approved_executable):
    require(re.fullmatch(r'[a-f0-9]{64}', approved_executable or ''), 'reviewed executable hash required')
    require(build_output.parent == Path('/tmp') and build_output.name.startswith('piv1-sbf-claims-'), 'unexpected build path')
    record = json.loads(regular(build_output / 'result.json').read_text())
    require(record['status'] == 'BUILD_PASS', 'build did not pass')
    require(record['runner_sha256'] == digest(Path(__file__)) and record['pins_sha256'] == digest(PINS), 'build used different reviewed inputs')
    binary = Path(record['executable']['path'])
    require(binary.is_relative_to(build_output / 'target'), 'untrusted executable path')
    require(digest(binary) == approved_executable == record['executable']['sha256'], 'executable drift')
    # No cargo test invocation here: only the separately identified test executable.
    require(run.command('claims', [str(binary), '--test-threads=1', '--nocapture']) == 0, 'runtime assertions failed')
    return {'path': str(binary), 'sha256': approved_executable}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--stage', choices=['preflight', 'build', 'run'], default='preflight')
    parser.add_argument('--approved-runner-sha256', required=True)
    parser.add_argument('--approved-pins-sha256', required=True)
    parser.add_argument('--build-output', type=Path)
    parser.add_argument('--approved-executable-sha256')
    args = parser.parse_args()
    fresh_output(args.output)
    run = Run(args.output)
    (args.output / 'tmp').mkdir(mode=0o700)
    result = {'status': 'FAIL', 'stage': args.stage, 'environment': run.env}
    before = None
    pins = None
    try:
        result['runner_sha256'] = digest(Path(__file__))
        result['pins_sha256'] = digest(PINS)
        require(result['runner_sha256'] == args.approved_runner_sha256, 'runner drift from review')
        require(result['pins_sha256'] == args.approved_pins_sha256, 'pin companion drift from review')
        pins = json.loads(regular(PINS).read_text())
        before = run.git_snapshot('before')
        require(before['branch'].strip() == 'integration/piv1-testnet', 'unexpected branch')
        preflight(run, pins)
        if args.stage == 'build':
            result['executable'] = build(run)
        elif args.stage == 'run':
            require(args.build_output is not None, 'build output required')
            result['executable'] = execute(run, args.build_output, args.approved_executable_sha256)
        result['status'] = {'preflight': 'PREFLIGHT_PASS', 'build': 'BUILD_PASS', 'run': 'RUNTIME_TESTS_PASS'}[args.stage]
    except Exception as error:
        result['error'] = f'{type(error).__name__}: {error}'
    finally:
        # Preservation errors are recorded independently of the original failure.
        try:
            if pins is not None:
                require(source_files() == pins['sources'], 'source changed during command')
                require(digest(ELF) == ELF_SHA, 'historical artifact changed during command')
                verify_artifact(pins['artifact'])
                require(digest(Path(__file__)) == result['runner_sha256'], 'runner changed during command')
                require(digest(PINS) == result['pins_sha256'], 'pins changed during command')
                for path, expected in pins['tools'].items():
                    require(digest(Path(path)) == expected, f'tool changed during command: {path}')
                verify_packages(pins['packages'])
            if before is not None:
                after = run.git_snapshot('after')
                require(before['head'] == after['head'] and before['refs'] == after['refs'], 'Git refs changed')
                write_json(args.output / 'preservation.json', {'before': before, 'after': after, 'status': 'PASS'})
        except Exception as error:
            result['preservation_error'] = f'{type(error).__name__}: {error}'
            result['status'] = 'FAIL'
        result['commands'] = run.commands
        write_json(args.output / 'result.json', result)
    print(json.dumps({'status': result['status'], 'output': str(args.output), 'error': result.get('error'),
        'preservation_error': result.get('preservation_error')}))
    return 0 if result['status'] != 'FAIL' else 1


if __name__ == '__main__':
    sys.exit(main())
