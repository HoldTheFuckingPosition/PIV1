"""Private public-only SDK adapter for exactly Buffer-create/first-Write.

The release operator owns authorization, native signing, TLS, journal and HOLD.
This worker retains only static messages and runs the unchanged public checker.
"""
import base64
import os
from pathlib import Path
import selectors
import subprocess
import time


# Read, pin, then compile the exact worker bytes; never execute a second path read.
BOOTSTRAP = r'''
const fs=require('node:fs'),crypto=require('node:crypto'),Module=require('node:module');
const file=process.argv[1],want=process.argv[2];
const fd=fs.openSync(file,fs.constants.O_RDONLY|fs.constants.O_NOFOLLOW|fs.constants.O_NONBLOCK);
let raw;try{const st=fs.fstatSync(fd);if(!st.isFile()||st.size>65536)throw Error('worker_source');raw=fs.readFileSync(fd);}finally{fs.closeSync(fd);}
if(crypto.createHash('sha256').update(raw).digest('hex')!==want)throw Error('worker_source');
const m=new Module(file);m.filename=file;m.paths=Module._nodeModulePaths(require('node:path').dirname(file));
m._compile(raw.toString('utf8'),file);m.exports.main().catch(()=>{process.stderr.write('sdk_worker_refused\n');process.exitCode=1;});
'''


