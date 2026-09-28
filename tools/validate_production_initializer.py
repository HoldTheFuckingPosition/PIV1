#!/usr/bin/python3
"""M1 real production SBF build and unsigned local Bank validation.

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
SCRIPT = ROOT / 'tools/validate_production_initializer.py'
PINS = ROOT / 'tools/production_initializer_pins.json'
OLD_BUILD = Path('/tmp/piv1-bank-smoke-genesis-build-t238-20260928-a')
CACHE = OLD_BUILD / 'target'
TEST = 'production_initializer_four_profiles'


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


def sbf_profile(helper, pins):
    profile = json.loads((ROOT / 'tools/keyless_sbf_pins.json').read_text())
    profile['source_files'] = {name: pins['sources'][name] for name in helper.sources()}
    for name, change in pins['sbf_tool_adjustments'].items():
        require(profile['files'][name]['sha256'] == change['previous'], 'unexpected old tool pin')
        require(name not in profile['reviewed_tools'], 'cannot change original reviewed tools')
        profile['files'][name]['sha256'] = change['current']
    return profile


def strict_diagnostics(output):
    for path in sorted(output.glob('*.stderr')):
        require(not re.search(r'(^|\s)(warning|error)(\[|:)', path.read_text(), re.I),
                f'diagnostics require review: {path}')


def build_sbf(output, pins, result):
    h = load('build_keyless_sbf', pins)
    guard = load('validate_bank_smoke', pins)
    profile = sbf_profile(h, pins)
    h.verify_pins(profile); h.configuration_absence()
    for base in [ROOT / 'validation', ROOT / 'validation/genesis-production-caller']:
        for name in ['config', 'config.toml']:
            require(not os.path.lexists(base / '.cargo' / name), 'unexpected caller Cargo configuration')
    h.verify_baseline(pins['baseline'])
    env = {**h.BASE_ENV, 'CARGO_HOME': str(h.CARGO_HOME),
        'RUSTC': str(h.PLATFORM / 'rust/bin/rustc'), 'RUSTDOC': str(h.PLATFORM / 'rust/bin/rustdoc'),
        'CC': str(h.LLVM / 'clang'), 'AR': str(h.LLVM / 'llvm-ar'),
        'OBJDUMP': str(h.LLVM / 'llvm-objdump'), 'OBJCOPY': str(h.LLVM / 'llvm-objcopy'),
        'CARGO_TARGET_SBPF_SOLANA_SOLANA_RUSTFLAGS': f'-Zremap-cwd-prefix= -C linker={h.LINKER}',
        'CARGO_TERM_COLOR': 'never', 'TMPDIR': str(output / 'tmp')}
    run = h.Run(output, env); result['commands'] = run.commands
    h.preflight(run, profile)
    artifacts = {}
    (output / 'artifacts').mkdir(mode=0o700)
    for role, manifest, package, filename in [
        ('callee', 'programs/piv1/Cargo.toml', 'piv1', 'piv1.so'),
        ('caller', 'validation/genesis-production-caller/Cargo.toml',
         'piv1-genesis-production-caller', 'piv1_genesis_production_caller.so')]:
        argv = [h.CARGO, 'build', '--manifest-path', ROOT / manifest,
            '--package', package, '--lib', '--release', '--target', h.TARGET,
            '--locked', '--offline', '--jobs', '1', '--target-dir', output / 'target']
        command = ['/usr/bin/env', '-i', *[f'{key}={value}' for key, value in sorted(env.items())], *map(str, argv)]
        guard.monitored(command, output, 'build-' + role, 1800)
        diagnostics = []
        for suffix in ['stdout', 'stderr']:
            log = output / ('build-' + role + '.' + suffix)
            diagnostics.extend({'file': log.name, 'line': number, 'text': line}
                for number, line in enumerate(h.regular_log(log).read_text().splitlines(), 1)
                if h.DIAGNOSTIC_ERROR.search(line))
        result['diagnostics'] = diagnostics
        require(not diagnostics, 'strict SBF compilation failed')
        strict_diagnostics(output); h.audit_outputs(output)
        compiled = output / 'target' / h.TARGET / 'release' / filename
        path = output / 'artifacts' / filename
        with compiled.open('rb') as source, path.open('xb') as target:
            shutil.copyfileobj(source, target)
        require(digest(compiled) == digest(path), 'SBF copy mismatch')
        artifacts[role] = {'path': str(path), 'sha256': digest(path), 'bytes': path.stat().st_size}
        run.command('elf-' + role, [h.LLVM / 'llvm-readelf', '--file-header', '--program-headers',
            '--sections', '--symbols', '--dyn-syms', '--relocations', path])
        run.command('disassembly-' + role, [h.LLVM / 'llvm-objdump', '--disassemble', '--demangle', path])
        result.setdefault('elf', {})[role] = h.elf_summary(path)
    strict_diagnostics(output)
    h.verify_pins(profile); h.audit_outputs(output)
    require(all(digest(Path(entry['path'])) == entry['sha256'] for entry in artifacts.values()), 'SBF artifact drift')
    result.update(status='SBF_PASS', artifacts=artifacts)


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
        'genesis_production', '--no-run', '--message-format=json']), output, 'build', 1800)
    messages = [json.loads(line) for line in (output / 'build.stdout').read_text().splitlines() if line.startswith('{')]
    require(not [m for m in messages if m.get('reason') == 'compiler-message'
        and m['message']['level'] in ['warning', 'error', 'failure-note']], 'compiler diagnostics')
    strict_diagnostics(output)
    paths = [Path(m['executable']) for m in messages if m.get('reason') == 'compiler-artifact'
        and m['target']['name'] == 'genesis_production' and m.get('executable')]
    require(len(paths) == 1, 'expected one production Bank binary')
    binary = paths[0]
    require(binary == binary.resolve(strict=True) and binary.parent == CACHE / 'debug/deps'
            and re.fullmatch('genesis_production-[a-f0-9]{16}', binary.name), 'untrusted cached executable')
    fresh = output / 'genesis_production'
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


def reviewed_sbf(path, pins_hash, approved):
    require(digest(path) == approved, 'unreviewed SBF result')
    return reviewed_result(path, pins_hash, 'SBF_PASS')


def verify_runtime_output(stdout):
    complete = [json.loads(line.split('PIV1_BANK_GENESIS_EVIDENCE ', 1)[1])
        for line in stdout.splitlines() if 'PIV1_BANK_GENESIS_EVIDENCE ' in line]
    complete = [row for row in complete if row.get('kind') == 'complete']
    require('test result: ok. 1 passed; 0 failed;' in stdout and
            f'test {TEST} ...' in stdout and len(complete) == 1 and
            all(complete[0].get(key) == value for key, value in
                {'status': 'PASS', 'profiles': 4, 'message_cases': 12, 'successful_initializations': 4}.items()),
            'missing actual production test completion')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('stage', choices=['sbf', 'bank-build', 'bank-run'])
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--build-output', type=Path)
    parser.add_argument('--sbf-output', type=Path)
    parser.add_argument('--approved-sha256')
    parser.add_argument('--approved-sbf-result-sha256')
    args = parser.parse_args()
    pins = json.loads(PINS.read_text()); verify(pins)
    h = load('validate_bank_smoke', pins); h.verify(bank_profile(pins))
    h.checked_output(args.output, fresh=True)
    for name in ['tmp', 'openssl-config']:
        (args.output / name).mkdir(mode=0o700)
    result = {'stage': args.stage, 'runner_sha256': digest(SCRIPT), 'pins_sha256': digest(PINS)}
    try:
        if args.stage == 'sbf':
            build_sbf(args.output, pins, result)
        elif args.stage == 'bank-build':
            build_bank(h, args.output, pins, result)
        else:
            built = reviewed_result(args.build_output / 'result.json', digest(PINS), 'BUILD_PASS')
            binary = Path(built['binary']['path'])
            require(binary == args.build_output / 'genesis_production'
                and binary == binary.resolve(strict=True)
                and digest(binary) == args.approved_sha256 == built['binary']['sha256'], 'unreviewed binary')
            compiled = reviewed_sbf(args.sbf_output / 'result.json', digest(PINS), args.approved_sbf_result_sha256)
            artifacts = {**compiled['artifacts'], 'token': pins['token']}
            command = ['/usr/bin/env']
            require(set(artifacts) == {'caller', 'callee', 'token'}, 'unexpected artifact set')
            for role, entry in artifacts.items():
                path = Path(entry['path'])
                require(path == path.resolve(strict=True) and digest(path) == entry['sha256']
                        and path.stat().st_size == entry['bytes'], 'artifact drift')
                for field, suffix in [('path', 'PATH'), ('sha256', 'SHA256'), ('bytes', 'BYTES')]:
                    command.append(f'PIV_GENESIS_{role.upper()}_{suffix}={entry[field]}')
            command.extend([str(binary), '--exact', TEST, '--test-threads=1', '--nocapture'])
            try:
                h.monitored(command, args.output, 'runtime', 60)
            finally:
                require(digest(binary) == args.approved_sha256, 'binary drift')
                require(digest(args.sbf_output / 'result.json') == args.approved_sbf_result_sha256, 'SBF result drift')
                for entry in artifacts.values():
                    path = Path(entry['path'])
                    require(digest(path) == entry['sha256'] and path.stat().st_size == entry['bytes'], 'runtime artifact drift')
            stdout = (args.output / 'runtime.stdout').read_text()
            verify_runtime_output(stdout)
            require((args.output / 'runtime.stderr').stat().st_size == 0, 'unexpected runtime stderr')
            require(digest(binary) == args.approved_sha256, 'binary drift')
            result.update(status='RUN_PASS', binary=built['binary'], artifacts=artifacts)
    except BaseException as error:
        result.update(status='FAIL', error=str(error))
        raise
    finally:
        try:
            verify(pins)
            require(digest(SCRIPT) == result['runner_sha256'] and digest(PINS) == result['pins_sha256'], 'runner/pin drift')
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
