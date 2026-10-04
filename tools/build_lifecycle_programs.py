#!/usr/bin/python3
"""Build exact upstream lifecycle programs offline; never execute a transaction."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import stat

ROOT = Path('/home/jerem/piv1')
PINS = ROOT / 'tools/lifecycle_program_pins.json'
SCRIPT = Path(__file__).resolve()


def require(value, message):
    if not value:
        raise RuntimeError(message)


def digest(path):
    path = Path(path)
    require(path.is_file() and not path.is_symlink(), f'nonregular input: {path}')
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def save(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write('\n')


def load(name, pins):
    path = ROOT / 'tools' / (name + '.py')
    require(digest(path) == pins['helpers'][str(path.relative_to(ROOT))], 'helper drift')
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def verify(pins):
    require(os.getuid() == 1001 and pins['schema'] == 1, 'wrong user/schema')
    for group in ['production_sources', 'workspace_sources', 'helpers']:
        for name, expected in pins[group].items():
            require(digest(ROOT / name) == expected, f'input drift: {name}')
    guard_pins = json.loads((ROOT / 'tools/bank_genesis_pins.json').read_text())
    require(str(Path('/usr/bin/env').resolve(strict=True)) == guard_pins['aliases']['/usr/bin/env']
            and digest('/usr/bin/env') == guard_pins['tools']['/usr/bin/env'], 'environment launcher drift')
    for package in pins['packages']:
        require(digest(package['archive']) == package['checksum'], 'archive drift')
        root = Path(package['source'])
        require(root == root.resolve(strict=True), 'aliased registry root')
        files = []
        for path in sorted(root.rglob('*')):
            require(not path.is_symlink(), f'registry symlink: {path}')
            if path.is_file() and path.name not in ['.cargo-ok', '.cargo-checksum.json']:
                files.append({'path': str(path.relative_to(root)), 'bytes': path.stat().st_size,
                              'sha256': digest(path)})
        files.sort(key=lambda row: row['path'])
        actual = hashlib.sha256(json.dumps(files, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
        require(actual == package['tree_sha256'] and len(files) == package['file_count'], 'registry tree drift')
    for artifact in pins['prior_artifacts'].values():
        path = Path(artifact['path'])
        require(path == path.resolve(strict=True) and digest(path) == artifact['sha256']
                and path.stat().st_size == artifact['bytes'], 'prior artifact drift')
    for directory in [ROOT / 'validation', ROOT / 'validation/pinned-pool-programs',
                      ROOT / 'validation/genesis-bank-runtime', ROOT / 'validation/pinned-token-program']:
        for name in ['config', 'config.toml']:
            require(not os.path.lexists(directory / '.cargo' / name), 'unexpected Cargo configuration')


def audit_outputs(helper, output):
    """Keep the existing name guard for two isolated Cargo target directories."""
    paths = []
    for base, directories, files in os.walk(output, followlinks=False):
        for name in sorted(directories + files):
            path = Path(base) / name
            require(not path.is_symlink(), f'unexpected output symlink (not followed): {path}')
            mode = path.stat(follow_symlinks=False).st_mode
            require(stat.S_ISREG(mode) or stat.S_ISDIR(mode), f'unexpected output file type: {path}')
            parts = path.relative_to(output).parts
            mapped = output / 'target' / Path(*parts[1:])
            allowed = (stat.S_ISREG(mode) and parts[0] in ['target-token', 'target-pool']
                       and helper.allowed_fingerprint(mapped, output))
            require(not helper.SUSPICIOUS.search(name) or allowed,
                    f'unexpected sensitive output name (not read): {path}')
            paths.append(str(path.relative_to(output)))
    return sorted(paths)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    pins = json.loads(PINS.read_text())
    verify(pins)
    h = load('build_keyless_sbf', pins)
    guard = load('validate_bank_smoke', pins)
    profile = json.loads((ROOT / 'tools/keyless_sbf_pins.json').read_text())
    profile['source_files'] = pins['production_sources']
    for name, change in pins['sbf_tool_adjustments'].items():
        require(profile['files'][name]['sha256'] == change['previous'], 'old tool pin differs')
        require(name not in profile['reviewed_tools'], 'reviewed tool override refused')
        profile['files'][name]['sha256'] = change['current']
    h.verify_pins(profile)
    h.configuration_absence()
    before = h.git_state()
    require(before['user'] == 'jerem' and before['branch'] == 'integration/piv1-testnet'
            and before['head'] == pins['baseline'], 'wrong repository state')
    guard.checked_output(args.output, fresh=True)
    (args.output / 'tmp').mkdir(mode=0o700)
    (args.output / 'artifacts').mkdir(mode=0o700)
    env = {**h.BASE_ENV, 'CARGO_HOME': str(h.CARGO_HOME),
           'RUSTC': str(h.PLATFORM / 'rust/bin/rustc'), 'RUSTDOC': str(h.PLATFORM / 'rust/bin/rustdoc'),
           'CC': str(h.LLVM / 'clang'), 'AR': str(h.LLVM / 'llvm-ar'),
           'OBJDUMP': str(h.LLVM / 'llvm-objdump'), 'OBJCOPY': str(h.LLVM / 'llvm-objcopy'),
           'CARGO_TARGET_SBPF_SOLANA_SOLANA_RUSTFLAGS': f'-Zremap-cwd-prefix= -C linker={h.LINKER}',
           'CARGO_NET_OFFLINE': 'true', 'CARGO_TERM_COLOR': 'never', 'TMPDIR': str(args.output / 'tmp')}
    run = h.Run(args.output, env)
    result = {'status': 'IN_PROGRESS', 'runner_sha256': digest(SCRIPT), 'pins_sha256': digest(PINS),
              'artifacts': {}, 'commands': run.commands,
              'limits': 'Actual upstream SBF compilation only; no runtime or live deployment proof.'}
    save(args.output / 'inputs.json', pins)
    try:
        h.preflight(run, profile)
        audit_outputs(h, args.output)
        for role, package, filename, workspace in [
                ('token', 'spl-token@8.0.0', 'spl_token.so', 'pinned-token-program'),
                ('pool', 'spl-stake-pool@2.0.3', 'spl_stake_pool.so', 'pinned-pool-programs')]:
            manifest = ROOT / 'validation' / workspace / 'Cargo.toml'
            run.command('metadata-' + role, [h.CARGO, 'metadata', '--manifest-path', manifest,
                        '--locked', '--offline', '--format-version', '1'])
            metadata = json.loads(run.stdout('metadata-' + role))
            selected = [p for p in metadata['packages']
                        if p['name'] + '@' + p['version'] == package]
            require(len(selected) == 1, 'missing exact upstream package')
            node = next(n for n in metadata['resolve']['nodes'] if n['id'] == selected[0]['id'])
            require('no-entrypoint' not in node['features'], 'upstream entrypoint disabled')
            argv = [h.CARGO, 'build', '--manifest-path', manifest, '--package', package,
                    '--lib', '--release', '--target', h.TARGET, '--locked', '--offline',
                    '--jobs', '1', '--target-dir', args.output / ('target-' + role)]
            command = ['/usr/bin/env', '-i', *[f'{k}={v}' for k, v in sorted(env.items())], *map(str, argv)]
            guard.monitored(command, args.output, 'build-' + role, 1800)
            for suffix in ['stdout', 'stderr']:
                path = args.output / ('build-' + role + '.' + suffix)
                for line in h.regular_log(path).read_text().splitlines():
                    require(not h.DIAGNOSTIC_ERROR.search(line)
                            and not re.search(r'(^|\s)(warning|error)(\[|:)', line, re.I),
                            'strict compiler/stack diagnostic')
            for path in args.output.glob('*.stderr'):
                text = h.regular_log(path).read_text()
                require(not h.DIAGNOSTIC_ERROR.search(text)
                        and not re.search(r'(^|\s)(warning|error)(\[|:)', text, re.I), 'strict compiler diagnostic')
            audit_outputs(h, args.output)
            source = args.output / ('target-' + role) / h.TARGET / 'release' / filename
            path = args.output / 'artifacts' / filename
            with source.open('rb') as src, path.open('xb') as dst:
                shutil.copyfileobj(src, dst)
            require(digest(source) == digest(path), 'artifact copy mismatch')
            run.command('elf-' + role, [h.LLVM / 'llvm-readelf', '--file-header', '--program-headers',
                        '--sections', '--symbols', '--dyn-syms', '--relocations', path])
            run.command('disassembly-' + role, [h.LLVM / 'llvm-objdump', '--disassemble', '--demangle', path])
            result['artifacts'][role] = {'path': str(path), 'sha256': digest(path), 'bytes': path.stat().st_size}
            result.setdefault('elf', {})[role] = h.elf_summary(path)
            require(re.search(r'\bentrypoint\b', run.stdout('elf-' + role)), 'missing executable entrypoint')
        for path in args.output.glob('*.stderr'):
            require(not re.search(r'(^|\s)(warning|error)(\[|:)', h.regular_log(path).read_text(), re.I),
                    'post-inspection diagnostic')
        result['status'] = 'PASS'
    except BaseException as error:
        result.update(status='FAIL', error=str(error))
        raise
    finally:
        try:
            verify(pins)
            h.verify_pins(profile)
            h.configuration_absence()
            audit_outputs(h, args.output)
            after = h.git_state()
            require(all(before[k] == after[k] for k in ['user', 'branch', 'head', 'refs']), 'protected Git drift')
            require(digest(SCRIPT) == result['runner_sha256'] and digest(PINS) == result['pins_sha256'], 'runner drift')
            result['preservation'] = 'PASS; unrelated Bank fixture work was outside the compilation input freeze.'
        except BaseException as error:
            result.update(status='FAIL', preservation=str(error))
            raise
        finally:
            result['logs'] = {p.name: digest(p) for p in args.output.iterdir() if p.suffix in ['.stdout', '.stderr']}
            save(args.output / 'result.json', result)
    print(json.dumps({'status': result['status'], 'artifacts': result['artifacts']}))


if __name__ == '__main__':
    main()
