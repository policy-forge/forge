#!/usr/bin/env python3
"""Bounded Linux headless IP-denial prerequisite; native use requires qualification.

Only the fixed stdin plan is accepted. Privileged code imports stdlib only;
source copies, network resources and child identities remain private. This
module does not install tools, observe every syscall, or award acceptance.
"""
import ctypes
import errno
import hashlib
import hmac
import json
import os
from pathlib import Path
import platform
import re
import select
import shutil
import signal
import socket
import stat
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True
MAX_PLAN = 65536
MAX_CAPTURE = 262144
MAX_FILE = 1073741824
MAX_PROC = 20000
MAX_INSTANCES = 128
MAX_FDS = 256
SOURCES = ("scripts/test_workspace_client.py", "scripts/workspace_client.py",
           "scripts/verify_workspace.py", "docs/api/forge-workspace-v1.openapi.yaml")
QUALIFICATIONS = ("facilities_qualified", "three_owned_namespaces", "dut_lo_only_before",
                  "dut_lo_only_after", "canary_dual_family", "client_preexec_fence",
                  "protected_execution")
CHECKS = frozenset(("upload_requires_confirmation", "idempotent_commit_replays_exact_bytes",
                   "conversion_succeeds", "explicit_no_relationship_is_preserved",
                   "omitted_scope_decisions_remain_under_review", "explicit_scope_decision_validates",
                   "committed_scope_analysis_succeeds", "export_matches_committed_redacted_bytes",
                   "read_only_summary_reconciles_resources", "read_only_mutation_is_rejected",
                   "clean_shutdown_writable", "clean_shutdown_read_only",
                   "metadata_bundle_preview_preserves_complete_denominator",
                   "metadata_bundle_fingerprints_match_exact_bytes",
                   "registered_bundle_comparison_reconciles_expected_current",
                   "metadata_bundle_queries_preserve_workspace_files"))
FAILURES = frozenset(("unsupported-platform", "tool-unavailable", "tool-untrusted",
                      "facility-unavailable", "privilege-unavailable", "source-invalid",
                      "source-changed", "privilege-fence", "namespace-unverified",
                      "probe-calibration-failed", "probe-denial-unverified", "loopback-unverified",
                      "client-failed", "client-receipt-invalid", "command-timeout", "output-bound",
                      "cleanup-unverified", "execution-unverified", "publication-failed"))


class Failure(Exception):
    """Carry a fixed public code and a qualified missing-prerequisite flag only."""

    def __init__(self, code, unavailable=False):
        """Reject arbitrary diagnostic strings before storing a failure."""
        if code not in FAILURES or type(unavailable) is not bool:
            raise ValueError("closed failure")
        self.code, self.unavailable = code, unavailable
        super().__init__(code)


class SigAction(ctypes.Structure):
    """Linux x86_64 glibc sigaction layout; actual target ABI remains a qualification gate."""

    _fields_ = [("handler", ctypes.c_void_p), ("mask", ctypes.c_ulong * 16),
                ("flags", ctypes.c_int), ("restorer", ctypes.c_void_p)]


def canonical(value):
    """Match the shared publisher's exact canonical redacted JSON representation."""
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True,
                       allow_nan=False) + "\n").encode("ascii")


def pairs(items):
    """Reject duplicate decoded object members at every depth."""
    result = {}
    for key, value in items:
        if key in result:
            raise Failure("source-invalid")
        result[key] = value
    return result


def nonfinite(_):
    """Reject nonfinite JSON constants without retaining their spelling."""
    raise Failure("source-invalid")


def decode(raw, limit=MAX_CAPTURE):
    """Apply a byte cap, strict decoder and bounded post-decode depth inventory."""
    if not isinstance(raw, bytes) or not raw or len(raw) > limit:
        raise Failure("source-invalid")
    try:
        value = json.loads(raw, object_pairs_hook=pairs, parse_constant=nonfinite)
        todo = [(value, 0)]
        while todo:
            item, depth = todo.pop()
            if depth > 16:
                raise Failure("source-invalid")
            if isinstance(item, dict):
                todo.extend((child, depth + 1) for child in item.values())
            elif isinstance(item, list):
                todo.extend((child, depth + 1) for child in item)
        return value
    except (ValueError, UnicodeError, RecursionError):
        raise Failure("source-invalid") from None


def exact_int(value, low, high):
    """Require an exact integer within the closed range, excluding booleans."""
    return type(value) is int and low <= value <= high


def valid_pin(value):
    """Validate a closed bounded byte/hash identity without accepting paths."""
    return (isinstance(value, dict) and set(value) == {"bytes", "sha256"}
            and exact_int(value["bytes"], 0, MAX_FILE)
            and isinstance(value["sha256"], str)
            and re.fullmatch(r"[0-9a-f]{64}", value["sha256"]) is not None)


def parse_plan(raw, now_ns):
    """Accept only canonical fixed inputs and an existing absolute Linux deadline."""
    plan = decode(raw, MAX_PLAN)
    keys = {"schema", "root", "forge", "output_dir", "ip", "python", "target_uid",
            "target_gid", "deadline_monotonic_ns", "source_pins", "release_pin"}
    if not isinstance(plan, dict) or set(plan) != keys or canonical(plan) != raw:
        raise Failure("source-invalid")
    if plan["schema"] != "forge.os-denial-private-plan/1":
        raise Failure("source-invalid")
    for name in ("root", "forge", "output_dir", "ip", "python"):
        value = plan[name]
        if (not isinstance(value, str) or len(value) > 4096 or not value.startswith("/")
                or any(ord(c) < 32 for c in value) or str(Path(value)) != value):
            raise Failure("source-invalid")
    if (not exact_int(plan["target_uid"], 1, 2147483647)
            or not exact_int(plan["target_gid"], 0, 2147483647)
            or not exact_int(plan["deadline_monotonic_ns"], now_ns + 1, now_ns + 600000000000)):
        raise Failure("source-invalid")
    if (not isinstance(plan["source_pins"], dict) or set(plan["source_pins"]) != set(SOURCES)
            or any(not valid_pin(pin) for pin in plan["source_pins"].values())
            or not valid_pin(plan["release_pin"])):
        raise Failure("source-invalid")
    return plan


def remaining(end, now=time.monotonic):
    """Return the remaining absolute budget or a fixed timeout failure."""
    left = end - now()
    if left <= 0:
        raise Failure("command-timeout")
    return left


def file_pin(path, end, destination=None):
    """Hash a stable no-follow regular file, optionally copying the same read bytes."""
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK)
    try:
        first = os.fstat(fd)
        if not stat.S_ISREG(first.st_mode) or first.st_size > MAX_FILE:
            raise Failure("source-invalid")
        digest, length = hashlib.sha256(), 0
        while True:
            remaining(end)
            block = os.read(fd, 1048576)
            if not block:
                break
            length += len(block)
            if length > MAX_FILE:
                raise Failure("source-invalid")
            digest.update(block)
            if destination is not None:
                destination.write(block)
        last = os.fstat(fd)
        current = os.stat(path, follow_symlinks=False)
        fields = ("st_dev", "st_ino", "st_size", "st_mtime_ns", "st_ctime_ns")
        if any(getattr(first, k) != getattr(last, k) or getattr(first, k) != getattr(current, k)
               for k in fields):
            raise Failure("source-changed")
        remaining(end)
        return {"bytes": length, "sha256": digest.hexdigest()}
    finally:
        os.close(fd)


