#!/usr/bin/python3
"""Build/run the unsigned local Bank genesis test with exact historical ELFs.

Reuse the unchanged Task 2.37 resource/socket guard after verifying its bytes.
This entry point adds artifact identity and separate build/review/run binding.
It does not authorize signing, network access, deployment or live transactions.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import stat

ROOT = Path('/home/jerem/piv1')
SCRIPT = ROOT / 'tools/validate_bank_genesis.py'
PINS = ROOT / 'tools/bank_genesis_pins.json'
HELPER = ROOT / 'tools/validate_bank_smoke.py'
HELPER_SHA256 = 'f5f1af7213c921049840584cde68bc6f672c0883bba830ed2dabc72623ceea20'
ARTIFACT_NAMES = ('caller', 'callee', 'token')
ENV = Path('/usr/bin/env')


def require(value, message):
    if not value:
        raise RuntimeError(message)


def digest(path):
    require(stat.S_ISREG(path.lstat().st_mode), f'nonregular input: {path}')
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def load_helper():
    require(digest(HELPER) == HELPER_SHA256, 'resource helper drift')
    spec = importlib.util.spec_from_file_location('piv1_bank_guard', HELPER)
    helper = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(helper)
    return helper


def verify_artifacts(pins):
    require(set(pins['artifacts']) == set(ARTIFACT_NAMES), 'unexpected runtime artifact set')
    for entry in [*pins['artifacts'].values(), *pins['historical_artifacts']]:
        path = Path(entry['path'])
        require(path.is_absolute() and path == path.resolve(strict=True), 'artifact path alias')
        require(digest(path) == entry['sha256'] and path.stat().st_size == entry['bytes'],
                f'artifact drift: {path}')


def runtime_command(binary, pins):
    require(digest(ENV) == pins['tools'][str(ENV)], 'environment wrapper drift')
    command = [str(ENV)]
    for name in ARTIFACT_NAMES:
        for field, suffix in [('path', 'PATH'), ('sha256', 'SHA256'), ('bytes', 'BYTES')]:
            command.append(f'PIV_GENESIS_{name.upper()}_{suffix}={pins["artifacts"][name][field]}')
    return [*command, str(binary), '--test-threads=1', '--nocapture']


def build_commands(helper):
    cargo = str(helper.TOOLCHAIN / 'cargo')
    manifest = str(helper.WORK / 'Cargo.toml')
    return [cargo, 'metadata', '--manifest-path', manifest, '--locked', '--offline',
            '--format-version', '1', '--filter-platform', 'x86_64-unknown-linux-gnu'], [
            cargo, 'test', '--manifest-path', manifest, '--locked', '--offline', '--jobs', '1',
            '--test', 'genesis_bank', '--no-run', '--message-format=json']


def reviewed_binary(helper, output, approved):
    require(output is not None and re.fullmatch('[a-f0-9]{64}', approved or ''),
            'reviewed build/hash required')
    helper.checked_output(output)
    built = json.loads((output / 'result.json').read_text())
    require(built['status'] == 'BUILD_PASS' and built['runner_sha256'] == digest(SCRIPT)
            and built['pins_sha256'] == digest(PINS) and built['helper_sha256'] == HELPER_SHA256,
            'build input/status mismatch')
    binary = Path(built['binary']['path'])
    require(binary == binary.resolve(strict=True) and binary.is_relative_to(output / 'target'),
            'untrusted executable path')
    require(digest(binary) == approved == built['binary']['sha256'], 'unreviewed executable')
    return binary


def inspect_build(helper, output):
    messages = [json.loads(line) for line in (output / 'build.stdout').read_text().splitlines()
                if line.startswith('{')]
    diagnostics = [m for m in messages if m.get('reason') == 'compiler-message'
                   and m['message']['level'] in ['warning', 'error', 'failure-note']]
    raw = [line for line in (output / 'build.stderr').read_text().splitlines()
           if re.search(r'(^|\s)(warning|error)(\[|:)', line, re.I)]
    helper.write(output / 'diagnostics.json', {'messages': diagnostics, 'stderr_diagnostics': raw})
    require(not diagnostics and not raw, 'diagnostics require review')
    executables = [Path(m['executable']) for m in messages if m.get('reason') == 'compiler-artifact'
                   and m['target']['name'] == 'genesis_bank' and m.get('executable')]
    require(len(executables) == 1, 'expected one genesis_bank executable')
    binary = executables[0]
    require(binary == binary.resolve(strict=True) and binary.is_relative_to(output / 'target'),
            'unexpected build executable')
    return binary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('stage', choices=['build', 'run'])
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--build-output', type=Path)
    parser.add_argument('--approved-sha256')
    args = parser.parse_args()
    helper = load_helper()
    pins = json.loads(PINS.read_text())
    helper.verify(pins)
    verify_artifacts(pins)
    if args.stage == 'run':
        binary = reviewed_binary(helper, args.build_output, args.approved_sha256)
    helper.checked_output(args.output, fresh=True)
    for name in ['tmp', 'openssl-config']:
        (args.output / name).mkdir(mode=0o700)
    result = {'stage': args.stage, 'runner_sha256': digest(SCRIPT),
              'pins_sha256': digest(PINS), 'helper_sha256': HELPER_SHA256}
    try:
        if args.stage == 'build':
            metadata_command, build_command = build_commands(helper)
            helper.monitored(metadata_command, args.output, 'metadata', 60)
            metadata = json.loads((args.output / 'metadata.stdout').read_text())
            require(metadata['resolve'] == pins['resolve'], 'resolved features changed')
            helper.monitored(build_command, args.output, 'build', 1800)
            binary = inspect_build(helper, args.output)
            result.update(status='BUILD_PASS', binary={'path': str(binary), 'sha256': digest(binary)})
        else:
            try:
                helper.monitored(runtime_command(binary, pins), args.output, 'runtime', 60)
            finally:
                require(digest(binary) == args.approved_sha256, 'binary changed during runtime')
            require((args.output / 'runtime.stderr').stat().st_size == 0,
                    'runtime stderr requires review')
            result.update(status='RUN_PASS', binary={'path': str(binary), 'sha256': digest(binary)})
    except BaseException as error:
        result.update(status='FAIL', error=str(error))
        raise
    finally:
        helper.write(args.output / 'result.json', result)


if __name__ == '__main__':
    main()
