#!/usr/bin/python3
"""Pinned local compiler/ELF inspection only; default mode never builds.

Execute only after separate technical review of this file, its pin companion,
and the exact command. Hash arguments bind that reviewed content; they are not
an authorization capability. This is not a sandbox or a supply-chain audit.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import struct
import subprocess
import sys
import time

REPO = Path('/home/jerem/piv1')
PLATFORM = Path('/home/jerem/.cache/solana/v1.54/platform-tools')
CARGO_HOME = Path('/home/jerem/.cargo')
SCRIPT = REPO / 'tools/build_keyless_sbf.py'
PINS = REPO / 'tools/keyless_sbf_pins.json'
CARGO = PLATFORM / 'rust/bin/cargo'
LLVM = PLATFORM / 'llvm/bin'
LINKER = PLATFORM / 'rust/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld'
TARGET = 'sbpf-solana-solana'
BASE_ENV = {'PATH': '/usr/bin:/bin', 'LC_ALL': 'C'}
GIT_ENV = {**BASE_ENV, 'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': '/dev/null',
           'GIT_OPTIONAL_LOCKS': '0'}
SUSPICIOUS = re.compile(r'keypair|credential|secret|wallet|(^|[-_.])id\.json$|\.(pem|p12|pfx|key)$', re.I)
# Preserve every raw diagnostic; fail closed on any mentioned stack bound or
# target error even when LLVM/Cargo returns zero. This is not a runtime verifier.
DIAGNOSTIC_ERROR = re.compile(
    r'\berror(?:\[[^]]+\])?:|LLVM ERROR|stack.{0,100}(?:exceed|limit|offset|too large|overflow)|'
    r'(?:exceed|too large).{0,80}stack|unsupported.{0,80}(?:target|relocation)|'
    r'(?:invalid|unknown).{0,40}(?:target|instruction)|could not compile', re.I)


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def write_json(path, value):
    with path.open('x', encoding='utf-8') as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write('\n')


def sources():
    paths = [REPO / n for n in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml')]
    for base in (REPO / 'programs/piv1', REPO / 'crates/piv1-math'):
        for path in base.rglob('*'):
            require(not path.is_symlink(), f'source symlink: {path}')
            if path.is_file() and (path.suffix == '.rs' or path.name == 'Cargo.toml'):
                paths.append(path)
    return {str(p.relative_to(REPO)): digest(p) for p in sorted(paths)}


def configuration_absence():
    directories = {REPO, *REPO.parents, REPO / 'programs', REPO / 'programs/piv1',
                   REPO / 'crates', REPO / 'crates/piv1-math', Path('/home/jerem')}
    paths = {d / '.cargo' / n for d in directories for n in ('config', 'config.toml')}
    paths.update(CARGO_HOME / n for n in ('config', 'config.toml'))
    for path in sorted(paths):
        require(not os.path.lexists(path), f'unexpected Cargo configuration (not read): {path}')
    return [str(p) for p in sorted(paths)]


def git_state():
    commands = {'user': ['/usr/bin/id', '-un'],
                'branch': ['/usr/bin/git', 'branch', '--show-current'],
                'head': ['/usr/bin/git', 'rev-parse', 'HEAD'],
                'refs': ['/usr/bin/git', 'for-each-ref', '--format=%(refname) %(objectname)',
                         'refs/heads', 'refs/remotes'],
                'status': ['/usr/bin/git', 'status', '--porcelain=v1', '--untracked-files=all']}
    return {name: subprocess.check_output(argv, cwd=REPO, env=GIT_ENV, text=True).strip()
            for name, argv in commands.items()}


def verify_baseline(baseline):
    result = subprocess.run(['/usr/bin/git', 'merge-base', '--is-ancestor', baseline, 'HEAD'],
                            cwd=REPO, env=GIT_ENV, stdin=subprocess.DEVNULL,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    require(result.returncode == 0, 'reviewed closure is not an ancestor of actual HEAD')


def verify_pins(pins):
    require(isinstance(pins, dict), 'pin companion must be a JSON object')
    require(pins['schema'] == 1, 'unsupported pin schema')
    for path, expected in pins['files'].items():
        require(str(Path(path).resolve(strict=True)) == expected['resolved'], f'resolved path drift: {path}')
        require(digest(path) == expected['sha256'], f'tool/support hash drift: {path}')
    for path, expected in pins['reviewed_tools'].items():
        require(pins['files'][path]['sha256'] == expected, f'original reviewed pin changed: {path}')
    for root in pins['tree_roots']:
        actual = {str(p) for p in Path(root).rglob('*') if p.is_file()}
        expected = {p for p in pins['files'] if p.startswith(root + '/')}
        require(actual == expected, f'library file set drift: {root}')
    for script in pins['build_scripts']:
        for path, expected in script['files'].items():
            require(digest(path) == expected, f'build-script input drift: {path}')
    require(sources() == pins['source_files'], 'frozen Task 2.11 source set/hash drift')


def regular_log(path):
    """Only known diagnostic paths may be read after a metadata-only check."""
    require(not path.is_symlink() and stat.S_ISREG(path.stat(follow_symlinks=False).st_mode),
            f'diagnostic log is not a regular file (not read): {path}')
    return path


def allowed_fingerprint(path, output):
    # This package/version is fixed by the independently pinned Cargo.lock.
    # Permit only its ordinary target metadata, never a generic id.json suffix.
    relative = path.relative_to(output).parts
    return (len(relative) == 6
            and relative[:4] == ('target', TARGET, 'release', '.fingerprint')
            and re.fullmatch(r'solana-sysvar-id-[0-9a-f]{16}', relative[4]) is not None
            and relative[5] == 'lib-solana_sysvar_id.json')


def audit_outputs(output):
    """Names/metadata only: never open a suspicious output or follow a symlink."""
    paths = []
    for base, dirs, files in os.walk(output, followlinks=False):
        for name in sorted(dirs + files):
            path = Path(base) / name
            require(not path.is_symlink(), f'unexpected output symlink (not followed): {path}')
            mode = path.stat(follow_symlinks=False).st_mode
            require(stat.S_ISREG(mode) or stat.S_ISDIR(mode), f'unexpected output file type: {path}')
            require(not SUSPICIOUS.search(name)
                    or (stat.S_ISREG(mode) and allowed_fingerprint(path, output)),
                    f'unexpected sensitive output name (not read): {path}')
            paths.append(str(path.relative_to(output)))
    return sorted(paths)


class Run:
    def __init__(self, output, env):
        self.output = output
        self.env = env
        self.commands = []

    def command(self, name, argv, *, must_succeed=True):
        argv = [str(a) for a in argv]
        item = {'name': name, 'argv': argv, 'cwd': str(REPO), 'environment': self.env,
                'started_unix': time.time()}
        self.commands.append(item)
        # Independent streams retain complete bytes, including diagnostics with
        # exit zero. No shell, tee/filter, compiler wrapper or builder is invoked.
        with (self.output / (name + '.stdout')).open('xb') as stdout, \
                (self.output / (name + '.stderr')).open('xb') as stderr:
            result = subprocess.run(argv, cwd=REPO, env=self.env, stdin=subprocess.DEVNULL,
                                    stdout=stdout, stderr=stderr, check=False)
        item.update(exit_code=result.returncode, finished_unix=time.time())
        if must_succeed:
            require(result.returncode == 0, f'{name} exited {result.returncode}; see complete logs')
        return result.returncode

    def stdout(self, name):
        return regular_log(self.output / (name + '.stdout')).read_text(encoding='utf-8', errors='replace')


def preflight(run, pins):
    for name, argv in (
        ('cargo-version', [CARGO, '--version']),
        ('rustc-version', [PLATFORM / 'rust/bin/rustc', '--version', '--verbose']),
        ('rustdoc-version', [PLATFORM / 'rust/bin/rustdoc', '--version']),
        ('clang-version', [LLVM / 'clang', '--version']),
        ('ar-version', [LLVM / 'llvm-ar', '--version']),
        ('objdump-version', [LLVM / 'llvm-objdump', '--version']),
        ('objcopy-version', [LLVM / 'llvm-objcopy', '--version']),
        ('readelf-version', [LLVM / 'llvm-readelf', '--version']),
        ('linker-version', [LINKER, '-flavor', 'gnu', '--version']),
    ):
        run.command(name, argv)
    for index, (path, expected) in enumerate(pins['dynamic_dependencies'].items()):
        name = f'ldd-{index:02}'
        run.command(name, ['/usr/bin/ldd', path])
        text = run.stdout(name)
        actual = sorted(set(re.findall(r'(?:=>\s+|^\s*)(/[^\s]+)', text, re.M)))
        require('not found' not in text and actual == expected, f'dynamic dependency drift: {path}')
    run.command('metadata', [CARGO, 'metadata', '--manifest-path', REPO / 'Cargo.toml',
                            '--locked', '--offline', '--format-version', '1'])
    metadata = json.loads(run.stdout('metadata'))
    actual = sorted((f"{p['name']} {p['version']}", t['src_path'])
                    for p in metadata['packages'] for t in p['targets']
                    if 'custom-build' in t['kind'])
    expected = sorted((s['package'], s['path']) for s in pins['build_scripts'])
    require(actual == expected, 'locked build-script inventory drift')


def classify_diagnostics(output):
    matches = []
    for name in ('build.stdout', 'build.stderr'):
        for number, line in enumerate(regular_log(output / name).read_text(encoding='utf-8', errors='replace').splitlines(), 1):
            if DIAGNOSTIC_ERROR.search(line):
                matches.append({'file': name, 'line': number, 'text': line})
    return matches


def elf_summary(path):
    """Static ELF64 inspection; no loading, execution, header edits or repair."""
    data = path.read_bytes()
    require(data[:6] == b'\x7fELF\x02\x01', 'expected ELF64 little-endian artifact')
    header = struct.unpack_from('<HHIQQQIHHHHHH', data, 16)
    elf_type, machine, version, entry, phoff, shoff, flags, _, phsize, phnum, shsize, shnum, shstr = header
    require(phsize == 56 and shsize == 64 and shstr < shnum, 'unexpected ELF table format')
    require(phoff + phnum * phsize <= len(data) and shoff + shnum * shsize <= len(data),
            'ELF header table outside file')
    segments = [struct.unpack_from('<IIQQQQQQ', data, phoff + i * phsize) for i in range(phnum)]
    sections = [struct.unpack_from('<IIQQQQIIQQ', data, shoff + i * shsize) for i in range(shnum)]
    for section in sections:
        require(section[1] == 8 or section[4] + section[5] <= len(data), 'ELF section outside file')
    for segment in segments:
        require(segment[2] + segment[5] <= len(data) and segment[5] <= segment[6],
                'ELF segment outside file/memory range')
    names_section = sections[shstr]
    names = data[names_section[4]:names_section[4] + names_section[5]]
    named = {}
    for section in sections:
        end = names.find(b'\0', section[0])
        require(end >= 0, 'unterminated ELF section name')
        name = names[section[0]:end].decode('ascii')
        named[name] = section
    require('.text' in named, 'missing .text')
    text = named['.text']
    entry_in_text = text[3] <= entry < text[3] + text[5]
    executable_segments = [i for i, p in enumerate(segments)
                           if p[0] == 1 and p[1] & 1 and p[3] <= entry < p[3] + p[5]
                           and p[2] + entry - p[3] == text[4] + entry - text[3]]
    require(entry_in_text and (entry - text[3]) % 8 == 0 and text[2] & 4,
            'entry is outside executable .text or misaligned')
    require(executable_segments, 'entry is outside file-backed executable PT_LOAD')
    require(elf_type == 3 and machine in (247, 263) and version == 1, 'unexpected ELF type/machine/version')
    # v0 is the explicitly selected compiler target, not a deployment choice.
    require(flags == 0, 'unexpected v0 ELF flags; do not patch the artifact')
    symbols = []
    for section in sections:
        if section[1] not in (2, 11):
            continue
        require(section[9] == 24 and section[6] < len(sections), 'unexpected ELF symbol table')
        strings = sections[section[6]]
        table = data[strings[4]:strings[4] + strings[5]]
        for offset in range(section[4], section[4] + section[5], 24):
            name, info, other, index, value, size = struct.unpack_from('<IBBHQQ', data, offset)
            end = table.find(b'\0', name)
            require(end >= 0, 'unterminated ELF symbol name')
            symbol = table[name:end].decode('utf-8', errors='replace')
            if symbol:
                symbols.append({'name': symbol, 'binding': info >> 4, 'type': info & 15,
                                'visibility': other & 3, 'section': index, 'value': value,
                                'size': size, 'dynamic': section[1] == 11})
    entrypoints = [s for s in symbols if s['name'] == 'entrypoint' and s['section'] != 0]
    require(any(s['dynamic'] and s['binding'] == 1 and s['type'] == 2 and s['visibility'] == 0
                and s['value'] == entry and s['section'] == sections.index(text) for s in entrypoints), 'missing matching exported entrypoint')
    return {'sha256': digest(path), 'bytes': len(data), 'class': 'ELF64', 'endianness': 'little',
            'type': elf_type, 'machine': machine, 'flags': flags, 'entry': entry,
            'text_address': text[3], 'text_bytes': text[5], 'entry_offset': entry - text[3],
            'executable_entry_segments': executable_segments, 'entrypoints': entrypoints,
            'unresolved_symbols': [s for s in symbols if s['section'] == 0],
            'limits': 'Static structure only; no loader, syscall allowlist, stack/heap/compute or execution proof.'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, help='fresh direct /tmp/piv1-keyless-sbf-* directory')
    parser.add_argument('--execute', action='store_true', help='requires completed separate technical review')
    parser.add_argument('--approved-runner-sha256')
    parser.add_argument('--approved-pins-sha256')
    args = parser.parse_args()
    output = Path(args.output)
    require(output.parent == Path('/tmp') and re.fullmatch(r'piv1-keyless-sbf-[A-Za-z0-9-]+', output.name),
            'output must be a fresh direct /tmp/piv1-keyless-sbf-* path')
    require(not os.path.lexists(output), f'output already exists (untouched): {output}')
    require(Path(__file__).resolve() == SCRIPT and not SCRIPT.is_symlink(), 'unexpected runner location')
    require(not PINS.is_symlink() and stat.S_ISREG(PINS.stat().st_mode),
            'pin companion must be a regular file, not a symlink')
    identities = {'runner': digest(SCRIPT), 'pins': digest(PINS)}
    if args.execute:
        require(args.approved_runner_sha256 == identities['runner'], 'reviewed runner hash required/mismatched')
        require(args.approved_pins_sha256 == identities['pins'], 'reviewed pin hash required/mismatched')
    else:
        require(args.approved_runner_sha256 is None and args.approved_pins_sha256 is None,
                'approval hashes belong only to execution mode')
    # No existing output is reused, removed or chmodded, including dangling links.
    os.umask(0o077)
    output.mkdir(mode=0o700)
    env = {**BASE_ENV, 'CARGO_HOME': str(CARGO_HOME),
           'RUSTC': str(PLATFORM / 'rust/bin/rustc'), 'RUSTDOC': str(PLATFORM / 'rust/bin/rustdoc'),
           'CC': str(LLVM / 'clang'), 'AR': str(LLVM / 'llvm-ar'),
           'OBJDUMP': str(LLVM / 'llvm-objdump'), 'OBJCOPY': str(LLVM / 'llvm-objcopy'),
           'CARGO_TARGET_SBPF_SOLANA_SOLANA_RUSTFLAGS': f'-Zremap-cwd-prefix= -C linker={LINKER}',
           'CARGO_TERM_COLOR': 'never', 'TMPDIR': str(output / 'tmp')}
    (output / 'tmp').mkdir(mode=0o700)
    run = Run(output, env)
    argv = [str(CARGO), 'build', '--manifest-path', str(REPO / 'programs/piv1/Cargo.toml'),
            '--package', 'piv1', '--lib', '--release', '--target', TARGET,
            '--locked', '--offline', '--jobs', '1', '--target-dir', str(output / 'target')]
    result = {'mode': 'execute' if args.execute else 'preflight', 'identity': identities,
              'command': argv, 'environment': env, 'output': str(output), 'target_build_started': False,
              'commands': run.commands}
    code = 1
    pins = None
    pins_verified = False
    try:
        pins = json.loads(PINS.read_text())
        verify_pins(pins)
        pins_verified = True
        result['config_paths_absent'] = configuration_absence()
        before = git_state()
        result['git_before'] = before
        require(before['user'] == 'jerem' and before['branch'] == 'integration/piv1-testnet', 'wrong user/branch')
        verify_baseline(pins['baseline_head'])
        result['reviewed_baseline_is_ancestor'] = True
        write_json(output / 'inputs-before.json', pins)
        write_json(output / 'proposed-build.json', {'argv': argv, 'cwd': str(REPO), 'environment': env})
        preflight(run, pins)
        # Preflight may create outputs too; reject them before entering Cargo build.
        audit_outputs(output)
        if args.execute:
            # Recheck immediately before entering the reviewed compiler stage.
            verify_pins(pins)
            configuration_absence()
            require(digest(SCRIPT) == identities['runner'] and digest(PINS) == identities['pins'], 'runner/pins drift')
            result['target_build_started'] = True
            status = run.command('build', argv, must_succeed=False)
            result['compiler_exit_code'] = status
            result['diagnostic_errors'] = classify_diagnostics(output)
            require(status == 0 and not result['diagnostic_errors'], 'target compilation/diagnostics failed')
            # A clean audit is required before inspecting a generated artifact.
            # Finalization records this independently if it fails.
            audit_outputs(output)
            artifact = output / 'target' / TARGET / 'release/piv1.so'
            require(artifact.is_file(), 'compiler produced no piv1.so')
            run.command('elf', [LLVM / 'llvm-readelf', '--file-header', '--program-headers', '--sections',
                                '--symbols', '--dyn-syms', '--relocations', artifact])
            run.command('disassembly', [LLVM / 'llvm-objdump', '--disassemble', '--demangle', artifact])
            result['artifact_identity'] = {'path': str(artifact), 'sha256': digest(artifact),
                                           'bytes': artifact.stat().st_size}
            result['artifact'] = elf_summary(artifact)
        result['status'] = 'STATIC_BUILD_PASS' if args.execute else 'PREFLIGHT_PASS_NO_TARGET_BUILD'
        code = 0
    except KeyboardInterrupt:
        result['status'] = 'FAIL'
        result['error'] = 'interrupted; inspect retained diagnostics and output paths'
    except (OSError, RuntimeError, ValueError, TypeError, KeyError, struct.error, subprocess.SubprocessError) as error:
        result['status'] = 'FAIL'
        result['error'] = str(error)
    finally:
        # An output-audit failure must not hide compiler errors or independent
        # input/ref preservation. Each stage has its own durable outcome.
        try:
            result['output_paths'] = audit_outputs(output)
        except (OSError, RuntimeError) as error:
            result['output_audit_error'] = str(error)
            result['status'] = 'FAIL'
            code = 1
        try:
            result['source_after'] = sources()
            result['git_after'] = git_state()
            require(pins_verified, 'pin verification incomplete; baseline preservation cannot be asserted')
            require(result['source_after'] == pins['source_files'], 'source drift through run')
            require(digest(SCRIPT) == identities['runner'] and digest(PINS) == identities['pins'], 'runner/pins drift through run')
            verify_pins(pins)
            configuration_absence()
            for field in ('user', 'branch', 'head', 'refs'):
                require(result['git_before'][field] == result['git_after'][field], f'Git {field} changed through run')
            result['preservation'] = 'source/tool/runner/pin hashes and protected Git refs unchanged'
        except (OSError, RuntimeError, ValueError, TypeError, KeyError) as error:
            result['preservation_error'] = str(error)
            result['status'] = 'FAIL'
            code = 1
        result['logs'] = {}
        for command in run.commands:
            for suffix in ('.stdout', '.stderr'):
                path = output / (command['name'] + suffix)
                try:
                    regular_log(path)
                    result['logs'][path.name] = {'sha256': digest(path), 'bytes': path.stat().st_size}
                except (OSError, RuntimeError) as error:
                    result.setdefault('diagnostic_log_errors', []).append(str(error))
                    result['status'] = 'FAIL'
                    code = 1
        write_json(output / 'result.json', result)
    print(json.dumps({'status': result['status'], 'evidence': str(output), 'error': result.get('error'),
                      'preservation_error': result.get('preservation_error')}))
    return code


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (OSError, RuntimeError, ValueError) as error:
        print(f'REFUSED: {error}', file=sys.stderr)
        sys.exit(2)