def trusted_path(path, directory=False):
    """Require a canonical root-owned target and non-writable root-owned ancestors."""
    path = Path(path)
    if str(path.resolve(strict=True)) != str(path):
        raise Failure("tool-untrusted")
    for part in (path, *path.parents):
        st = part.stat(follow_symlinks=False)
        is_dir = part != path or directory
        if (st.st_uid != 0 or st.st_mode & 0o022
                or not (stat.S_ISDIR(st.st_mode) if is_dir else stat.S_ISREG(st.st_mode))):
            raise Failure("tool-untrusted")
    return path


def trusted_stdlib(end):
    """Revalidate complete isolated stdlib roots with streamed depth/entry/byte bounds."""
    count, total, directories = 0, 0, set()
    roots = []
    for value in sys.path:
        if not value or not os.path.isabs(value):
            raise Failure("tool-untrusted")
        path = Path(value)
        if path.suffix == ".zip" and not path.exists():
            continue
        if not path.is_dir():
            raise Failure("tool-untrusted")
        trusted_path(str(path), True)
        roots.append(path)
    if not roots or not any((p / "os.py").is_file() for p in roots) or not any(p.name == "lib-dynload" for p in roots):
        raise Failure("tool-untrusted")
    todo = [(p, 0) for p in roots]
    while todo:
        parent, depth = todo.pop()
        if parent in directories:
            continue
        directories.add(parent)
        if depth > 32:
            raise Failure("tool-untrusted")
        with os.scandir(parent) as entries:
            for entry in entries:
                remaining(end)
                count += 1
                if count > MAX_PROC:
                    raise Failure("tool-untrusted")
                s = entry.stat(follow_symlinks=False)
                if s.st_uid != 0 or s.st_mode & 0o022:
                    raise Failure("tool-untrusted")
                if stat.S_ISDIR(s.st_mode):
                    todo.append((Path(entry.path), depth + 1))
                elif stat.S_ISREG(s.st_mode):
                    total += s.st_size
                    if total > 268435456:
                        raise Failure("tool-untrusted")
                    file_pin(entry.path, end)
                else:
                    raise Failure("tool-untrusted")
    return count


def qualify_libc(lib):
    """Require the explicit GNU libc symbol and a bounded safe version before ABI-dependent queries."""
    try:
        version = lib.gnu_get_libc_version
        version.argtypes, version.restype = [], ctypes.c_char_p
        raw = version()
    except BaseException:
        raise Failure("facility-unavailable", True) from None
    if (not isinstance(raw, bytes) or not 1 <= len(raw) <= 16
            or not re.fullmatch(rb"[0-9]{1,2}\.[0-9]{1,3}", raw)):
        raise Failure("facility-unavailable", True)
    return raw


def start_identity(pid):
    """Read a direct child's start value without assuming a zombie retains namespace/FD entries."""
    raw = read_proc(Path("/proc") / str(pid) / "stat", 8192)
    closing = raw.rfind(b")")
    fields = raw[closing + 2:].split()
    if closing < 0 or len(fields) < 20:
        raise Failure("execution-unverified")
    return int(fields[19])


def read_proc(path, limit=65536):
    """Bound one proc observation; unreadable or truncated rows receive no proof."""
    with open(path, "rb") as stream:
        raw = stream.read(limit + 1)
    if len(raw) > limit:
        raise Failure("execution-unverified")
    return raw


def bounded_fd_names(path):
    """Stream and validate at most MAX_FDS numeric descriptor names before later FD operations."""
    names = []
    with os.scandir(path) as rows:
        for count, row in enumerate(rows, 1):
            if count > MAX_FDS or not re.fullmatch(r"[0-9]{1,10}", row.name):
                raise Failure("execution-unverified")
            number = int(row.name)
            if not exact_int(number, 0, 2147483647) or str(number) != row.name or number in names:
                raise Failure("execution-unverified")
            names.append(number)
    return names


def process_record(pid, with_fds=False):
    """Read one process instance and actual credentials, namespace and descriptor links."""
    base = Path("/proc") / str(pid)
    raw = read_proc(base / "stat", 8192)
    closing = raw.rfind(b")")
    fields = raw[closing + 2:].split()
    if closing < 0 or len(fields) < 20:
        raise Failure("execution-unverified")
    parent, start, state = int(fields[1]), int(fields[19]), fields[0]
    status = {}
    for line in read_proc(base / "status").decode("ascii").splitlines():
        key, sep, value = line.partition(":")
        if sep:
            status[key] = value.strip()
    net = (base / "ns/net").stat()
    fds = {}
    if with_fds:
        names = bounded_fd_names(base / "fd")
        fds = {number: os.readlink(base / "fd" / str(number)) for number in names}
    if start_identity(pid) != start:
        raise Failure("execution-unverified")
    return {"pid": pid, "ppid": parent, "start": start, "state": state,
            "status": status, "net": (net.st_dev, net.st_ino), "fds": fds}


def privilege_proof(record, uid, gid):
    """Prove all four IDs, no groups/capabilities, NNP and single-thread setup state."""
    s = record["status"]
    try:
        return (list(map(int, s["Uid"].split())) == [uid] * 4
                and list(map(int, s["Gid"].split())) == [gid] * 4
                and s["Groups"] == "" and s["NoNewPrivs"] == "1"
                and all(int(s[k], 16) == 0 for k in ("CapInh", "CapPrm", "CapEff", "CapBnd", "CapAmb")))
    except (KeyError, ValueError):
        return False


def blocked_proof(record, child, net, uid, gid, allowed):
    """Independently validate the blocked instance and exact allowed FD relation."""
    if (record["pid"] != child.pid or record["start"] != child.start
            or record["net"] != net or record["status"].get("Threads") != "1"
            or not privilege_proof(record, uid, gid) or record["fds"] != allowed):
        raise Failure("privilege-fence")


def client_receipt(raw, declared):
    """Validate the unchanged closed client/2 before ordinary-wrapper revalidation."""
    if len(declared) != 39:
        raise Failure("client-receipt-invalid")
    v = decode(raw)
    keys = {"schema_version", "status", "checks", "check_count", "declared_operation_count",
            "observed_operations", "unobserved_operations", "operation_outcomes"}
    if (not isinstance(v, dict) or set(v) != keys or canonical(v) != raw
            or v["schema_version"] != "forge.workspace-client-verification/2" or v["status"] != "passed"):
        raise Failure("client-receipt-invalid")
    checks, observed, missing = v["checks"], v["observed_operations"], v["unobserved_operations"]
    if (not isinstance(checks, list) or any(type(x) is not str for x in checks)
            or len(checks) != 16 or set(checks) != CHECKS or type(v["check_count"]) is not int
            or v["check_count"] != 16 or type(v["declared_operation_count"]) is not int
            or v["declared_operation_count"] != len(declared)):
        raise Failure("client-receipt-invalid")
    for items in (observed, missing):
        if (not isinstance(items, list) or any(type(x) is not str for x in items)
                or items != sorted(set(items))):
            raise Failure("client-receipt-invalid")
    outcomes = v["operation_outcomes"]
    if (set(observed) & set(missing) or set(observed) | set(missing) != set(declared)
            or not isinstance(outcomes, dict) or set(outcomes) != set(observed)):
        raise Failure("client-receipt-invalid")
    total = 0
    for counts in outcomes.values():
        if (not isinstance(counts, dict) or set(counts) != {"succeeded", "rejected", "transport_failed"}
                or any(not exact_int(x, 0, 10000) for x in counts.values()) or sum(counts.values()) == 0):
            raise Failure("client-receipt-invalid")
        total += sum(counts.values())
    if not 0 < total <= 10000:
        raise Failure("client-receipt-invalid")
    return v


