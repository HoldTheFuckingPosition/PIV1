#!/usr/bin/python3
"""Build-only validation probes; no runtime, signer, deployment or network stage.

The unchanged production runner supplies reviewed tool, output and ELF guards.
Its bytes are checked before import; its globals and production pins are never
overridden. Probe-specific lock/source/configuration checks are additive.
"""
import argparse
import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import stat
import struct
import subprocess
import sys
import tarfile
import tomllib

ROOT = Path('/home/jerem/piv1')
SCRIPT = ROOT / 'tools/build_genesis_initialization_probes.py'
PINS = ROOT / 'tools/genesis_initialization_probe_pins.json'
WORKSPACE = ROOT / 'validation/genesis-initialization-probes'
HELPER = ROOT / 'tools/build_keyless_sbf.py'
HELPER_SHA256 = 'e4455e55a88fe5b22796cc21ef1a938522a5cf55cc226bee3570df9756bf3cb8'
PRODUCTION_PINS = ROOT / 'tools/keyless_sbf_pins.json'
PRODUCTION_PINS_SHA256 = '8bbfca7d66c6612ee970ea6e1058b5dd9efd152aadd4f6c4af86348767238cd5'
PACKAGES = ('piv1-genesis-initialization-probe', 'piv1-genesis-initialization-caller',
            'piv1-genesis-initialization-token')
REGISTRY = 'index.crates.io-1949cf8c6b5b557f'
PROFILE_ADJUSTMENT = {
    'path': '/lib/x86_64-linux-gnu/libexpat.so.1',
    'resolved': '/usr/lib/x86_64-linux-gnu/libexpat.so.1.9.1',
    'old_sha256': 'c42ff317838b4b4639e2ea801905f0317177c6df7e31b2f0d0240e3c3ac0cfde',
    'new_sha256': 'ec6c12d33bb8f9d0e90804121adf19930f36b1b2a4aeb6e1a454b89c7a50c801',
}


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def digest(path):
    require(stat.S_ISREG(path.lstat().st_mode), f'not a regular non-symlink file (not read): {path}')
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def load_helper():
    require(digest(HELPER) == HELPER_SHA256, 'immutable helper changed; import refused')
    spec = importlib.util.spec_from_file_location('piv1_reviewed_sbf_helper', HELPER)
    helper = importlib.util.module_from_spec(spec)
    sys.dont_write_bytecode = True
    spec.loader.exec_module(helper)
    return helper


def current_target_profile(pins):
    """One verified installed-file refresh; no old pins or helper globals change."""
    require(digest(PRODUCTION_PINS) == PRODUCTION_PINS_SHA256, 'production pin drift')
    require(pins['target_profile_adjustment'] == PROFILE_ADJUSTMENT, 'unsupported target profile adjustment')
    original = json.loads(PRODUCTION_PINS.read_text())
    change = PROFILE_ADJUSTMENT
    require(original['files'][change['path']] == {
        'resolved': change['resolved'], 'sha256': change['old_sha256']}, 'unexpected historical file identity')
    require(change['path'] not in original['reviewed_tools'], 'cannot adjust an original reviewed tool')
    current = copy.deepcopy(original)
    current['files'][change['path']]['sha256'] = change['new_sha256']
    return current


def probe_sources(helper):
    paths = [SCRIPT, ROOT / 'tools/test_build_genesis_initialization_probes.py']
    for path in WORKSPACE.rglob('*'):
        require(not path.is_symlink(), f'probe source symlink (not followed): {path}')
        require(not helper.SUSPICIOUS.search(path.name), f'suspicious probe input (not read): {path}')
        if path.is_file():
            require(path.suffix in ('.rs', '.toml', '.lock', '.md'), f'unexpected probe input: {path}')
            paths.append(path)
    return {str(path.relative_to(ROOT)): digest(path) for path in sorted(paths)}


