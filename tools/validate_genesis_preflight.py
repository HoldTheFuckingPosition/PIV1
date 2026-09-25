#!/usr/bin/python3
"""Locked/offline two-ELF local preflight validation; no rebuild or live operation.

Build only produces one host test executable. Run requires its separately reviewed
hash. The unchanged claims helper is imported only after byte verification; its
module globals are never reassigned. Every reused command receives an explicit cwd.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import stat
import sys
import tomllib

ROOT = Path('/home/jerem/piv1')
WORKSPACE = ROOT / 'validation/genesis-preflight-runtime'
SCRIPT = ROOT / 'tools/validate_genesis_preflight.py'
PINS = ROOT / 'tools/genesis_preflight_runtime_pins.json'
HELPER = ROOT / 'tools/validate_sbf_claims.py'
HELPER_SHA256 = 'ff4706b6101b57c6385b0ae43b175a3ed252cb40e150f7a3120a84beb7c01fc8'
BASELINE = '352fe7d4ecd8609d93cf2b0a2a96009018d3a7de'
SUSPICIOUS = re.compile(r'(^|[._-])(secret|wallet|keypair|credentials?|seed)([._-]|$)', re.I)


def require(value, message):
    if not value:
        raise RuntimeError(message)


def digest(path):
    require(stat.S_ISREG(path.lstat().st_mode), f'nonregular input (not read): {path}')
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def load_helper():
    require(digest(HELPER) == HELPER_SHA256, 'immutable helper changed; import refused')
    spec = importlib.util.spec_from_file_location('piv1_claims_helper', HELPER)
    helper = importlib.util.module_from_spec(spec)
    sys.dont_write_bytecode = True
    spec.loader.exec_module(helper)
    return helper


def sources():
    paths = [SCRIPT, ROOT / 'tools/test_validate_genesis_preflight.py']
    for path in WORKSPACE.rglob('*'):
        require(not path.is_symlink(), 'workspace symlink refused')
        require(not SUSPICIOUS.search(path.name), 'suspicious input name refused')
        if path.is_file():
            require(path.suffix in ('.toml', '.lock', '.rs', '.md'), 'unexpected workspace input')
            paths.append(path)
    return {str(p.relative_to(ROOT)): digest(p) for p in sorted(paths)}


def registry(lock):
    return {(p['name'], p['version'], p['source'], p['checksum']) for p in lock['package'] if 'source' in p}


def verify_lock(pins):
    baseline = tomllib.loads((ROOT / 'validation/sbf-claims/Cargo.lock').read_text())
    current = tomllib.loads((WORKSPACE / 'Cargo.lock').read_text())
    actual = registry(current)
    require(actual <= registry(baseline), 'new registry version or checksum')
    records = sorted([{'name': n, 'version': v, 'checksum': c} for n, v, _, c in actual], key=lambda p: (p['name'], p['version']))
    require(records == pins['packages'], 'registry closure changed')
    require(sorted((p['name'], p['version']) for p in current['package'] if 'source' not in p)
            == [('piv1', '0.1.0'), ('piv1-genesis-preflight-runtime', '0.1.0'), ('piv1-math', '0.1.0')], 'unexpected local package')


def verify_inputs(helper, pins):
    require(pins['schema'] == 1 and pins['baseline_head'] == BASELINE, 'invalid pin schema/baseline')
    require(digest(HELPER) == HELPER_SHA256, 'helper drift')
    require(sources() == pins['sources'], 'source set/hash drift')
    for relative, expected in pins['protected_inputs'].items():
        require(digest(ROOT / relative) == expected, f'protected input changed: {relative}')
    for path, expected in pins['tools'].items():
        require(digest(Path(path)) == expected, f'tool changed: {path}')
    for path, expected in pins['aliases'].items():
        require(str(Path(path).resolve(strict=True)) == expected, f'tool alias changed: {path}')
    require(set(pins['artifacts']) == {'caller', 'callee'}, 'both exact probe artifacts required')
    for artifact in [*pins['artifacts'].values(), *pins['historical_artifacts']]:
        helper.verify_artifact(artifact)
    helper.reject_configs()
    for directory in [WORKSPACE, *WORKSPACE.parents]:
        for name in ('.cargo/config', '.cargo/config.toml'):
            require(not os.path.lexists(directory / name), 'unexpected runtime Cargo configuration')
    verify_lock(pins)


def preflight(helper, run, pins):
    verify_inputs(helper, pins)
    require(run.command('baseline-ancestry', ['/usr/bin/git', 'merge-base', '--is-ancestor', BASELINE, 'HEAD'], ROOT) == 0, 'baseline not ancestor')
    checked = helper.verify_packages(pins['packages'])
    helper.write_json(run.output / 'packages.json', checked)
    require(run.command('metadata', [str(helper.TOOLCHAIN / 'bin/cargo'), 'metadata', '--manifest-path', str(WORKSPACE / 'Cargo.toml'),
        '--locked', '--offline', '--format-version', '1', '--filter-platform', 'x86_64-unknown-linux-gnu'], WORKSPACE) == 0, 'metadata failed')
    metadata = json.loads((run.output / 'metadata.stdout').read_text())
    require(metadata['resolve'] == pins['resolve'], 'feature graph changed')
    require(metadata['workspace_members'] == [metadata['resolve']['root']], 'workspace isolation changed')
    require(run.command('host-cpu', ['/usr/bin/lscpu'], WORKSPACE) == 0, 'CPU inventory failed')
    for role, artifact in pins['artifacts'].items():
        run.env.update({f'PIV_GENESIS_{role.upper()}_PATH': artifact['path'],
                        f'PIV_GENESIS_{role.upper()}_SHA256': artifact['sha256'],
                        f'PIV_GENESIS_{role.upper()}_BYTES': str(artifact['bytes'])})
    helper.write_json(run.output / 'preflight.json', {'status': 'PREFLIGHT_PASS', 'sources': pins['sources'], 'artifacts': pins['artifacts']})


def build(helper, run):
    status = run.command('build', [str(helper.TOOLCHAIN / 'bin/cargo'), 'test', '--manifest-path', str(WORKSPACE / 'Cargo.toml'),
        '--locked', '--offline', '--jobs', '1', '--test', 'runtime', '--no-run', '--message-format=json'], WORKSPACE)
    messages = [json.loads(line) for line in (run.output / 'build.stdout').read_text().splitlines() if line.startswith('{')]
    diagnostics = [m for m in messages if m.get('reason') == 'compiler-message']
    bad = [m for m in diagnostics if m['message']['level'] in ('warning', 'error', 'failure-note')]
    raw = [line for line in (run.output / 'build.stderr').read_text().splitlines() if re.search(r'(^|\s)(warning|error)(\[|:)', line, re.I)]
    helper.write_json(run.output / 'diagnostics.json', {'cargo_exit': status, 'messages': diagnostics, 'stderr_diagnostics': raw})
    require(status == 0 and not bad and not raw, 'build failed or diagnostics require review')
    candidates = [m for m in messages if m.get('reason') == 'compiler-artifact' and m['target']['name'] == 'runtime' and m.get('executable')]
    require(len(candidates) == 1, 'expected one runtime test executable')
    executable = Path(candidates[0]['executable'])
    require(executable == executable.resolve(strict=True) and executable.is_relative_to(run.output / 'target'), 'untrusted executable path')
    require(os.access(helper.regular(executable), os.X_OK), 'output not executable')
    with executable.open('rb') as stream:
        header = stream.read(20)
    require(header[:6] == b'\x7fELF\x02\x01' and int.from_bytes(header[18:20], 'little') == 62, 'not host x86_64 ELF')
    return {'path': str(executable), 'sha256': digest(executable)}


def checked_output(path, fresh=False):
    require(path.parent == Path('/tmp') and re.fullmatch(r'piv1-genesis-runtime-[a-z0-9-]+', path.name), 'unexpected output path')
    require(not path.is_symlink(), 'output symlink refused')
    if fresh:
        require(not os.path.lexists(path), 'output already exists (untouched)')
        path.mkdir(mode=0o700)
    else:
        require(path == path.resolve(strict=True) and path.is_dir(), 'invalid existing output')


def execute(helper, run, build_output, approved):
    require(re.fullmatch(r'[a-f0-9]{64}', approved or ''), 'reviewed executable hash required')
    checked_output(build_output)
    record = json.loads(helper.regular(build_output / 'result.json').read_text())
    require(record['status'] == 'BUILD_PASS' and record['stage'] == 'build', 'build did not pass')
    require(record['runner_sha256'] == digest(SCRIPT) and record['pins_sha256'] == digest(PINS), 'build input mismatch')
    binary = Path(record['executable']['path'])
    require(binary == binary.resolve(strict=True) and binary.is_relative_to(build_output / 'target'), 'untrusted executable path')
    require(digest(binary) == approved == record['executable']['sha256'], 'executable drift')
    try:
        status = run.command('runtime', [str(binary), '--test-threads=1', '--nocapture'], WORKSPACE)
    finally:
        actual = digest(binary)
        helper.write_json(run.output / 'executable-preservation.json', {'path': str(binary),
            'before_sha256': approved, 'after_sha256': actual, 'status': 'PASS' if actual == approved else 'FAIL'})
        require(actual == approved, 'executable changed during execution')
    require(status == 0, 'runtime assertions failed')
    return {'path': str(binary), 'sha256': approved}


def audit_output(output):
    for path in output.rglob('*'):
        require(not path.is_symlink(), 'output symlink refused')
        require(path.is_file() or path.is_dir(), 'nonregular output')
        require(not SUSPICIOUS.search(path.name), 'suspicious generated output name')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--stage', choices=['preflight', 'build', 'run'], default='preflight')
    parser.add_argument('--approved-runner-sha256', required=True)
    parser.add_argument('--approved-pins-sha256', required=True)
    parser.add_argument('--build-output', type=Path)
    parser.add_argument('--approved-executable-sha256')
    args = parser.parse_args()
    require(Path(__file__).resolve() == SCRIPT and not SCRIPT.is_symlink(), 'unexpected runner location')
    helper = load_helper()
    checked_output(args.output, fresh=True)
    (args.output / 'tmp').mkdir(mode=0o700)
    run = helper.Run(args.output)
    run.env.update(CARGO_PROFILE_DEV_DEBUG='0', CARGO_PROFILE_TEST_DEBUG='0', CARGO_INCREMENTAL='0')
    result = {'status': 'FAIL', 'stage': args.stage, 'environment': run.env}
    pins = before = None
    try:
        result.update(runner_sha256=digest(SCRIPT), pins_sha256=digest(PINS))
        require(result['runner_sha256'] == args.approved_runner_sha256, 'runner drift from review')
        require(result['pins_sha256'] == args.approved_pins_sha256, 'pin drift from review')
        pins = json.loads(helper.regular(PINS).read_text())
        before = run.git_snapshot('before')
        require(before['branch'].strip() == 'integration/piv1-testnet', 'unexpected branch')
        preflight(helper, run, pins)
        if args.stage == 'build':
            result['executable'] = build(helper, run)
        elif args.stage == 'run':
            require(args.build_output is not None, 'build output required')
            result['executable'] = execute(helper, run, args.build_output, args.approved_executable_sha256)
        result['status'] = {'preflight': 'PREFLIGHT_PASS', 'build': 'BUILD_PASS', 'run': 'RUNTIME_TESTS_PASS'}[args.stage]
    except Exception as error:
        result['error'] = f'{type(error).__name__}: {error}'
    finally:
        try:
            if pins is not None:
                verify_inputs(helper, pins)
                helper.verify_packages(pins['packages'])
                require(digest(SCRIPT) == result['runner_sha256'] and digest(PINS) == result['pins_sha256'], 'runner/pin changed during command')
            audit_output(args.output)
            if before is not None:
                after = run.git_snapshot('after')
                require(before['head'] == after['head'] and before['refs'] == after['refs'], 'Git refs changed')
                helper.write_json(args.output / 'preservation.json', {'before': before, 'after': after, 'status': 'PASS'})
        except Exception as error:
            result['preservation_error'] = f'{type(error).__name__}: {error}'
            result['status'] = 'FAIL'
        result['commands'] = run.commands
        helper.write_json(args.output / 'result.json', result)
    print(json.dumps({'status': result['status'], 'output': str(args.output), 'error': result.get('error'), 'preservation_error': result.get('preservation_error')}))
    return int(result['status'] == 'FAIL')


if __name__ == '__main__':
    sys.exit(main())