def initial_receipt(plan):
    """Create explicit not-run rows and false observations without native credit."""
    rows = []
    for phase in ("before", "after"):
        for family in ("ipv4", "ipv6"):
            for role in ("calibration-external", "dut-external", "dut-loopback"):
                rows.append({"phase": phase, "family": family, "role": role, "status": "not-run",
                             "outcome": "not-run", "errno": None, "challenge_verified": False})
    return {"schema_version": "forge.packaged-runtime-os-denial/1",
            "scope": "linux-headless-ip-network-denial-prerequisite", "truth_state": "synthetic-development",
            "acceptance_eligible": False, "status": "failed", "failure": None, "cleanup_failure": None,
            "platform": None, "pins": {"provided_release": plan["release_pin"], "protected_release": None,
                                      "client_inputs": plan["source_pins"], "protected_client_inputs": None},
            "qualification": dict.fromkeys(QUALIFICATIONS, False), "probes": rows,
            "client": {"status": "not-run", "exit_code": None, "receipt": None, "receipt_pin": None},
            "observed_runtime": {"instances": 0, "namespace_checked": 0, "privilege_checked": 0,
                                 "scan_complete": False, "scope": "observed-process-instances-only"},
            "cleanup": {"state": "unverified", "owned_processes_empty": False,
                        "owned_references_closed": False, "owned_names_absent": False,
                        "virtual_links_absent": False, "forced": False},
            "attempted_egress": {"state": "unmeasured", "count": None}}


def read_plan(stream):
    """Read bounded stdin to EOF within the nonrenewing 30-second startup cap."""
    end = time.monotonic() + 30
    raw = bytearray()
    fd = stream.fileno()
    os.set_blocking(fd, False)
    while True:
        if not select.select([fd], [], [], remaining(end))[0]:
            raise Failure("command-timeout")
        block = os.read(fd, min(8192, MAX_PLAN + 1 - len(raw)))
        if not block:
            return bytes(raw)
        raw.extend(block)
        if len(raw) > MAX_PLAN:
            raise Failure("source-invalid")


def remove_private(path, end):
    """Remove only the retained private tree with no-follow dirfds, depth/entry/deadline bounds."""
    root = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)
    identity = os.fstat(root)
    count = [0]

    def entries(fd, depth):
        """Stream bounded directory rows without following worker-created symbolic links."""
        if depth > 32:
            raise Failure("cleanup-unverified")
        with os.scandir(fd) as rows:
            for row in rows:
                remaining(end)
                count[0] += 1
                if count[0] > MAX_PROC:
                    raise Failure("cleanup-unverified")
                before = row.stat(follow_symlinks=False)
                if stat.S_ISDIR(before.st_mode):
                    child = os.open(row.name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC, dir_fd=fd)
                    try:
                        actual = os.fstat(child)
                        if (actual.st_dev, actual.st_ino) != (before.st_dev, before.st_ino):
                            raise Failure("cleanup-unverified")
                        entries(child, depth + 1)
                    finally:
                        os.close(child)
                    os.rmdir(row.name, dir_fd=fd)
                else:
                    os.unlink(row.name, dir_fd=fd)
    try:
        entries(root, 0)
        current = os.stat(path, follow_symlinks=False)
        if (current.st_dev, current.st_ino) != (identity.st_dev, identity.st_ino):
            raise Failure("cleanup-unverified")
        remaining(end)
        os.rmdir(path)
    finally:
        os.close(root)


class Child:
    """Own one fork or Popen wait status, retained instance identity and pidfd."""

    def __init__(self, pid, start, pidfd, owner=None, output=None, stop=None, rollback=False):
        """Store private identity; no supplied process identifier is accepted."""
        self.pid, self.start, self.pidfd, self.owner = pid, start, pidfd, owner
        self.output, self.stop, self.exit = output, stop, None
        self.captured, self.eof = bytearray(), output is None
        self.rollback = rollback

    def poll(self):
        """Use exactly the reserving Popen or this fork child's specific wait owner."""
        if self.exit is None:
            if self.owner is not None:
                self.exit = self.owner.poll()
            else:
                pid, status = os.waitpid(self.pid, os.WNOHANG)
                if pid:
                    self.exit = os.waitstatus_to_exitcode(status)
        return self.exit

    def drain(self):
        """Bound private capture and require actual EOF separately from process exit."""
        if self.output is None or self.eof:
            return
        try:
            block = os.read(self.output, 65536)
        except BlockingIOError:
            return
        if not block:
            self.eof = True
        else:
            if len(self.captured) + len(block) > MAX_CAPTURE:
                self.captured.clear()
                raise Failure("output-bound")
            self.captured.extend(block)

    def close(self):
        """Release only this retained child's private handles after final accounting."""
        failed = False
        for name in ("output", "stop", "pidfd"):
            fd = getattr(self, name)
            if fd is not None:
                try:
                    if name == "output" and self.owner is not None and self.owner.stdout is not None:
                        self.owner.stdout.close()
                    else:
                        os.close(fd)
                    setattr(self, name, None)
                except BaseException:
                    failed = True
        if failed:
            raise Failure("cleanup-unverified")