def reject_configs(helper):
    paths = helper.configuration_absence()
    for directory in [WORKSPACE, WORKSPACE / 'callee', WORKSPACE / 'caller', WORKSPACE / 'token', WORKSPACE.parent,
                      helper.CARGO_HOME]:
        names = ('config', 'config.toml', 'credentials', 'credentials.toml') if directory == helper.CARGO_HOME else ('.cargo/config', '.cargo/config.toml')
        for name in names:
            path = directory / name
            require(not os.path.lexists(path), f'unexpected configuration (not read): {path}')
            paths.append(str(path))
    return sorted(set(paths))


def registry_entries(lock):
    return {(p['name'], p['version'], p['source'], p['checksum'])
            for p in lock['package'] if 'source' in p}


def verify_lock(pins):
    baseline = tomllib.loads((ROOT / 'Cargo.lock').read_text())
    probe = tomllib.loads((WORKSPACE / 'Cargo.lock').read_text())
    packages = registry_entries(probe)
    require(packages <= registry_entries(baseline), 'probe introduces a registry version/checksum')
    records = sorted([{'name': name, 'version': version, 'source': source, 'checksum': checksum}
                      for name, version, source, checksum in packages], key=lambda p: (p['name'], p['version']))
    require(records == pins['packages'], 'probe registry closure changed')
    require(sorted((p['name'], p['version']) for p in probe['package'] if 'source' not in p)
            == [('piv1', '0.1.0'), ('piv1-genesis-initialization-caller', '0.1.0'),
                ('piv1-genesis-initialization-probe', '0.1.0'),
                ('piv1-genesis-initialization-token', '0.1.0'), ('piv1-math', '0.1.0')], 'unexpected local package')
    return records


def verify_packages(helper, packages):
    """Compare actual compiler inputs against checksum-bound public archives.

    No extraction, installation, archive script or mutable checksum manifest is
    used. Configuration/credential paths are excluded before Cargo is invoked.
    """
    checked = []
    for package in packages:
        name = f"{package['name']}-{package['version']}"
        archive = helper.CARGO_HOME / 'registry/cache' / REGISTRY / (name + '.crate')
        source = helper.CARGO_HOME / 'registry/src' / REGISTRY / name
        require(digest(archive) == package['checksum'], f'package archive changed: {name}')
        require(source.is_dir() and not source.is_symlink(), f'bad package directory: {name}')
        actual = set()
        for path in source.rglob('*'):
            require(not path.is_symlink(), f'package symlink (not followed): {path}')
            if path.is_file():
                actual.add(str(path.relative_to(source)))
        expected = set()
        with tarfile.open(archive, 'r:gz') as stream:
            for member in stream:
                relative = Path(member.name)
                require(not relative.is_absolute() and '..' not in relative.parts
                        and relative.parts[0] == name, 'unsafe package member')
                if member.isdir():
                    continue
                require(member.isfile(), 'nonregular package member')
                relative = Path(*relative.parts[1:])
                require(str(relative) not in expected, 'duplicate package member')
                expected.add(str(relative))
                with stream.extractfile(member) as contents:
                    expected_hash = hashlib.file_digest(contents, 'sha256').hexdigest()
                require(digest(source / relative) == expected_hash, f'package source changed: {name}/{relative}')
        require(actual - {'.cargo-ok', '.cargo-checksum.json'} == expected, f'package file set changed: {name}')
        checked.append({'package': name, 'checksum': package['checksum'], 'files': len(expected)})
    return checked


def verify_inputs(helper, pins):
    require(pins['schema'] == 1, 'unsupported probe pin schema')
    require(digest(HELPER) == HELPER_SHA256, 'helper drift')
    require(digest(PRODUCTION_PINS) == PRODUCTION_PINS_SHA256, 'production pin drift')
    production = current_target_profile(pins)
    helper.verify_pins(production)
    require(probe_sources(helper) == pins['sources'], 'probe source set/hash drift')
    for name, expected in pins['protected_inputs'].items():
        require(digest(ROOT / name) == expected, f'protected input changed: {name}')
    for artifact in pins['protected_artifacts']:
        path = Path(artifact['path'])
        require(digest(path) == artifact['sha256'] and path.stat().st_size == artifact['bytes'],
                'historical artifact changed')
    verify_lock(pins)
    reject_configs(helper)
    return production


