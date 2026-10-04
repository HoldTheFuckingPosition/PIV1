#!/usr/bin/python3
"""Complete production lifecycle through real external programs in unsigned local Bank.

Compose hash-bound existing guards, preserving historical evidence. Only the fixed
Task 2.38 compiler cache may change; each command log and runnable binary is new.
No signing, deployment, secrets or live operations.
"""
import argparse
import importlib.util
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


def verify_runtime_output(stdout):
    records = [json.loads(line.split('PIV1_LIFECYCLE_EVIDENCE ', 1)[1])
        for line in stdout.splitlines() if 'PIV1_LIFECYCLE_EVIDENCE ' in line]
    complete = [row for row in records if row.get('kind') == 'lifecycle_complete']
    require('test result: ok. 1 passed; 0 failed;' in stdout and
            f'test {TEST} ...' in stdout and len(complete) == 1 and
            all(complete[0].get(key) == value for key, value in
                {'status': 'PASS', 'legs': 2, 'pending_sol': 0, 'pending_tokens': 0,
                 'live_ready': False, 'signature_verified': False,
                 'actual_programs': ['PIV1', 'System', 'Token8.0.0', 'SPLpool2.0.3', 'Stake5.1.0']}.items()),
            'missing actual lifecycle completion')
    return complete[0]


def verify_artifacts(pins):
    require(set(pins['artifacts']) == {'piv', 'caller', 'token', 'pool', 'stake'}, 'unexpected program set')
    for entry in pins['artifacts'].values():
        path = Path(entry['path'])
        require(path == path.resolve(strict=True) and digest(path) == entry['sha256']
                and path.stat().st_size == entry['bytes'], 'program artifact drift')

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('stage', choices=['bank-build', 'bank-run'])
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--build-output', type=Path)
    parser.add_argument('--approved-sha256')
    args = parser.parse_args()
    pins = json.loads(PINS.read_text()); verify(pins); verify_artifacts(pins)
    h = load('validate_bank_smoke', pins); h.verify(bank_profile(pins))
    git = load('build_keyless_sbf', pins)
    before = git.git_state()
    require(before['user'] == 'jerem' and before['branch'] == 'integration/piv1-testnet'
            and before['head'] == pins['baseline'], 'wrong repository state')
    h.checked_output(args.output, fresh=True)
    for name in ['tmp', 'openssl-config']:
        (args.output / name).mkdir(mode=0o700)
    result = {'stage': args.stage, 'runner_sha256': digest(SCRIPT), 'pins_sha256': digest(PINS)}
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
            command.extend([str(binary), '--exact', TEST, '--test-threads=1', '--nocapture'])
            try:
                h.monitored(command, args.output, 'runtime', 300)
            finally:
                require(digest(binary) == args.approved_sha256, 'binary drift')
                for entry in artifacts.values():
                    path = Path(entry['path'])
                    require(digest(path) == entry['sha256'] and path.stat().st_size == entry['bytes'], 'runtime artifact drift')
            stdout = (args.output / 'runtime.stdout').read_text()
            result['completion'] = verify_runtime_output(stdout)
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
