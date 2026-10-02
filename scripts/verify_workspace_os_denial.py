#!/usr/bin/env python3
"""Bind a bounded Linux headless IP-denial experiment to exact protected inputs; no attempt or acceptance credit."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import selectors
import shutil
import stat
import subprocess
import sys
import tempfile
import time

import verify_workspace as shared

sys.dont_write_bytecode = True
SCHEMA = "forge.workspace-os-denial-verification/1"
NATIVE_SCHEMA = "forge.packaged-runtime-os-denial/1"
SCOPE = "linux-headless-ip-network-denial-prerequisite"
OUTPUT = "workspace-os-denial-verification.json"
NATIVE_OUTPUT = "os-denial-smoke.json"
CLIENT_OUTPUT = "client-verification.json"
MAX_CAPTURE = 262144
MAX_ENTRIES = 20000
MAX_STDLIB_BYTES = 268435456
SOURCE_KEYS = ("scripts/test_workspace_client.py", "scripts/workspace_client.py",
               "scripts/verify_workspace.py", "docs/api/forge-workspace-v1.openapi.yaml")
EXTRA_INPUTS = ("scripts/test_workspace_os_denial.py", "scripts/verify_workspace_os_denial.py",
                "scripts/test_verify_workspace_os_denial.py")
QUALIFICATION = ("facilities_qualified", "three_owned_namespaces", "dut_lo_only_before",
                 "dut_lo_only_after", "canary_dual_family", "client_preexec_fence", "protected_execution")
OWNED = ("owned_processes_empty", "owned_references_closed", "owned_names_absent", "virtual_links_absent")
FAILURES = frozenset(("unsupported-platform", "tool-unavailable", "tool-untrusted", "facility-unavailable",
    "privilege-unavailable", "source-invalid", "source-changed", "privilege-fence", "namespace-unverified",
    "probe-calibration-failed", "probe-denial-unverified", "loopback-unverified", "client-failed",
    "client-receipt-invalid", "command-timeout", "output-bound", "cleanup-unverified",
    "execution-unverified", "publication-failed"))
PENDING = ("full-F04-Must-and-Should-acceptance", "runtime-attempted-egress-observation",
    "browser-and-remaining-platform-matrix", "manual-assistive-technology-and-human-acceptance",
    "D069-accepted-source-audits", "release-qualification", "final-integrated-documentation-review")
OUTER_FAILURES = FAILURES | frozenset(("build-not-qualified", "tracked-source-unclean", "tool-identity-changed", "verification-input-invalid"))
PROBE_ORDER = tuple((phase, family, role) for phase in ("before", "after")
    for family in ("ipv4", "ipv6") for role in ("calibration-external", "dut-external", "dut-loopback"))
PYTHON_PROBE = ("import json,sys;print(json.dumps({'version':'.'.join(map(str,sys.version_info[:3])),"
                "'paths':sys.path},sort_keys=True,separators=(',',':')))")
# This fixed bootstrap does not import checkout modules and executes only the single hash-verified buffer.
BOOTSTRAP = """import hashlib,os,stat,sys
p,h=sys.argv[1:];f=os.open(p,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
try:
 s=os.fstat(f)
 if not stat.S_ISREG(s.st_mode) or not 0<s.st_size<=131072:raise RuntimeError('source')
 b=b''
 while len(b)<=131072:
  c=os.read(f,min(8192,131073-len(b)))
  if not c:break
  b+=c
 t=os.fstat(f)
 if (s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns)!=(t.st_dev,t.st_ino,t.st_size,t.st_mtime_ns,t.st_ctime_ns) or len(b)!=s.st_size or hashlib.sha256(b).hexdigest()!=h:raise RuntimeError('source')
finally:os.close(f)
sys.argv=[p,'--plan-stdin'];exec(compile(b,p,'exec'),{'__name__':'__main__','__file__':p})
"""


class GateError(ValueError):
    """Carry one fixed public phase failure without retaining arbitrary exception text or private paths."""

    def __init__(self, code, incomplete=False):
        """Retain only a known fixed code and whether execution had not yet created any resources."""
        if code not in OUTER_FAILURES:
            raise ValueError("unknown failure code")
        self.code = code
        self.incomplete = incomplete
        super().__init__(code)


def closed(value, keys):
    """Reject unknown or missing receipt members rather than infer producer observations."""
    if type(value) is not dict or set(value) != set(keys):
        raise GateError("execution-unverified")


def integer(value, high, low=0):
    """Reject booleans, negative values and counters outside the explicit inclusive bound."""
    if type(value) is not int or not low <= value <= high:
        raise GateError("execution-unverified")
    return value


def strict_json(raw):
    """Decode one bounded duplicate-safe JSON value and enforce decoded depth without allocation claims."""
    if type(raw) is not bytes or not raw or len(raw) > MAX_CAPTURE:
        raise GateError("output-bound")
    def pairs(rows):
        """Reject nested or escaped duplicate keys before a dictionary can collapse them."""
        value = {}
        for key, item in rows:
            if key in value:
                raise GateError("execution-unverified")
            value[key] = item
        return value
    def constant(_value):
        """Reject nonfinite JSON tokens without copying their representation into public errors."""
        raise GateError("execution-unverified")
    value = json.loads(raw, object_pairs_hook=pairs, parse_constant=constant)
    def depth(item, level):
        """Bound decoded container depth after parsing; byte caps do not alone confine decoder allocation."""
        if level > 16:
            raise GateError("output-bound")
        if isinstance(item, dict):
            for child in item.values():
                depth(child, level + 1)
        elif isinstance(item, list):
            for child in item:
                depth(child, level + 1)
    depth(value, 0)
    return value


def metadata(item):
    """Compare complete relevant descriptor/path metadata to detect substitution or mutation during reads."""
    return (item.st_dev, item.st_ino, item.st_mode, item.st_uid, item.st_gid,
            item.st_size, item.st_mtime_ns, item.st_ctime_ns)


def read_bytes(path, limit=MAX_CAPTURE):
    """Read bounded unchanged regular bytes with no link or special-file opens."""
    path = Path(path)
    before = path.lstat()
    if not stat.S_ISREG(before.st_mode) or not 0 < before.st_size <= limit:
        raise GateError("source-invalid")
    descriptor = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | os.O_NONBLOCK)
    try:
        opened = os.fstat(descriptor)
        if metadata(before) != metadata(opened):
            raise GateError("source-changed")
        chunks = []
        total = 0
        while total <= limit:
            chunk = os.read(descriptor, min(8192, limit + 1 - total))
            if not chunk:
                break
            chunks.append(chunk)
            total += len(chunk)
        if total > limit or total != opened.st_size or metadata(opened) != metadata(os.fstat(descriptor)) or metadata(opened) != metadata(path.lstat()):
            raise GateError("source-changed")
        return b"".join(chunks)
    finally:
        os.close(descriptor)


def pin_bytes(raw):
    """Bind exact private bytes rather than a parsed reconstruction or a supplied filename."""
    return {"bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}


def pin(value):
    """Validate the closed SHA256/length tuple, including exact integer typing."""
    closed(value, ("bytes", "sha256"))
    integer(value["bytes"], shared.MAX_INPUT)
    if type(value["sha256"]) is not str or not re.fullmatch(r"[0-9a-f]{64}", value["sha256"]):
        raise GateError("execution-unverified")
    return value


def root_trusted(path, directory=False):
    """Require the canonical target and every ancestor to be root-owned and not worker-writable."""
    path = Path(path)
    if not path.is_absolute():
        raise GateError("tool-untrusted")
    item = path
    while True:
        info = item.lstat()
        if info.st_uid != 0 or info.st_mode & 0o022 or stat.S_ISLNK(info.st_mode):
            raise GateError("tool-untrusted")
        if item == path:
            if directory and not stat.S_ISDIR(info.st_mode) or not directory and not stat.S_ISREG(info.st_mode):
                raise GateError("tool-untrusted")
        elif not stat.S_ISDIR(info.st_mode):
            raise GateError("tool-untrusted")
        if item.parent == item:
            break
        item = item.parent
    return path


def administration_tool(path):
    """Resolve only an already-present tool through root-trusted link ancestors, with no install or fallback."""
    path = Path(path)
    if not path.exists():
        raise GateError("tool-unavailable", True)
    # A fixed executable symlink may resolve to another trusted root-owned file; directory trees do not follow links.
    link = path.lstat()
    if link.st_uid != 0 or link.st_mode & 0o022 and not stat.S_ISLNK(link.st_mode):
        raise GateError("tool-untrusted", True)
    root_trusted(path.parent, True)
    current = path
    for _link in range(16):
        root_trusted(current.parent, True)
        info = current.lstat()
        if info.st_uid != 0:
            raise GateError("tool-untrusted", True)
        if not stat.S_ISLNK(info.st_mode):
            break
        target = Path(os.readlink(current))
        current = target if target.is_absolute() else current.parent / target
    else:
        raise GateError("tool-untrusted", True)
    actual = current.resolve(strict=True)
    root_trusted(actual)
    if not os.access(actual, os.X_OK):
        raise GateError("tool-untrusted", True)
    return actual


def command(argv, root, deadline, input_bytes=None, privileged=False):
    """Drain separate bounded pipes under one deadline; arbitrary output stays private and root cleanup is never guessed."""
    if time.monotonic() >= deadline:
        return {"exit_code": None, "failure": "command-timeout", "output": b""}
    result = {"exit_code": None, "failure": "execution-unverified", "output": b""}
    output = bytearray()
    failure = None
    total = 0
    child = None
    selection = selectors.DefaultSelector()
    writing = memoryview(input_bytes or b"")
    try:
        environment = {"PATH": "/usr/sbin:/usr/bin:/sbin:/bin", "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"}
        if time.monotonic() >= deadline:
            result.update(failure="command-timeout")
            return result
        child = subprocess.Popen(argv, cwd=root, env=environment,
            stdin=subprocess.PIPE if input_bytes is not None else subprocess.DEVNULL,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, close_fds=True)
        for stream in (child.stdout, child.stderr):
            os.set_blocking(stream.fileno(), False)
            selection.register(stream, selectors.EVENT_READ)
        if child.stdin is not None:
            os.set_blocking(child.stdin.fileno(), False)
            selection.register(child.stdin, selectors.EVENT_WRITE)
        while selection.get_map() or child.poll() is None:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                failure = failure or "command-timeout"
                break
            for key, _events in selection.select(min(remaining, 0.1)):
                stream = key.fileobj
                if stream is child.stdin:
                    if not writing:
                        selection.unregister(stream)
                        stream.close()
                        continue
                    try:
                        count = os.write(stream.fileno(), writing[:8192])
                        writing = writing[count:]
                    except BlockingIOError:
                        continue
                    except BrokenPipeError:
                        selection.unregister(stream)
                        stream.close()
                        failure = failure or "execution-unverified"
                else:
                    try:
                        data = os.read(stream.fileno(), 8192)
                    except BlockingIOError:
                        continue
                    if not data:
                        selection.unregister(stream)
                    else:
                        total += len(data)
                        if total > MAX_CAPTURE:
                            failure = failure or "output-bound"
                        elif stream is child.stdout and failure is None:
                            output.extend(data)
        if child.poll() is None and not privileged:
            child.kill()
            try:
                child.wait(timeout=1)
            except subprocess.TimeoutExpired:
                failure = failure or "cleanup-unverified"
        if child.poll() is None:
            failure = failure or "cleanup-unverified"
        code = child.poll()
        if code is not None:
            child.wait(timeout=0)
        result.update(exit_code=code, failure=failure, output=bytes(output) if failure is None else b"")
        return result
    except Exception:
        if child is not None and child.poll() is None and not privileged:
            try:
                child.kill()
                child.wait(timeout=1)
            except Exception:
                pass
        result.update(exit_code=None if child is None else child.poll(), failure="execution-unverified", output=b"")
        return result
    finally:
        cleanup_failed = False
        try:
            selection.close()
        except BaseException:
            cleanup_failed = True
        if child is not None:
            for stream in (child.stdin, child.stdout, child.stderr):
                if stream is not None:
                    try:
                        stream.close()
                    except BaseException:
                        cleanup_failed = True
        if cleanup_failed:
            result.update(failure=result["failure"] or "cleanup-unverified", output=b"")


def stdlib_inventory(paths, deadline):
    """Stream complete root-trusted stdlib roots with entry/depth/byte caps, rejecting unsupported links and special files."""
    if type(paths) is not list or not 2 <= len(paths) <= 8 or len(paths) != len(set(paths)) or any(type(path) is not str or not Path(path).is_absolute() for path in paths):
        raise GateError("tool-untrusted", True)
    roots = []
    for value in paths:
        path = Path(value)
        if not path.exists():
            if path.suffix != ".zip":
                raise GateError("tool-untrusted", True)
            root_trusted(path.parent, True)
            continue
        root_trusted(path, True)
        roots.append(path)
    if len(roots) < 2 or not any(path.name == "lib-dynload" for path in roots):
        raise GateError("tool-untrusted", True)
    rows = []
    count = 0
    total = 0
    def visit(root, directory, level, index):
        """Cap every streamed directory entry before retaining it or recursing into another trusted directory."""
        nonlocal count, total
        if level > 32:
            raise GateError("tool-untrusted", True)
        root_trusted(directory, True)
        with os.scandir(directory) as entries:
            for entry in entries:
                if time.monotonic() >= deadline:
                    raise GateError("command-timeout", True)
                count += 1
                if count > MAX_ENTRIES:
                    raise GateError("tool-untrusted", True)
                path = Path(entry.path)
                info = entry.stat(follow_symlinks=False)
                if info.st_uid != 0 or info.st_mode & 0o022 or stat.S_ISLNK(info.st_mode):
                    raise GateError("tool-untrusted", True)
                relative = path.relative_to(root).as_posix()
                if stat.S_ISDIR(info.st_mode):
                    rows.append([index, relative, "directory", info.st_mode & 0o7777, None])
                    visit(root, path, level + 1, index)
                elif stat.S_ISREG(info.st_mode):
                    total += info.st_size
                    if total > MAX_STDLIB_BYTES:
                        raise GateError("tool-untrusted", True)
                    rows.append([index, relative, "file", info.st_mode & 0o7777, shared.hash_file(path)])
                else:
                    raise GateError("tool-untrusted", True)
    for index, root in enumerate(roots):
        visit(root, root, 0, index)
    if count == 0:
        raise GateError("tool-untrusted", True)
    return {"pin": pin_bytes(shared.canonical_bytes(sorted(rows))), "entries": count}


def capture_tools(root, deadline):
    """Qualify only existing Linux tools and complete trusted distro stdlib; setup-Python is separate ordinary evidence."""
    if time.monotonic() >= deadline:
        raise GateError("command-timeout", True)
    if sys.platform != "linux" or os.uname().machine != "x86_64":
        raise GateError("unsupported-platform", True)
    if os.getuid() == 0 or os.geteuid() != os.getuid() or os.getegid() != os.getgid():
        raise GateError("privilege-unavailable", True)
    python = administration_tool(Path("/usr/bin/python3"))
    resolved = {name: administration_tool(Path(shutil.which(name) or "/nonexistent/" + name)) for name in ("ip", "sudo")}
    observed = command([str(python), "-I", "-S", "-B", "-c", PYTHON_PROBE], root, min(deadline, time.monotonic() + 10))
    if observed["failure"] is not None or observed["exit_code"] != 0:
        raise GateError("tool-unavailable", True)
    observation = strict_json(observed["output"])
    closed(observation, ("version", "paths"))
    if type(observation["version"]) is not str or not re.fullmatch(r"3\.[0-9]{1,2}\.[0-9]{1,3}", observation["version"]) or tuple(map(int, observation["version"].split("."))) < (3, 11, 0):
        raise GateError("tool-untrusted", True)
    inventory = stdlib_inventory(observation["paths"], deadline)
    if time.monotonic() >= deadline:
        raise GateError("command-timeout", True)
    tools = {"ordinary": shared.tool_versions(root, max(0.1, min(10, deadline - time.monotonic()))),
        "privileged_python": {"pin": shared.hash_file(python), "version": observation["version"], "root_trust": True,
            "stdlib_pin": inventory["pin"], "stdlib_entries": inventory["entries"]}}
    for name, executable in resolved.items():
        observed = command([str(executable), "-V" if name == "ip" else "--version"], root, min(deadline, time.monotonic() + 10))
        pattern = rb"ip utility, iproute2-([0-9]+(?:\.[0-9]+){1,2}(?:-[A-Za-z0-9.]+)?)(?:, [A-Za-z0-9., -]+)?" if name == "ip" else rb"Sudo version ([0-9]+(?:\.[0-9]+){1,2}(?:p[0-9]+)?)"
        line = observed["output"].split(b"\n", 1)[0]
        match = re.fullmatch(pattern, line)
        if observed["failure"] is not None or observed["exit_code"] != 0 or match is None:
            raise GateError("tool-unavailable", True)
        tools[name] = {"pin": shared.hash_file(executable), "version": match[1].decode("ascii"), "root_trust": True}
    if any(tools["ordinary"].get(name) is None for name in ("cargo", "rustc", "rust_host")):
        raise GateError("tool-unavailable", True)
    if time.monotonic() >= deadline:
        raise GateError("command-timeout", True)
    return tools, {"python": python, **resolved}


def capture_identity(root, forge, timeout):
    """Supplement unchanged shared commit-object and checkout inputs with this slice's three new sources."""
    value = shared.capture_identity(root, forge, timeout)
    value["inputs"].update({name: shared.hash_file(root / name) for name in EXTRA_INPUTS})
    return value


def read_native(path, exit_code, expected_release, expected_sources, declared):
    """Reconcile all closed native rows, actual exits and the preserved unchanged shared client receipt without root imports."""
    raw = read_bytes(path)
    value = strict_json(raw)
    closed(value, ("schema_version", "scope", "truth_state", "acceptance_eligible", "status", "failure", "cleanup_failure",
        "platform", "pins", "qualification", "probes", "client", "observed_runtime", "cleanup", "attempted_egress"))
    status = value["status"]
    exits = {"passed": 0, "failed": 1, "incomplete": 2}
    if status not in exits or type(exit_code) is not int or exit_code != exits[status]:
        raise GateError("execution-unverified")
    if value["schema_version"] != NATIVE_SCHEMA or value["scope"] != SCOPE or value["truth_state"] != "synthetic-development" or value["acceptance_eligible"] is not False:
        raise GateError("execution-unverified")
    closed(value["attempted_egress"], ("state", "count"))
    if value["attempted_egress"] != {"state": "unmeasured", "count": None}:
        raise GateError("execution-unverified")
    if value["failure"] is not None and value["failure"] not in FAILURES or value["cleanup_failure"] not in (None, "cleanup-unverified", "forced-cleanup"):
        raise GateError("execution-unverified")
    if status == "passed" and (value["failure"] is not None or value["cleanup_failure"] is not None) or status != "passed" and value["failure"] is None:
        raise GateError("execution-unverified")
    platform = value["platform"]
    if platform is not None:
        closed(platform, ("os", "machine", "kernel_release", "python_version"))
        if platform["os"] != "Linux" or platform["machine"] != "x86_64" or any(type(platform[key]) is not str or not re.fullmatch(r"[A-Za-z0-9._+-]{1,96}", platform[key]) for key in ("kernel_release", "python_version")):
            raise GateError("execution-unverified")
    pins = value["pins"]
    closed(pins, ("provided_release", "protected_release", "client_inputs", "protected_client_inputs"))
    if pin(pins["provided_release"]) != expected_release or pins["client_inputs"] != expected_sources:
        raise GateError("source-changed")
    closed(pins["client_inputs"], SOURCE_KEYS)
    for row in pins["client_inputs"].values():
        pin(row)
    if pins["protected_release"] is not None and pin(pins["protected_release"]) != expected_release:
        raise GateError("source-changed")
    if pins["protected_client_inputs"] is not None:
        closed(pins["protected_client_inputs"], SOURCE_KEYS)
        for row in pins["protected_client_inputs"].values():
            pin(row)
        if pins["protected_client_inputs"] != expected_sources:
            raise GateError("source-changed")
    qualification = value["qualification"]
    closed(qualification, QUALIFICATION)
    if any(type(flag) is not bool for flag in qualification.values()):
        raise GateError("execution-unverified")
    rows = value["probes"]
    if type(rows) is not list or len(rows) != 12:
        raise GateError("execution-unverified")
    for row, expected in zip(rows, PROBE_ORDER):
        closed(row, ("phase", "family", "role", "status", "outcome", "errno", "challenge_verified"))
        if (row["phase"], row["family"], row["role"]) != expected or row["status"] not in ("not-run", "passed", "failed", "incomplete") or row["outcome"] not in ("not-run", "connected", "denied", "unverified") or type(row["challenge_verified"]) is not bool:
            raise GateError("execution-unverified")
        if row["errno"] is not None and (type(row["errno"]) is not int or row["errno"] != 101):
            raise GateError("execution-unverified")
        if row["status"] == "not-run" and (row["outcome"], row["errno"], row["challenge_verified"]) != ("not-run", None, False):
            raise GateError("execution-unverified")
        denial = row["role"] == "dut-external"
        if row["status"] == "passed" and (row["outcome"], row["errno"], row["challenge_verified"]) != (("denied", 101, False) if denial else ("connected", None, True)):
            raise GateError("probe-denial-unverified" if denial else "probe-calibration-failed")
    client = value["client"]
    closed(client, ("status", "exit_code", "receipt", "receipt_pin"))
    if client["status"] not in ("not-run", "passed", "failed"):
        raise GateError("client-receipt-invalid")
    if client["exit_code"] is not None:
        integer(client["exit_code"], 2147483647, -2147483648)
    if client["status"] == "passed":
        if type(client["exit_code"]) is not int or client["exit_code"] != 0:
            raise GateError("client-receipt-invalid")
        client_path = Path(path).parent / CLIENT_OUTPUT
        client_raw = read_bytes(client_path)
        actual = shared.read_client_receipt(client_path, declared)
        if client_raw != read_bytes(client_path) or client_raw != shared.canonical_bytes(actual) or pin(client["receipt_pin"]) != pin_bytes(client_raw) or actual != client["receipt"]:
            raise GateError("client-receipt-invalid")
    elif client["receipt"] is not None or client["receipt_pin"] is not None or client["status"] == "not-run" and client["exit_code"] is not None:
        raise GateError("client-receipt-invalid")
    observed = value["observed_runtime"]
    closed(observed, ("instances", "namespace_checked", "privilege_checked", "scan_complete", "scope"))
    instances = integer(observed["instances"], 128)
    integer(observed["namespace_checked"], instances)
    integer(observed["privilege_checked"], instances)
    if type(observed["scan_complete"]) is not bool or observed["scope"] != "observed-process-instances-only":
        raise GateError("execution-unverified")
    cleanup = value["cleanup"]
    closed(cleanup, ("state", *OWNED, "forced"))
    if cleanup["state"] not in ("verified-not-created", "verified", "unverified") or any(type(cleanup[key]) is not bool for key in (*OWNED, "forced")):
        raise GateError("cleanup-unverified")
    if cleanup["state"] == "verified" and not all(cleanup[key] for key in OWNED):
        raise GateError("cleanup-unverified")
    if cleanup["forced"] and (status != "failed" or cleanup["state"] == "verified" and value["cleanup_failure"] != "forced-cleanup") or cleanup["state"] == "unverified" and (status != "failed" or value["cleanup_failure"] != "cleanup-unverified"):
        raise GateError("cleanup-unverified")
    if cleanup["state"] == "verified-not-created":
        if status != "incomplete" or cleanup["forced"] or any(cleanup[key] for key in OWNED) or any(qualification.values()) or instances != 0 or observed["namespace_checked"] != 0 or observed["privilege_checked"] != 0 or observed["scan_complete"] or any(row["status"] != "not-run" for row in rows) or client["status"] != "not-run" or pins["protected_release"] is not None or pins["protected_client_inputs"] is not None or value["cleanup_failure"] is not None:
            raise GateError("cleanup-unverified")
    if status == "incomplete" and (cleanup["state"] not in ("verified", "verified-not-created") or cleanup["forced"]):
        raise GateError("cleanup-unverified")
    if status == "passed" and (platform is None or not all(qualification.values()) or any(row["status"] != "passed" for row in rows) or client["status"] != "passed" or pins["protected_release"] is None or pins["protected_client_inputs"] is None or instances == 0 or observed["namespace_checked"] != instances or observed["privilege_checked"] != instances or not observed["scan_complete"] or cleanup["state"] != "verified" or cleanup["forced"]):
        raise GateError("execution-unverified")
    return value, pin_bytes(raw)


def native_run(root, forge, identity, paths, deadline):
    """Dispatch only the verified fixed supervisor, preserve raw private receipts and reject unknown privileged cleanup."""
    row = {"status": "failed", "exit_code": None, "failure": "execution-unverified", "receipt": None, "receipt_pin": None}
    if time.monotonic() >= deadline:
        row["failure"] = "command-timeout"
        return row
    with tempfile.TemporaryDirectory(prefix="forge-os-denial-") as private:
        os.chmod(private, 0o700)
        plan = {"schema": "forge.os-denial-private-plan/1", "root": str(root), "forge": str(forge),
            "output_dir": private, "ip": str(paths["ip"]), "python": str(paths["python"]),
            "target_uid": os.getuid(), "target_gid": os.getgid(), "deadline_monotonic_ns": int((deadline - 5) * 1000000000),
            "source_pins": {name: identity["inputs"][name] for name in SOURCE_KEYS}, "release_pin": identity["provided_release_binary"]}
        raw = shared.canonical_bytes(plan)
        if len(raw) > 65536:
            raise GateError("source-invalid")
        helper = root / EXTRA_INPUTS[0]
        # The protected native helper independently revalidates its plan/admin paths before namespace creation.
        expected = identity["inputs"][EXTRA_INPUTS[0]]["sha256"]
        if time.monotonic() >= deadline:
            row["failure"] = "command-timeout"
            return row
        completed = command([str(paths["sudo"]), "-n", "--", str(paths["python"]), "-I", "-S", "-B", "-c", BOOTSTRAP, str(helper), expected], root, deadline, raw, True)
        row["exit_code"] = completed["exit_code"]
        if completed["failure"] is not None:
            row["failure"] = completed["failure"]
            return row
        try:
            declared = [name for _method, _path, name in shared.contract_routes(root)]
            value, retained = read_native(Path(private) / NATIVE_OUTPUT, completed["exit_code"],
                plan["release_pin"], plan["source_pins"], declared)
            row.update(status=value["status"], failure=value["failure"], receipt=value, receipt_pin=retained)
        except Exception:
            row["failure"] = "execution-unverified"
        return row


def verify(root, forge, output_dir, expected_commit=None, build_outcome="unrecorded", *, event="local",
           checkout_kind="local", requested_head=None, requested_base=None):
    """Bind build/context/tool/source stability to one closed native experiment while keeping all wider acceptance gates open."""
    if build_outcome not in shared.BUILD_OUTCOMES:
        raise ValueError("build outcome")
    destination = Path(output_dir)
    if destination.exists() and (not destination.is_dir() or destination.is_symlink() or any(destination.iterdir())):
        raise ValueError("fresh output")
    destination.mkdir(parents=True, exist_ok=True, mode=0o700)
    os.chmod(destination, 0o700)
    deadline = time.monotonic() + 600
    receipt = {"schema_version": SCHEMA, "scope": SCOPE, "truth_state": "synthetic-development", "acceptance_eligible": False,
        "status": "failed", "failure": "execution-unverified", "identity": None, "checkout": None, "tools": None,
        "build": {"outcome": build_outcome, "profile": "release", "features": "default", "locked": True, "offline": True, "binding": "workflow-step-assertion"},
        "input_stability": "unverified", "tool_stability": "unverified",
        "producer": {"status": "not-run", "exit_code": None, "failure": None, "receipt": None, "receipt_pin": None},
        "cleanup": {"state": "verified-not-created", "forced": False},
        "attempted_egress": {"state": "unmeasured", "count": None}, "pending_gates": list(PENDING)}
    dispatched = False
    before = None
    tools = None
    paths = None
    try:
        before = capture_identity(root, forge, 30)
        receipt["identity"] = before
        receipt["checkout"] = shared.checkout_binding(before, expected_commit, event=event,
            checkout_kind=checkout_kind, requested_head=requested_head, requested_base=requested_base)
        if not before["tracked_source_clean"]:
            raise GateError("tracked-source-unclean")
        if sys.platform != "linux":
            raise GateError("unsupported-platform", True)
        if build_outcome != "success":
            raise GateError("build-not-qualified", build_outcome not in ("failure", "cancelled"))
        tools, paths = capture_tools(root, deadline)
        receipt["tools"] = tools
        dispatched = True
        receipt["cleanup"] = {"state": "unverified", "forced": False}
        receipt["producer"] = {"status": "failed", "exit_code": None, "failure": "execution-unverified", "receipt": None, "receipt_pin": None}
        receipt["producer"] = native_run(root, forge, before, paths, deadline)
        producer = receipt["producer"]
        receipt["status"], receipt["failure"] = producer["status"], producer["failure"]
        if producer["receipt"] is not None:
            cleanup = producer["receipt"]["cleanup"]
            receipt["cleanup"] = {"state": cleanup["state"], "forced": cleanup["forced"]}
        elif producer["status"] == "passed":
            raise GateError("execution-unverified")
    except GateError as error:
        receipt.update(status="incomplete" if error.incomplete and not dispatched else "failed", failure=error.code)
    except Exception:
        receipt.update(status="failed", failure="verification-input-invalid")
    try:
        after = capture_identity(root, forge, max(0.1, min(30, deadline - time.monotonic())))
        receipt["input_stability"] = "unverified" if before is None else "unchanged" if before == after else "changed"
        if tools is not None:
            after_tools, after_paths = capture_tools(root, deadline)
            receipt["tool_stability"] = "unchanged" if tools == after_tools and paths == after_paths else "changed"
        if receipt["input_stability"] == "changed":
            receipt.update(status="failed", failure="source-changed")
        elif tools is not None and receipt["tool_stability"] != "unchanged":
            receipt.update(status="failed", failure="tool-identity-changed")
        if receipt["status"] == "passed" and (not dispatched or receipt["producer"]["status"] != "passed" or receipt["cleanup"] != {"state": "verified", "forced": False} or receipt["input_stability"] != "unchanged" or receipt["tool_stability"] != "unchanged"):
            receipt.update(status="failed", failure="execution-unverified")
    except Exception:
        receipt.update(status="failed", failure="verification-input-invalid")
    if len(shared.canonical_bytes(receipt)) > MAX_CAPTURE:
        raise ValueError("outer bound")
    shared.atomic_receipt(destination / OUTPUT, receipt)
    return receipt


def main():
    """Expose only fixed producer/context choices; raw failures are private and publication errors exit nonzero."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--forge", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--expected-commit")
    parser.add_argument("--requested-head")
    parser.add_argument("--requested-base")
    parser.add_argument("--event", choices=("local", "push", "pull_request", "workflow_dispatch"), default="local")
    parser.add_argument("--checkout-kind", choices=("local", "head", "pull-request-merge"), default="local")
    parser.add_argument("--build-outcome", choices=shared.BUILD_OUTCOMES, default="unrecorded")
    args = parser.parse_args()
    try:
        value = verify(Path(__file__).resolve().parents[1], args.forge.absolute(), args.output_dir,
            args.expected_commit, args.build_outcome, event=args.event, checkout_kind=args.checkout_kind,
            requested_head=args.requested_head, requested_base=args.requested_base)
    except Exception:
        print("OS-denial prerequisite receipt could not be published.", file=sys.stderr)
        return 1
    print("OS-denial prerequisite: " + value["status"] + "; wider gates remain open.")
    return {"passed": 0, "failed": 1, "incomplete": 2}[value["status"]]


if __name__ == "__main__":
    sys.exit(main())