def verify_metadata(helper, pins, production, run):
    run.command('probe-metadata', [helper.CARGO, 'metadata', '--manifest-path', WORKSPACE / 'Cargo.toml',
        '--locked', '--offline', '--format-version', '1'])
    metadata = json.loads(run.stdout('probe-metadata'))
    require(metadata['resolve'] == pins['resolve'], 'probe resolved feature graph changed')
    actual = {(f"{p['name']} {p['version']}", t['src_path']) for p in metadata['packages']
              for t in p['targets'] if 'custom-build' in t['kind']}
    expected = {(s['package'], s['path']) for s in production['build_scripts']}
    require(actual <= expected, 'probe added an unreviewed build script')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True)
    parser.add_argument('--execute', action='store_true')
    parser.add_argument('--approved-runner-sha256')
    parser.add_argument('--approved-pins-sha256')
    args = parser.parse_args()
    output = Path(args.output)
    require(output.parent == Path('/tmp') and re.fullmatch(r'piv1-genesis-initialization-probes-[a-z0-9-]+', output.name), 'unexpected output path')
    require(not os.path.lexists(output), 'output already exists (untouched)')
    require(Path(__file__).resolve() == SCRIPT and not SCRIPT.is_symlink(), 'unexpected runner location')
    identities = {'runner': digest(SCRIPT), 'pins': digest(PINS), 'helper': HELPER_SHA256}
    if args.execute:
        require(args.approved_runner_sha256 == identities['runner'] and args.approved_pins_sha256 == identities['pins'],
                'reviewed runner/pin hashes required')
    else:
        require(args.approved_runner_sha256 is None and args.approved_pins_sha256 is None, 'hash arguments require execute mode')
    helper = load_helper()
    os.umask(0o077)
    output.mkdir(mode=0o700)
    (output / 'tmp').mkdir(mode=0o700)
    env = {**helper.BASE_ENV, 'CARGO_HOME': str(helper.CARGO_HOME),
        'RUSTC': str(helper.PLATFORM / 'rust/bin/rustc'), 'RUSTDOC': str(helper.PLATFORM / 'rust/bin/rustdoc'),
        'CC': str(helper.LLVM / 'clang'), 'AR': str(helper.LLVM / 'llvm-ar'),
        'OBJDUMP': str(helper.LLVM / 'llvm-objdump'), 'OBJCOPY': str(helper.LLVM / 'llvm-objcopy'),
        'CARGO_TARGET_SBPF_SOLANA_SOLANA_RUSTFLAGS': f'-Zremap-cwd-prefix= -C linker={helper.LINKER}',
        'CARGO_TERM_COLOR': 'never', 'CARGO_NET_OFFLINE': 'true', 'TMPDIR': str(output / 'tmp')}
    run = helper.Run(output, env)
    argv = [str(helper.CARGO), 'build', '--manifest-path', str(WORKSPACE / 'Cargo.toml'),
        '--workspace', '--lib', '--release', '--target', helper.TARGET, '--locked', '--offline', '--jobs', '1',
        '--target-dir', str(output / 'target')]
    result = {'identity': identities, 'mode': 'execute' if args.execute else 'preflight',
        'command': argv, 'environment': env, 'commands': run.commands, 'target_build_started': False}
    pins = None
    before = None
    code = 1
    try:
        pins = json.loads(PINS.read_text())
        production = verify_inputs(helper, pins)
        result['target_profile_adjustment'] = pins['target_profile_adjustment']
        before = helper.git_state()
        result['git_before'] = before
        require(before['user'] == 'jerem' and before['branch'] == 'integration/piv1-testnet', 'unexpected user/branch')
        helper.verify_baseline(pins['baseline_head'])
        helper.write_json(output / 'inputs-before.json', pins)
        result['packages'] = verify_packages(helper, pins['packages'])
        helper.preflight(run, production)
        verify_metadata(helper, pins, production, run)
        helper.audit_outputs(output)
        if args.execute:
            verify_inputs(helper, pins)
            require(digest(SCRIPT) == identities['runner'] and digest(PINS) == identities['pins'], 'runner/pins drift')
            result['target_build_started'] = True
            status = run.command('build', argv, must_succeed=False)
            result['compiler_exit_code'] = status
            result['diagnostic_errors'] = helper.classify_diagnostics(output)
            result['diagnostic_warnings'] = [
                {'file': name, 'line': number, 'text': line}
                for name in ('build.stdout', 'build.stderr')
                for number, line in enumerate(helper.regular_log(output / name).read_text(errors='replace').splitlines(), 1)
                if re.search(r'^\s*warning(?:\[[^]]+\])?:', line, re.I)]
            require(status == 0 and not result['diagnostic_errors'] and not result['diagnostic_warnings'],
                    'probe target compilation/diagnostics failed')
            helper.audit_outputs(output)
            result['artifacts'] = {}
            for name in PACKAGES:
                path = output / 'target' / helper.TARGET / 'release' / (name.replace('-', '_') + '.so')
                run.command(name + '-elf', [helper.LLVM / 'llvm-readelf', '--file-header', '--program-headers',
                    '--sections', '--symbols', '--dyn-syms', '--relocations', path])
                run.command(name + '-disassembly', [helper.LLVM / 'llvm-objdump', '--disassemble', '--demangle', path])
                result['artifacts'][name] = {'path': str(path), **helper.elf_summary(path)}
        result['status'] = 'PROBE_STATIC_BUILD_PASS' if args.execute else 'PROBE_PREFLIGHT_PASS_NO_TARGET_BUILD'
        code = 0
    except (KeyboardInterrupt, OSError, RuntimeError, ValueError, TypeError, KeyError,
            struct.error, tarfile.TarError, subprocess.SubprocessError) as error:
        result['status'] = 'FAIL'; result['error'] = str(error) or 'interrupted'
    finally:
        try:
            result['output_paths'] = helper.audit_outputs(output)
        except (OSError, RuntimeError) as error:
            result['output_audit_error'] = str(error); result['status'] = 'FAIL'; code = 1
        try:
            require(pins is not None and before is not None, 'input verification incomplete')
            verify_inputs(helper, pins)
            result['packages_after'] = verify_packages(helper, pins['packages'])
            require(digest(SCRIPT) == identities['runner'] and digest(PINS) == identities['pins'], 'runner/pins drift')
            after = helper.git_state(); result['git_after'] = after
            for field in ('user', 'branch', 'head', 'refs'):
                require(before[field] == after[field], f'protected Git {field} changed')
            result['preservation'] = 'probe/production/package/tool/helper/pin inputs and protected Git refs unchanged'
        except (OSError, RuntimeError, ValueError, TypeError, KeyError, tarfile.TarError) as error:
            result['preservation_error'] = str(error); result['status'] = 'FAIL'; code = 1
        result['logs'] = {}
        for command in run.commands:
            for suffix in ('.stdout', '.stderr'):
                path = output / (command['name'] + suffix)
                try:
                    result['logs'][path.name] = {'sha256': digest(path), 'bytes': path.stat().st_size}
                except (OSError, RuntimeError) as error:
                    result.setdefault('log_errors', []).append(str(error)); result['status'] = 'FAIL'; code = 1
        helper.write_json(output / 'result.json', result)
    print(json.dumps({'status': result['status'], 'output': str(output), 'error': result.get('error')}))
    return code


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (OSError, RuntimeError, ValueError) as error:
        print(f'REFUSED: {error}', file=sys.stderr)
        sys.exit(2)