class Native:
    """Linux adapters; tests replace native operations and never instantiate this API."""

    def __init__(self, plan):
        """Reserve state before native/resource creation and bind one libc instance."""
        self.plan, self.end = plan, plan["deadline_monotonic_ns"] / 1000000000
        self.work_end = self.end - 10
        self.setup_end, self.in_setup = min(self.work_end, time.monotonic() + 30), True
        self.lib = None
        self.children, self.observed, self.names, self.netfds = [], {}, {}, {}
        self.pending_fds, self.local_reference_failure = set(), False
        self.names_attempted, self.links_attempted = [], []
        self.base, self.package, self.workspace, self.output_fd = None, None, None, None
        self.link_name, self.link_peer = None, None
        self.links = {}
        self.created, self.forced, self.scan_complete = False, False, True
        self.servers, self.ports, self.challenge = {}, {}, os.urandom(32)

    def call(self, operation, *args):
        """Check one declared native return without publishing errno or exception text."""
        if operation(*args) != 0:
            raise Failure("facility-unavailable")

    def prctl(self, option, value=0):
        """Call the explicitly typed Linux prctl adapter with fixed unused zero arguments."""
        self.call(self.lib.prctl, option, value, 0, 0, 0)

    def pipe(self):
        """Reserve both parent pipe descriptors before later setup can fail."""
        pair = os.pipe2(os.O_CLOEXEC)
        self.pending_fds.update(pair)
        return pair

    def close_pending(self, fd):
        """Attempt a reserved parent handle independently and retain any failed ownership."""
        if fd is not None and fd in self.pending_fds:
            try:
                os.close(fd)
                self.pending_fds.remove(fd)
            except BaseException:
                self.local_reference_failure = True

    def qualify(self):
        """Qualify platform, privileged interpreter/stdlib/tools and setup capabilities."""
        if platform.system() != "Linux" or platform.machine() != "x86_64":
            raise Failure("unsupported-platform", True)
        if os.geteuid() != 0 or os.getuid() != 0:
            raise Failure("privilege-unavailable", True)
        if not (sys.flags.isolated and sys.flags.no_site and sys.flags.ignore_environment
                and sys.dont_write_bytecode and sys.version_info >= (3, 11)):
            raise Failure("tool-untrusted")
        if str(Path(sys.executable).resolve()) != self.plan["python"]:
            raise Failure("tool-untrusted")
        trusted_path(self.plan["python"])
        trusted_path(self.plan["ip"])
        trusted_stdlib(self.setup_end)
        for module in (os, json, socket, ctypes, subprocess, tempfile, shutil):
            path = getattr(module, "__file__", None)
            if path and not path.startswith("<"):
                trusted_path(str(Path(path).resolve()))
        if not shutil.rmtree.avoids_symlink_attacks or not hasattr(os, "pidfd_open"):
            raise Failure("facility-unavailable", True)
        status = process_record(os.getpid())["status"]
        if status.get("Threads") != "1":
            raise Failure("privilege-fence")
        self.lib = ctypes.CDLL(None, use_errno=True)
        qualify_libc(self.lib)
        self.lib.prctl.argtypes = [ctypes.c_int, ctypes.c_ulong, ctypes.c_ulong,
                                  ctypes.c_ulong, ctypes.c_ulong]
        self.lib.prctl.restype = ctypes.c_int
        self.lib.setns.argtypes, self.lib.setns.restype = [ctypes.c_int, ctypes.c_int], ctypes.c_int
        self.lib.capset.argtypes, self.lib.capset.restype = [ctypes.c_void_p, ctypes.c_void_p], ctypes.c_int
        self.lib.sigaction.argtypes, self.lib.sigaction.restype = [ctypes.c_int, ctypes.c_void_p, ctypes.c_void_p], ctypes.c_int
        if ctypes.sizeof(SigAction) != 152 or SigAction.flags.offset != 136 or SigAction.restorer.offset != 144:
            raise Failure("facility-unavailable", True)
        self.prctl(36, 1)  # PR_SET_CHILD_SUBREAPER; this private supervisor owns its descendants.
        release = platform.release()
        if not re.fullmatch(r"[A-Za-z0-9._+-]{1,96}", release):
            raise Failure("unsupported-platform", True)
        return {"os": "Linux", "machine": "x86_64", "kernel_release": release,
                "python_version": platform.python_version()}

    def open_output(self):
        """Retain a no-follow output directory identity; worker renames cannot retarget writes."""
        self.output_fd = os.open(self.plan["output_dir"], os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)
        s = os.fstat(self.output_fd)
        if s.st_uid != self.plan["target_uid"] or s.st_mode & 0o077:
            raise Failure("source-invalid")

    def protect(self):
        """Copy the exact planned release/four sources into a root-owned protected package."""
        remaining(self.setup_end)
        self.base = Path(tempfile.mkdtemp(prefix="forge-os-denial-"))
        self.created = True
        self.base.chmod(0o755)
        self.package, self.workspace = self.base / "package", self.base / "worker"
        self.package.mkdir(mode=0o755)
        self.package.chmod(0o755)
        self.workspace.mkdir(mode=0o700)
        os.chown(self.workspace, self.plan["target_uid"], self.plan["target_gid"])
        pins = {}
        for key in (*SOURCES, "forge"):
            target = self.package / key
            target.parent.mkdir(mode=0o755, parents=True, exist_ok=True)
            for parent in (target.parent, *target.parent.parents):
                parent.chmod(0o755)
                if parent == self.package:
                    break
            origin = Path(self.plan["forge"]) if key == "forge" else Path(self.plan["root"]) / key
            expected = self.plan["release_pin"] if key == "forge" else self.plan["source_pins"][key]
            with target.open("xb") as stream:
                pin = file_pin(origin, self.setup_end, stream)
                stream.flush()
                os.fsync(stream.fileno())
            target.chmod(0o555 if key == "forge" else 0o444)
            if pin != expected or file_pin(target, self.setup_end) != expected:
                raise Failure("source-changed")
            pins[key] = pin
        return pins["forge"], {key: pins[key] for key in SOURCES}

    def owner_proof(self):
        """Inspect actual SIGCHLD hidden flags and one-thread sole-owner eligibility."""
        if process_record(os.getpid())["status"].get("Threads") != "1":
            return False
        action = SigAction()
        self.call(self.lib.sigaction, signal.SIGCHLD, None, ctypes.byref(action))
        return action.handler is None and not action.flags & 2  # SA_NOCLDWAIT; SIG_DFL is zero.

    def reserve(self, pid, owner=None, output=None, stop=None, rollback=False):
        """Open a pidfd before release and retain the reserving wait owner even on scan failure."""
        if not exact_int(pid, 1, 2147483647):
            raise Failure("execution-unverified")
        child = Child(pid, None, None, owner, output, stop, rollback)
        self.children.append(child)
        if len(self.children) > MAX_INSTANCES:
            raise Failure("execution-unverified")
        child.pidfd = os.pidfd_open(pid, 0)
        child.start = start_identity(pid)
        return child

    def administrative(self, arguments, end=None, cleanup=False):
        """Run only a fixed ip argv, reserve Popen ownership and discard every output byte."""
        ceiling = self.end if cleanup else self.setup_end if self.in_setup else self.work_end
        end = min(ceiling, end if end is not None else ceiling)
        remaining(end)
        proof = self.owner_proof()
        if not proof:
            raise Failure("execution-unverified")
        remaining(end)
        output_r, output_w = self.pipe()
        child = None
        try:
            os.set_blocking(output_r, False)
            remaining(end)
            proc = subprocess.Popen([self.plan["ip"], *arguments], stdin=subprocess.DEVNULL,
                                    stdout=output_w, stderr=subprocess.STDOUT,
                                    env={"PATH": "/usr/sbin:/usr/bin:/sbin:/bin", "LC_ALL": "C"})
            owned_output, output_r = output_r, None
            self.pending_fds.remove(owned_output)
            child = self.reserve(proc.pid, proc, owned_output, rollback=proof)
            self.close_pending(output_w)
            while child.poll() is None or not child.eof:
                remaining(end)
                child.drain()
                time.sleep(min(.01, remaining(end)))
            if child.exit != 0:
                raise Failure("namespace-unverified")
            remaining(end)
            return bytes(child.captured)
        finally:
            if child is not None:
                child.captured.clear()
            for fd in (output_r, output_w):
                self.close_pending(fd)

    def setup(self):
        """Create exactly three private namespaces and one isolated calibration-canary veth."""
        setup_end = self.setup_end
        remaining(setup_end)
        token = os.urandom(8).hex()
        for role in ("canary", "calibration", "dut"):
            name = "fgd-" + token + "-" + role[:3]
            path = Path("/run/netns") / name
            if os.path.lexists(path):
                raise Failure("namespace-unverified")
            self.names_attempted.append(name)
            self.created = True
            self.administrative(["netns", "add", name], setup_end)
            fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC)
            s = os.fstat(fd)
            self.names[role] = (name, s.st_dev, s.st_ino)
            self.netfds[role] = fd
            self.administrative(["-n", name, "link", "set", "lo", "up"], setup_end)
            self.administrative(["-n", name, "-6", "addr", "replace", "::1/128", "dev", "lo", "nodad"], setup_end)
        self.link_name, self.link_peer = "fd" + token[:10], "fp" + token[:10]
        self.links_attempted = [self.link_name, self.link_peer]
        self.administrative(["-n", self.names["canary"][0], "link", "add", self.link_name,
                             "type", "veth", "peer", "name", self.link_peer], setup_end)
        self.links[("canary", self.link_name)] = self.capture_link("canary", self.link_name, setup_end)
        self.links[("canary", self.link_peer)] = self.capture_link("canary", self.link_peer, setup_end)
        self.links[("calibration", self.link_peer)] = None
        self.administrative(["-n", self.names["canary"][0], "link", "set", self.link_peer,
                             "netns", self.names["calibration"][0]], setup_end)
        peer = self.capture_link("calibration", self.link_peer, setup_end)
        if peer[1] != self.links[("canary", self.link_peer)][1]:
            raise Failure("namespace-unverified")
        self.links[("calibration", self.link_peer)] = peer
        del self.links[("canary", self.link_peer)]
        for role, link, v4, v6 in (("canary", self.link_name, "198.18.0.1/30", "fd00:62::1/64"),
                                    ("calibration", self.link_peer, "198.18.0.2/30", "fd00:62::2/64")):
            name = self.names[role][0]
            self.administrative(["-n", name, "addr", "add", v4, "dev", link], setup_end)
            self.administrative(["-n", name, "-6", "addr", "add", v6, "dev", link, "nodad"], setup_end)
            self.administrative(["-n", name, "link", "set", link, "up"], setup_end)

    def namespace_links(self, role, end, cleanup=False):
        """Read affected namespace links only while actual name and retained handle identity agree."""
        name, dev, inode = self.names[role]
        path = Path("/run/netns") / name
        for s in (path.stat(), os.fstat(self.netfds[role])):
            if (s.st_dev, s.st_ino) != (dev, inode):
                raise Failure("namespace-unverified")
        rows = decode(self.administrative(["-n", name, "-j", "-d", "link", "show"], end, cleanup))
        if (not isinstance(rows, list) or len(rows) > 64 or any(not isinstance(v, dict) for v in rows)
                or len({v.get("ifname") for v in rows}) != len(rows)):
            raise Failure("namespace-unverified")
        s = path.stat()
        if (s.st_dev, s.st_ino) != (dev, inode):
            raise Failure("namespace-unverified")
        return rows

    def capture_link(self, role, name, end):
        """Retain a private kernel ifindex/MAC identity for an actual planned veth endpoint."""
        rows = [v for v in self.namespace_links(role, end) if v.get("ifname") == name]
        if (len(rows) != 1 or not exact_int(rows[0].get("ifindex"), 1, 2147483647)
                or rows[0].get("linkinfo", {}).get("info_kind") != "veth"
                or not isinstance(rows[0].get("address"), str)
                or not re.fullmatch(r"(?:[0-9a-f]{2}:){5}[0-9a-f]{2}", rows[0]["address"])):
            raise Failure("namespace-unverified")
        return rows[0]["ifindex"], rows[0]["address"]

    def cleanup_links(self):
        """Delete only identity-qualified veth endpoints, then measure absence in both affected namespaces."""
        if not self.links_attempted:
            return True
        good = True
        for role in ("canary", "calibration"):
            try:
                rows = self.namespace_links(role, self.end, True)
                for row in rows:
                    name = row.get("ifname")
                    if name not in self.links_attempted:
                        continue
                    expected = self.links.get((role, name))
                    if (expected is None or (row.get("ifindex"), row.get("address")) != expected
                            or row.get("linkinfo", {}).get("info_kind") != "veth"):
                        good = False
                        continue
                    self.administrative(["-n", self.names[role][0], "link", "delete", "dev", name], self.end, True)
            except BaseException:
                good = False
        # Kernel veth deletion removes its peer; both actual namespace inventories still require proof.
        for role in ("canary", "calibration"):
            try:
                rows = self.namespace_links(role, self.end, True)
                if any(row.get("ifname") in self.links_attempted for row in rows):
                    good = False
            except BaseException:
                good = False
        return good

    def dut_only(self):
        """Validate actual DUT link/address/route JSON; no host network is modified."""
        name = self.names["dut"][0]
        links = decode(self.administrative(["-n", name, "-j", "link", "show"]))
        addresses = decode(self.administrative(["-n", name, "-j", "addr", "show"]))
        if (not isinstance(links, list) or len(links) != 1 or links[0].get("ifname") != "lo"
                or "UP" not in links[0].get("flags", []) or not isinstance(addresses, list)
                or len(addresses) != 1 or addresses[0].get("ifname") != "lo"):
            raise Failure("namespace-unverified")
        actual = {(v.get("family"), v.get("local"), v.get("prefixlen"))
                  for v in addresses[0].get("addr_info", [])}
        if actual != {("inet", "127.0.0.1", 8), ("inet6", "::1", 128)}:
            raise Failure("namespace-unverified")
        for family in ("-4", "-6"):
            rows = decode(self.administrative(["-n", name, "-j", family, "route", "show", "table", "all"]))
            if not isinstance(rows, list) or len(rows) > 64:
                raise Failure("namespace-unverified")
            for row in rows:
                if (not isinstance(row, dict) or row.get("gateway") or
                        (row.get("dev") != "lo" and not
                         (row.get("type") == "unreachable" and row.get("dev") in (None, "lo")))):
                    raise Failure("namespace-unverified")

    def drop(self):
        """Join first elsewhere, then irreversibly clear bounding/ambient and all worker IDs/caps."""
        last = int(read_proc("/proc/sys/kernel/cap_last_cap", 32))
        if not 0 <= last <= 63:
            raise Failure("privilege-fence")
        for cap in range(last + 1):
            self.prctl(24, cap)  # PR_CAPBSET_DROP, while CAP_SETPCAP remains effective.
        self.call(self.lib.prctl, 47, 4, 0, 0, 0)  # PR_CAP_AMBIENT_CLEAR_ALL.
        self.prctl(8, 0)  # PR_SET_KEEPCAPS.
        os.setgroups([])
        os.setresgid(self.plan["target_gid"], self.plan["target_gid"], self.plan["target_gid"])
        os.setresuid(self.plan["target_uid"], self.plan["target_uid"], self.plan["target_uid"])
        header = (ctypes.c_uint32 * 2)(0x20080522, 0)
        data = (ctypes.c_uint32 * 6)(0, 0, 0, 0, 0, 0)
        self.call(self.lib.capset, ctypes.byref(header), ctypes.byref(data))
        self.prctl(38, 1)  # PR_SET_NO_NEW_PRIVS.

    def launch(self, role, callback, end, extra=()):
        """Fork a blocked worker, prove its actual reduced state, then release fixed work."""
        end = min(end, self.work_end)
        fence_end = min(end, self.setup_end) if self.in_setup else end
        remaining(fence_end)
        ready_r, ready_w = self.pipe()
        release_r, release_w = self.pipe()
        output_r, output_w = self.pipe()
        os.set_blocking(output_r, False)
        allowed = {0: "/dev/null", 1: os.readlink("/proc/self/fd/" + str(output_w)),
                   2: os.readlink("/proc/self/fd/" + str(output_w)),
                   ready_w: os.readlink("/proc/self/fd/" + str(ready_w)),
                   release_r: os.readlink("/proc/self/fd/" + str(release_r))}
        allowed.update({fd: os.readlink("/proc/self/fd/" + str(fd)) for fd in extra})
        child = None
        try:
            proof = self.owner_proof()
            if not proof:
                raise Failure("execution-unverified")
            remaining(fence_end)
            pid = os.fork()
            if pid == 0:
                try:
                    self.call(self.lib.setns, self.netfds[role], 0x40000000)
                    self.drop()
                    null = os.open("/dev/null", os.O_RDONLY)
                    os.dup2(null, 0)
                    os.dup2(output_w, 1)
                    os.dup2(output_w, 2)
                    for fd in bounded_fd_names("/proc/self/fd"):
                        if fd not in allowed:
                            try:
                                os.close(fd)
                            except OSError as error:
                                if error.errno != errno.EBADF:
                                    raise
                    os.environ.clear()
                    os.chdir(self.workspace)
                    os.write(ready_w, b"R")
                    if os.read(release_r, 1) != b"X":
                        os._exit(127)
                    os.close(ready_w)
                    os.close(release_r)
                    remaining(end)
                    callback(end)
                    os._exit(0)
                except BaseException:
                    os._exit(127)
            owned_output, output_r = output_r, None
            self.pending_fds.remove(owned_output)
            child = self.reserve(pid, output=owned_output, rollback=proof)
            if not select.select([ready_r], [], [], remaining(fence_end))[0] or os.read(ready_r, 2) != b"R":
                raise Failure("privilege-fence")
            record = process_record(pid, True)
            net = os.fstat(self.netfds[role])
            blocked_proof(record, child, (net.st_dev, net.st_ino), self.plan["target_uid"],
                          self.plan["target_gid"], allowed)
            self.observe_instance(record, role)
            remaining(fence_end)
            os.write(release_w, b"X")
            return child
        except BaseException:
            if child is not None and child.pidfd is not None:
                self.forced = True
                signal.pidfd_send_signal(child.pidfd, signal.SIGKILL)
            raise
        finally:
            for fd in (ready_r, ready_w, release_r, release_w, output_w, output_r):
                self.close_pending(fd)

    def observe_instance(self, record, role):
        """Prove one observed live runtime instance without an exhaustive fork claim."""
        key = (record["pid"], record["start"])
        if key not in self.observed:
            if len(self.observed) >= MAX_INSTANCES:
                raise Failure("execution-unverified")
            self.observed[key] = {"namespace": False, "privilege": False, "role": role, "pidfd": None}
        item = self.observed[key]
        expected = self.names[role][1:]
        item["namespace"] = record["net"] == expected
        item["privilege"] = privilege_proof(record, self.plan["target_uid"], self.plan["target_gid"])
        if not item["namespace"] or not item["privilege"]:
            raise Failure("privilege-fence")
        if item["pidfd"] is None:
            item["pidfd"] = os.pidfd_open(record["pid"], 0)
            if process_record(record["pid"])["start"] != record["start"]:
                raise Failure("execution-unverified")

    def scan(self):
        """Bound proc topology and prove observed owned runtime descendants, preserving wait owners."""
        try:
            entries = []
            with os.scandir("/proc") as rows:
                for count, entry in enumerate(rows, 1):
                    remaining(self.end)
                    if count > MAX_PROC:
                        raise Failure("execution-unverified")
                    if entry.name.isdigit():
                        entries.append(entry.name)
            topology = {}
            for name in entries:
                if name.isdigit():
                    try:
                        raw = read_proc(Path("/proc") / name / "stat", 8192)
                        fields = raw[raw.rfind(b")") + 2:].split()
                        topology[int(name)] = (int(fields[1]), int(fields[19]), fields[0])
                    except FileNotFoundError:
                        continue
            roles = {pid: item["role"] for (pid, _), item in self.observed.items()}
            changed = True
            while changed:
                changed = False
                for pid, (ppid, start, state) in topology.items():
                    if pid not in roles and ppid in roles:
                        roles[pid] = roles[ppid]
                        changed = True
                    elif pid not in roles and ppid == os.getpid() and not any(c.pid == pid for c in self.children):
                        roles[pid] = "dut"  # Sole private subreaper: an unknown adopted runtime child.
                        changed = True
            for pid, role in roles.items():
                row = topology.get(pid)
                if row is None:
                    continue
                prior = [key for key in self.observed if key[0] == pid]
                if prior and all(key[1] != row[1] for key in prior):
                    raise Failure("execution-unverified")
                if row[2] == b"Z":
                    if row[0] == os.getpid() and not any(c.pid == pid for c in self.children):
                        os.waitpid(pid, os.WNOHANG)
                    continue
                record = process_record(pid)
                if record["start"] != row[1]:
                    raise Failure("execution-unverified")
                self.observe_instance(record, role)
        except BaseException:
            self.scan_complete = False
            raise

    def await_child(self, child, end):
        """Require natural actual exit and EOF while scanning observed owned descendants."""
        while child.poll() is None or not child.eof:
            remaining(end)
            child.drain()
            self.scan()
            time.sleep(min(.01, remaining(end)))
        self.scan()
        remaining(end)
        return child.exit, bytes(child.captured)

    def server(self, role):
        """Start one dual-family numeric TCP challenge fixture in its own fenced namespace."""
        remaining(min(self.work_end, self.setup_end) if self.in_setup else self.work_end)
        stop_r, stop_w = self.pipe()

        def serve(end):
            """Bind numeric fixtures after privilege fencing and naturally stop at deadline."""
            listeners = []
            try:
                for family, address in ((socket.AF_INET, "198.18.0.1" if role == "canary" else "127.0.0.1"),
                                         (socket.AF_INET6, "fd00:62::1" if role == "canary" else "::1")):
                    s = socket.socket(family, socket.SOCK_STREAM)
                    if family == socket.AF_INET6:
                        s.setsockopt(socket.IPPROTO_IPV6, socket.IPV6_V6ONLY, 1)
                    s.bind((address, 0))
                    s.listen(16)
                    listeners.append(s)
                os.write(1, canonical([s.getsockname()[1] for s in listeners]))
                while time.monotonic() < end:
                    ready = select.select([stop_r, *listeners], [], [], min(.1, remaining(end)))[0]
                    if stop_r in ready:
                        if os.read(stop_r, 1) == b"":
                            break
                        raise Failure("execution-unverified")
                    for s in ready:
                        conn, _ = s.accept()
                        with conn:
                            conn.settimeout(min(2, remaining(end)))
                            frame = bytearray()
                            while len(frame) < 64:
                                conn.settimeout(min(2, remaining(end)))
                                block = conn.recv(64 - len(frame))
                                if not block:
                                    break
                                frame.extend(block)
                            nonce = bytes(frame[:32])
                            if len(frame) == 64 and hmac.compare_digest(
                                    bytes(frame[32:]), hmac.digest(self.challenge, nonce, "sha256")):
                                conn.sendall(hmac.digest(self.challenge, b"reply" + nonce, "sha256"))
            finally:
                for s in listeners:
                    s.close()
                os.close(stop_r)

        try:
            child = self.launch(role, serve, self.work_end, (stop_r,))
            child.stop = stop_w
            self.pending_fds.remove(stop_w)
        except BaseException:
            self.close_pending(stop_w)
            raise
        finally:
            self.close_pending(stop_r)
        self.servers[role] = child
        end = min(self.work_end, time.monotonic() + 2)
        while not child.captured.endswith(b"\n"):
            remaining(end)
            child.drain()
            if child.poll() is not None:
                raise Failure("probe-calibration-failed")
            time.sleep(.01)
        ports = decode(bytes(child.captured))
        if not isinstance(ports, list) or len(ports) != 2 or any(not exact_int(x, 1, 65535) for x in ports):
            raise Failure("probe-calibration-failed")
        child.captured.clear()
        self.ports[role] = ports

    def finish_setup(self):
        """Close the single setup deadline after both qualified dual-family fixtures are ready."""
        remaining(self.setup_end)
        self.in_setup = False

    def probe(self, family, role):
        """Observe an actual numeric TCP connect/challenge or exact ENETUNREACH denial."""
        end = min(self.work_end, time.monotonic() + 2)
        namespace = "calibration" if role == "calibration-external" else "dut"
        server = "dut" if role == "dut-loopback" else "canary"
        index = 0 if family == "ipv4" else 1
        address = (("127.0.0.1", "::1") if server == "dut" else ("198.18.0.1", "fd00:62::1"))[index]
        nonce = os.urandom(32)

        def connect(probe_end):
            """Perform one fixed peer connect in the released child's actual namespace."""
            af = socket.AF_INET if family == "ipv4" else socket.AF_INET6
            with socket.socket(af, socket.SOCK_STREAM) as s:
                s.settimeout(remaining(probe_end))
                try:
                    s.connect((address, self.ports[server][index]))
                except OSError as error:
                    result = {"outcome": "denied" if error.errno == errno.ENETUNREACH else "unverified",
                              "errno": 101 if error.errno == errno.ENETUNREACH else None,
                              "challenge_verified": False}
                else:
                    s.sendall(nonce + hmac.digest(self.challenge, nonce, "sha256"))
                    received = bytearray()
                    while len(received) < 32:
                        s.settimeout(remaining(probe_end))
                        chunk = s.recv(32 - len(received))
                        if not chunk:
                            break
                        received.extend(chunk)
                    result = {"outcome": "connected", "errno": None,
                              "challenge_verified": hmac.compare_digest(
                                  bytes(received), hmac.digest(self.challenge, b"reply" + nonce, "sha256"))}
                os.write(1, canonical(result))

        child = self.launch(namespace, connect, end)
        code, raw = self.await_child(child, end)
        if code != 0:
            raise Failure("probe-denial-unverified" if role == "dut-external" else "probe-calibration-failed")
        result = decode(raw)
        if (not isinstance(result, dict) or set(result) != {"outcome", "errno", "challenge_verified"}
                or result["outcome"] not in ("connected", "denied", "unverified")
                or result["errno"] not in (None, 101) or type(result["challenge_verified"]) is not bool):
            raise Failure("execution-unverified")
        return result

    def run_client(self):
        """Execute the unchanged protected client with fixed argv and no optimization."""
        receipt = self.workspace / "client-verification.json"
        end = min(self.work_end, time.monotonic() + 300)

        def execute(client_end):
            """Replace the fenced worker with the qualified interpreter and exact client source."""
            environment = {"HOME": str(self.workspace), "TMPDIR": str(self.workspace),
                           "PATH": "/usr/bin:/bin", "LC_ALL": "C.UTF-8"}
            remaining(client_end)
            os.execve(self.plan["python"], [self.plan["python"], "-E", "-s", "-S", "-B",
                      str(self.package / SOURCES[0]), "--forge", str(self.package / "forge"),
                      "--receipt", str(receipt)], environment)

        child = self.launch("dut", execute, end)
        code, _ = self.await_child(child, end)
        child.captured.clear()
        if code != 0:
            return code, None, None
        fd = os.open(receipt, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK)
        try:
            s = os.fstat(fd)
            if not stat.S_ISREG(s.st_mode) or s.st_size > MAX_CAPTURE:
                raise Failure("client-receipt-invalid")
            blocks = bytearray()
            while True:
                remaining(end)
                block = os.read(fd, min(8192, MAX_CAPTURE + 1 - len(blocks)))
                if not block:
                    break
                blocks.extend(block)
                if len(blocks) > MAX_CAPTURE:
                    raise Failure("client-receipt-invalid")
            raw = bytes(blocks)
            after, current = os.fstat(fd), receipt.stat(follow_symlinks=False)
            fields = ("st_dev", "st_ino", "st_size", "st_mtime_ns", "st_ctime_ns")
            if len(raw) != s.st_size or any(getattr(s, k) != getattr(after, k) or
                                         getattr(s, k) != getattr(current, k) for k in fields):
                raise Failure("client-receipt-invalid")
        finally:
            os.close(fd)
        with (self.package / SOURCES[3]).open("rb") as stream:
            contract = stream.read(MAX_CAPTURE + 1)
        declared = re.findall(rb"^      operationId: ([A-Za-z0-9_-]+)\s*$", contract, re.M)
        if len(contract) > MAX_CAPTURE or len(declared) != 39 or len(set(declared)) != 39:
            raise Failure("client-receipt-invalid")
        value = client_receipt(raw, {x.decode("ascii") for x in declared})
        self.publish("client-verification.json", raw)
        return code, value, {"bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}

    def stable(self):
        """Recheck protected and supplied identities after all actual client/probe work."""
        for key in (*SOURCES, "forge"):
            original = Path(self.plan["forge"]) if key == "forge" else Path(self.plan["root"]) / key
            expected = self.plan["release_pin"] if key == "forge" else self.plan["source_pins"][key]
            if file_pin(original, self.work_end) != expected or file_pin(self.package / key, self.work_end) != expected:
                raise Failure("source-changed")

    def accounting(self):
        """Expose counts only for actually observed runtime instances, with scan qualification."""
        return {"instances": len(self.observed),
                "namespace_checked": sum(item["namespace"] for item in self.observed.values()),
                "privilege_checked": sum(item["privilege"] for item in self.observed.values()),
                "scan_complete": self.scan_complete, "scope": "observed-process-instances-only"}

    def stop_direct(self, child):
        """Revoke pass before pidfd kill or the approved never-reaped sole-owner rollback."""
        self.forced = True
        if child.pidfd is not None:
            signal.pidfd_send_signal(child.pidfd, signal.SIGKILL)
        elif (child.rollback and child.exit is None and exact_int(child.pid, 1, 2147483647)
              and self.owner_proof()):
            # This PID came from our direct creation, never from a proc/supplied PID.
            # With DFL/no-NOCLDWAIT/one thread/no other waiter, reuse cannot precede our wait.
            os.kill(child.pid, signal.SIGKILL)
        else:
            raise Failure("cleanup-unverified")

    def cleanup(self):
        """Reconcile natural workers, fallback direct cleanup, owned references and named resources."""
        if not self.created and not self.children and not self.names_attempted:
            return {"state": "verified-not-created", "owned_processes_empty": False,
                    "owned_references_closed": False, "owned_names_absent": False,
                    "virtual_links_absent": False, "forced": False}
        good, empty, refs = True, False, not self.local_reference_failure
        for fd in list(self.pending_fds):
            self.close_pending(fd)
        if self.pending_fds or self.local_reference_failure:
            refs = False
        for child in self.servers.values():
            if child.stop is not None:
                try:
                    os.close(child.stop)
                    child.stop = None
                except OSError:
                    good = False
        grace = min(self.end, time.monotonic() + 5)
        while time.monotonic() < grace:
            try:
                self.scan()
                for child in self.children:
                    child.drain()
                if all(child.poll() is not None and child.eof for child in self.children):
                    break
            except BaseException:
                good = False
                break
            time.sleep(.01)
        # Even a failed proc/output scan must reach the reserved direct-child fallback.
        for child in self.children:
            try:
                if child.poll() is None:
                    self.stop_direct(child)
            except BaseException:
                good = False
        for item in self.observed.values():
            try:
                fd = item["pidfd"]
                if fd is not None and not select.select([fd], [], [], 0)[0]:
                    self.forced = True
                    signal.pidfd_send_signal(fd, signal.SIGKILL)
            except BaseException:
                good = False
        while time.monotonic() < self.end:
            try:
                statuses = [child.poll() for child in self.children]
                for child in self.children:
                    child.drain()
                self.scan()
                if all(code is not None for code in statuses) and all(child.eof for child in self.children) and all(
                        select.select([item["pidfd"]], [], [], 0)[0] for item in self.observed.values()
                        if item["pidfd"] is not None):
                    empty = True
                    break
            except BaseException:
                good = False
                # Direct statuses remain owned even when complete proc reconciliation fails.
                for child in self.children:
                    try:
                        child.poll()
                    except BaseException:
                        pass
                break
            time.sleep(.01)
        # Namespace/link identities remain held until the affected-scope absence observations finish.
        links_absent = self.cleanup_links()
        for fd in self.netfds.values():
            try:
                os.close(fd)
            except OSError:
                refs = False
        self.netfds.clear()
        if any(child.exit != 0 for child in self.servers.values()):
            good = False
        names_absent = True
        for name in reversed(self.names_attempted):
            path = Path("/run/netns") / name
            owned = next((v for v in self.names.values() if v[0] == name), None)
            try:
                if os.path.lexists(path):
                    s = path.stat()
                    if owned is None or (s.st_dev, s.st_ino) != owned[1:]:
                        names_absent = False
                        continue
                    self.administrative(["netns", "delete", name], self.end, True)
                if os.path.lexists(path):
                    names_absent = False
            except BaseException:
                names_absent = False
        # Cleanup ip children are reserved too: account and close them after the last operation.
        for child in self.children:
            try:
                if child.poll() is None:
                    empty = False
                    self.stop_direct(child)
                    while child.poll() is None and time.monotonic() < self.end:
                        time.sleep(.01)
                    good = False
                child.close()
            except BaseException:
                refs, good = False, False
        for item in self.observed.values():
            if item["pidfd"] is not None:
                try:
                    os.close(item["pidfd"])
                    item["pidfd"] = None
                except OSError:
                    refs = False
        if self.base is not None:
            try:
                remaining(self.end)
                remove_private(self.base, self.end)
            except BaseException:
                good = False
        verified = good and empty and refs and names_absent and links_absent and self.scan_complete
        return {"state": "verified" if verified else "unverified", "owned_processes_empty": empty,
                "owned_references_closed": refs, "owned_names_absent": names_absent,
                "virtual_links_absent": links_absent, "forced": self.forced}

    def publish(self, name, raw):
        """Publish only a fixed bounded file through the retained output dirfd without replacement."""
        if name not in ("os-denial-smoke.json", "client-verification.json") or len(raw) > MAX_CAPTURE:
            raise Failure("publication-failed")
        if self.output_fd is None:
            self.open_output()
        temp = ".receipt-" + os.urandom(8).hex()
        fd = os.open(temp, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW | os.O_CLOEXEC,
                     0o644, dir_fd=self.output_fd)
        try:
            os.fchmod(fd, 0o644)
            with os.fdopen(fd, "wb") as stream:
                stream.write(raw)
                stream.flush()
                os.fsync(stream.fileno())
            os.link(temp, name, src_dir_fd=self.output_fd, dst_dir_fd=self.output_fd, follow_symlinks=False)
            os.fsync(self.output_fd)
        finally:
            os.unlink(temp, dir_fd=self.output_fd)

    def finish_output(self):
        """Close the retained directory after publication without optimistic exit success."""
        if self.output_fd is not None:
            os.close(self.output_fd)
            self.output_fd = None


def expected_probe(row, result):
    """Accept only a live positive challenge or exact native external denial."""
    denied = row["role"] == "dut-external"
    return (result == {"outcome": "denied", "errno": 101, "challenge_verified": False} if denied
            else result == {"outcome": "connected", "errno": None, "challenge_verified": True})


def execute(plan, api):
    """Run fixed phases, retain first cause, independently revoke credit on cleanup failure."""
    receipt, primary = initial_receipt(plan), None
    try:
        receipt["platform"] = api.qualify()
        receipt["qualification"]["facilities_qualified"] = True
        api.open_output()
        release, sources = api.protect()
        receipt["pins"]["protected_release"] = release
        receipt["pins"]["protected_client_inputs"] = sources
        receipt["qualification"]["protected_execution"] = True
        api.setup()
        receipt["qualification"]["three_owned_namespaces"] = True
        api.dut_only()
        receipt["qualification"]["dut_lo_only_before"] = True
        api.server("canary")
        api.server("dut")
        api.finish_setup()
        receipt["qualification"]["canary_dual_family"] = True
        for phase in ("before", "after"):
            for row in (r for r in receipt["probes"] if r["phase"] == phase):
                result = api.probe(row["family"], row["role"])
                row.update(result)
                row["status"] = "passed" if expected_probe(row, result) else "failed"
                if row["status"] != "passed":
                    raise Failure("probe-denial-unverified" if row["role"] == "dut-external" else
                                  "loopback-unverified" if row["role"] == "dut-loopback" else "probe-calibration-failed")
            if phase == "before":
                code, value, pin = api.run_client()
                receipt["client"].update(status="failed", exit_code=code)
                if code != 0 or value is None or pin is None:
                    raise Failure("client-failed")
                receipt["client"].update(status="passed", receipt=value, receipt_pin=pin)
                receipt["qualification"]["client_preexec_fence"] = True
        api.dut_only()
        receipt["qualification"]["dut_lo_only_after"] = True
        api.stable()
    except Failure as fault:
        primary = fault
    except BaseException:
        primary = Failure("execution-unverified")
    finally:
        try:
            receipt["cleanup"] = api.cleanup()
            receipt["observed_runtime"] = api.accounting()
        except BaseException:
            receipt["cleanup"]["state"] = "unverified"
    cleanup = receipt["cleanup"]
    if cleanup["state"] == "verified" and not all(cleanup[k] is True for k in
            ("owned_processes_empty", "owned_references_closed", "owned_names_absent", "virtual_links_absent")):
        cleanup["state"] = "unverified"
    if cleanup["state"] == "unverified":
        receipt["cleanup_failure"] = "cleanup-unverified"
    elif cleanup["forced"]:
        receipt["cleanup_failure"] = "forced-cleanup"
    if primary is not None:
        receipt["failure"] = primary.code
        receipt["status"] = ("incomplete" if primary.unavailable and receipt["cleanup_failure"] is None
                             else "failed")
        if receipt["status"] == "incomplete" and cleanup["state"] == "verified-not-created":
            receipt["qualification"] = dict.fromkeys(QUALIFICATIONS, False)
            receipt["observed_runtime"] = initial_receipt(plan)["observed_runtime"]
        elif cleanup["state"] == "verified-not-created":
            cleanup["state"] = "unverified"
            receipt["cleanup_failure"] = "cleanup-unverified"
    elif receipt["cleanup_failure"] is not None:
        receipt["failure"] = "cleanup-unverified"
    else:
        a = receipt["observed_runtime"]
        if (not all(receipt["qualification"].values()) or a["instances"] <= 0
                or a["instances"] != a["namespace_checked"] or a["instances"] != a["privilege_checked"]
                or not a["scan_complete"] or cleanup["state"] != "verified"
                or not all(cleanup[k] is True for k in ("owned_processes_empty", "owned_references_closed",
                                                       "owned_names_absent", "virtual_links_absent"))):
            receipt["failure"] = "execution-unverified"
        else:
            receipt["status"] = "passed"
    return receipt


def main():
    """Read the sole bounded private plan and publish a redacted receipt with reconciled exit."""
    if sys.argv[1:] != ["--plan-stdin"]:
        return 1
    api = None
    try:
        raw = read_plan(sys.stdin.buffer)
        plan = parse_plan(raw, time.monotonic_ns())
        api = Native(plan)
        receipt = execute(plan, api)
        api.publish("os-denial-smoke.json", canonical(receipt))
        api.finish_output()
        return {"passed": 0, "failed": 1, "incomplete": 2}[receipt["status"]]
    except BaseException:
        if api is not None:
            try:
                api.finish_output()
            except BaseException:
                pass
        return 1


if __name__ == "__main__":
    sys.exit(main())