class BufferSdkClient:
    def __init__(self, operator, journal, manifest, unchanged):
        self.o = operator
        self.j = journal
        self.unchanged = unchanged
        self.process = None
        self.buffer = bytearray()
        self.stderr = bytearray()
        self.closed = False
        self.repo = journal.pins()
        self.sources = {self.repo/name:(self.repo/name).read_bytes() for name in
            ("tools/startup_minimal_buffer.py", "tools/startup_minimal_buffer.cjs")}
        self.guard("sdk_prepare_before")
        try:
            self.process = subprocess.Popen([str(journal.NODE), "-e", BOOTSTRAP,
                str(self.repo/"tools/startup_minimal_buffer.cjs"),
                operator.sha(self.sources[self.repo/"tools/startup_minimal_buffer.cjs"])],
                cwd=self.repo, env={"PIV1_PUBLIC_CHECK_REPO":str(self.repo)}, stdin=subprocess.PIPE,
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, close_fds=True,
                preexec_fn=operator.disable_signer_core_dumps)
            for stream in (self.process.stdin,self.process.stdout,self.process.stderr): os.set_blocking(stream.fileno(),False)
            reply = self.request({"action":"PREPARE", "workerSha256":operator.sha(self.sources[self.repo/"tools/startup_minimal_buffer.cjs"]),
                "reviews":[manifest["phases"][index]["reviewed"] for index in (0,1)]})
            operator.need(reply == {"sdk":"1.98.4", "preparedPhases":[0,1]}, "sdk_preparation_reply")
            self.guard("sdk_prepare_after")
        except BaseException:
            self.close()
            raise

    def guard(self, boundary):
        self.o.need(not self.closed, "sdk_operation_consumed")
        self.unchanged(boundary)
        for path,raw in self.sources.items():
            self.o.need(not path.is_symlink() and path.read_bytes() == raw, "sdk_source_changed")
        if self.process is not None:
            while True:
                try: chunk = os.read(self.process.stderr.fileno(),16385)
                except BlockingIOError: break
                if not chunk: break
                self.stderr.extend(chunk)
                self.o.need(len(self.stderr) <= 16384, "sdk_stderr_bound")
            self.o.need(not self.stderr, "sdk_checker_stderr")

    def request(self, value):
        self.guard("sdk_request_before")
        raw = self.o.encode(value)
        self.o.need(len(raw) <= 24*1024*1024, "sdk_request_bound")
        deadline = time.monotonic()+20
        selector = selectors.DefaultSelector()
        position = 0
        try:
            selector.register(self.process.stdin,selectors.EVENT_WRITE)
            selector.register(self.process.stdout,selectors.EVENT_READ)
            selector.register(self.process.stderr,selectors.EVENT_READ)
            while b"\n" not in self.buffer:
                self.o.need(time.monotonic() < deadline and self.process.poll() is None, "sdk_worker_unresolved")
                for event,_ in selector.select(.25):
                    if event.fileobj is self.process.stdin:
                        position += os.write(self.process.stdin.fileno(),raw[position:position+65536])
                        if position == len(raw): selector.unregister(self.process.stdin)
                    elif event.fileobj is self.process.stdout:
                        chunk = os.read(self.process.stdout.fileno(),65536)
                        self.o.need(bool(chunk), "sdk_worker_unresolved")
                        self.buffer.extend(chunk)
                        self.o.need(len(self.buffer) <= 512*1024, "sdk_reply_bound")
                    else:
                        chunk = os.read(self.process.stderr.fileno(),16385)
                        self.stderr.extend(chunk)
                        self.o.need(len(self.stderr) <= 16384, "sdk_stderr_bound")
                        self.o.need(not self.stderr, "sdk_checker_stderr")
                        if not chunk: selector.unregister(self.process.stderr)
            self.o.need(position == len(raw), "sdk_early_reply")
            line,_,rest = self.buffer.partition(b"\n")
            self.o.need(not rest, "sdk_unsolicited_reply")
            self.buffer.clear()
            result = self.j.parse(bytes(line))
            self.o.fields(result,("result",))
            self.guard("sdk_request_after")
            return result["result"]
        except BaseException:
            self.close()  # No command, hash refresh, worker replacement or adoption.
            raise
        finally: selector.close()

    def install_checks(self, manifest_api, journal):
        """Only private modules of this locked operation; never global dispatch."""
        original = manifest_api._candidate_python_helpers
        def attach(module):
            def check(inputs, planner=False, *, candidate_planner=False):
                self.o.need(planner is False and candidate_planner is False, "sdk_checker_only")
                module.pins()  # Preserve the original per-call runtime/source gate.
                return self.request({"action":"CHECK", "inputs":inputs})
            module._node_request = check
        def helpers():
            repo,modules = original()
            attach(modules[0])
            return repo,modules
        attach(journal)
        manifest_api._candidate_python_helpers = helpers

    def prepare_manifest(self, manifest_api, manifest):
        """Preconstruct static semantics once; never retain a dynamic check."""
        raw = self.o.encode(manifest_api.candidate_validate(manifest))
        self.guard("sdk_static_manifest_prepared")
        def validate(value):
            # These are the original constructor's fresh source/runtime/artifact
            # gates. Exact complete input bytes must still match its rebuild.
            self.guard("sdk_static_manifest_before")
            manifest_api._candidate_helpers()
            manifest_api.candidate_oracle()
            self.o.need(type(value) is dict and self.o.encode(value) == raw, "sdk_static_manifest_changed")
            manifest_api._candidate_image(value["elfBase64"])
            self.guard("sdk_static_manifest_after")
            return self.j.parse(raw)  # No mutable alias into the private proof.
        manifest_api.candidate_validate = validate

    def bind(self, index, blockhash, expected):
        result = self.request({"action":"BIND", "index":index, "blockhash":blockhash})
        self.o.fields(result,("wireBase64","reviewed"))
        self.o.need(self.o.encode(result["reviewed"]) == self.o.encode(expected), "sdk_exact_builder_mismatch")
        count = len(expected["requiredSigners"])
        wire = bytes([count])+bytes(64*count)+base64.b64decode(expected["messageBase64"],validate=True)
        self.o.need(result["wireBase64"] == base64.b64encode(wire).decode(), "sdk_exact_unsigned_mismatch")
        return result["wireBase64"]

    def verify(self, index, pending):
        result = self.request({"action":"VERIFY", "index":index, "wireBase64":pending["input"]["wireBase64"]})
        self.o.need(result == {key:pending["identity"][key] for key in ("wireSha256","messageSha256","signatureHex")},
            "sdk_exact_signed_mismatch")

    def close(self):
        self.closed = True
        if self.process is not None:
            if self.process.poll() is None: self.process.kill()
            self.process.wait()
            for stream in (self.process.stdin,self.process.stdout,self.process.stderr):
                if not stream.closed: stream.close()
