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
from types import MappingProxyType

import verify_workspace as shared

sys.dont_write_bytecode = True
SCHEMA = "forge.workspace-os-denial-verification/2"
NATIVE_SCHEMA = "forge.packaged-runtime-os-denial/1"
SCOPE = "linux-headless-ip-network-denial-prerequisite"
OUTPUT = "workspace-os-denial-verification.json"
NATIVE_OUTPUT = "os-denial-smoke.json"
CLIENT_OUTPUT = "client-verification.json"
MAX_CAPTURE = 262144
MAX_ENTRIES = 20000
MAX_STDLIB_BYTES = 268435456
# Private fixed administrative names; availability and every existing trust gate remain mandatory.
_ADMINISTRATIVE_PATHS = MappingProxyType({
    "python": "/usr/bin/python3", "ip": "/usr/bin/ip", "sudo": "/usr/bin/sudo",
})
SOURCE_KEYS = ("scripts/test_workspace_client.py", "scripts/workspace_client.py",
               "scripts/verify_workspace.py", "docs/api/forge-workspace-v1.openapi.yaml")
EXTRA_INPUTS = ("scripts/test_workspace_os_denial.py", "scripts/verify_workspace_os_denial.py",
                "scripts/test_verify_workspace_os_denial.py", "scripts/test_workspace_stdlib_link_diagnostic.py")
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
# Fixed diagnostic vocabulary contains no filesystem names, arbitrary messages or raw observations.
PATH_REASONS = ("missing", "not-absolute", "not-root-owned", "worker-writable", "unsupported-link",
                "not-directory", "not-regular", "link-bound", "not-executable", "path-observation-unverified")
COMMAND_REASONS = ("command-failed", "command-timeout", "output-bound", "command-cleanup-unverified")
TOOL_PHASE_REASONS = {
    "python-path": PATH_REASONS, "ip-path": PATH_REASONS, "sudo-path": PATH_REASONS,
    "python-probe": COMMAND_REASONS + ("probe-json-invalid", "probe-shape-invalid", "version-invalid"),
    "stdlib-roots": ("root-shape-invalid", "missing-root", "not-root-owned", "worker-writable", "unsupported-link",
                     "not-directory", "not-regular", "dynload-missing", "path-observation-unverified"),
    "stdlib-entry": ("not-root-owned", "worker-writable", "unsupported-link", "not-directory", "not-regular",
                     "unsupported-kind", "entry-bound", "depth-bound", "byte-bound", "entry-observation-unverified"),
    "ip-version": COMMAND_REASONS + ("version-invalid", "identity-observation-unverified"),
    "sudo-version": COMMAND_REASONS + ("version-invalid", "identity-observation-unverified"),
    "ordinary-tools": ("required-identity-missing", "identity-observation-unverified"),
    "qualification-budget": ("deadline-expired",),
}
TOOL_DEFAULT_REASON = {"python-path": "path-observation-unverified", "ip-path": "path-observation-unverified",
    "sudo-path": "path-observation-unverified", "python-probe": "command-failed",
    "stdlib-roots": "path-observation-unverified", "stdlib-entry": "entry-observation-unverified",
    "ip-version": "identity-observation-unverified", "sudo-version": "identity-observation-unverified",
    "ordinary-tools": "identity-observation-unverified", "qualification-budget": "deadline-expired"}
TOOL_COMMAND_PHASES = frozenset(("python-probe", "ip-version", "sudo-version"))
TOOL_REASONS = frozenset(reason for reasons in TOOL_PHASE_REASONS.values() for reason in reasons)
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

    def __init__(self, code, incomplete=False, *, tool_reason=None, diagnostic=None):
        """Keep the existing code/resource classification and optional fixed reason/closed diagnostic, never private text."""
        if code not in OUTER_FAILURES:
            raise ValueError("unknown failure code")
        if tool_reason is not None and (type(tool_reason) is not str or tool_reason not in TOOL_REASONS):
            raise ValueError("unknown tool reason")
        self.code = code
        self.incomplete = incomplete
        self.tool_reason = tool_reason
        self.diagnostic = validate_tool_diagnostic(diagnostic)
        self.link_subcondition = None
        super().__init__(code)


def validate_tool_diagnostic(value):
    """Accept null or one exact fixed phase/reason/status tuple; reject bools, unknown pairs and private fields."""
    if value is None:
        return None
    if type(value) is not dict or set(value) != {"phase", "reason", "exit_code"}:
        raise ValueError("invalid tool diagnostic")
    phase, reason, code = value["phase"], value["reason"], value["exit_code"]
    if type(phase) is not str or phase not in TOOL_PHASE_REASONS or type(reason) is not str or reason not in TOOL_PHASE_REASONS[phase]:
        raise ValueError("invalid tool diagnostic pair")
    if code is not None and (type(code) is not int or not -255 <= code <= 255):
        raise ValueError("invalid tool diagnostic status")
    if code is not None and phase not in TOOL_COMMAND_PHASES:
        raise ValueError("status outside command phase")
    if reason in ("probe-json-invalid", "probe-shape-invalid", "version-invalid") and code not in (None, 0):
        raise ValueError("invalid parsed observation status")
    return dict(value)


def tool_diagnostic(phase, reason, exit_code=None):
    """Construct a validated public tuple from authored constants and an optional actual child status."""
    return validate_tool_diagnostic({"phase": phase, "reason": reason, "exit_code": exit_code})


def observed_exit(value):
    """Retain only a signed bounded exact integer; absent or malformed internal status is unobserved, never coerced."""
    return value if type(value) is int and -255 <= value <= 255 else None


def tool_step(phase, callback, *args, reason=None, exit_code=None):
    """Call the unchanged adapter signature and attach a safe first qualification fault without changing its status code."""
    if type(phase) is not str or phase not in TOOL_PHASE_REASONS or reason is not None and reason not in TOOL_PHASE_REASONS[phase]:
        raise ValueError("invalid tool step")
    try:
        return callback(*args)
    except GateError as error:
        if error.diagnostic is None:
            selected = error.tool_reason or reason or TOOL_DEFAULT_REASON[phase]
            if selected not in TOOL_PHASE_REASONS[phase]:
                selected = TOOL_DEFAULT_REASON[phase]
            error.diagnostic = tool_diagnostic(phase, selected, observed_exit(exit_code))
        raise
    except Exception:
        # These faults previously reached verify's generic failed/verification-input-invalid branch.
        raise GateError("verification-input-invalid", diagnostic=tool_diagnostic(
            phase, reason or TOOL_DEFAULT_REASON[phase], observed_exit(exit_code))) from None


def tool_budget(deadline):
    """Mark the existing absolute qualification expiry boundary without renewing it or dispatching new work."""
    if time.monotonic() >= deadline:
        raise GateError("command-timeout", True, diagnostic=tool_diagnostic("qualification-budget", "deadline-expired"))


def command_diagnostic(phase, observation):
    """Map existing command failures to fixed reasons while preserving the relevant observed return status only."""
    reasons = {"command-timeout": "command-timeout", "output-bound": "output-bound",
               "cleanup-unverified": "command-cleanup-unverified"}
    return tool_diagnostic(phase, reasons.get(observation["failure"], "command-failed"),
                           observed_exit(observation["exit_code"]))


def retain_tool_diagnostic(receipt, error):
    """Keep the first typed qualification diagnostic while later observations retain all original failure overrides."""
    if receipt["diagnostic"] is None and isinstance(error, GateError) and error.diagnostic is not None:
        receipt["diagnostic"] = validate_tool_diagnostic(error.diagnostic)


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
    """Reject unowned objects and links before applying write-bit checks to non-link targets and ancestors."""
    path = Path(path)
    if not path.is_absolute():
        raise GateError("tool-untrusted", tool_reason="not-absolute")
    item = path
    while True:
        info = item.lstat()
        if info.st_uid != 0:
            raise GateError("tool-untrusted", tool_reason="not-root-owned")
        if stat.S_ISLNK(info.st_mode):
            raise GateError("tool-untrusted", tool_reason="unsupported-link")
        if info.st_mode & 0o022:
            raise GateError("tool-untrusted", tool_reason="worker-writable")
        if item == path:
            if directory and not stat.S_ISDIR(info.st_mode) or not directory and not stat.S_ISREG(info.st_mode):
                raise GateError("tool-untrusted", tool_reason="not-directory" if directory else "not-regular")
        elif not stat.S_ISDIR(info.st_mode):
            raise GateError("tool-untrusted", tool_reason="not-directory")
        if item.parent == item:
            break
        item = item.parent
    return path


def administration_tool(path):
    """Resolve only an already-present tool through root-trusted link ancestors, with no install or fallback."""
    path = Path(path)
    if not path.exists():
        raise GateError("tool-unavailable", True, tool_reason="missing")
    # A fixed executable symlink may resolve to another trusted root-owned file; directory trees do not follow links.
    link = path.lstat()
    if link.st_uid != 0:
        raise GateError("tool-untrusted", True, tool_reason="not-root-owned")
    if link.st_mode & 0o022 and not stat.S_ISLNK(link.st_mode):
        raise GateError("tool-untrusted", True, tool_reason="worker-writable")
    root_trusted(path.parent, True)
    current = path
    for _link in range(16):
        root_trusted(current.parent, True)
        info = current.lstat()
        if info.st_uid != 0:
            raise GateError("tool-untrusted", True, tool_reason="not-root-owned")
        if not stat.S_ISLNK(info.st_mode):
            break
        target = Path(os.readlink(current))
        current = target if target.is_absolute() else current.parent / target
    else:
        raise GateError("tool-untrusted", True, tool_reason="link-bound")
    actual = current.resolve(strict=True)
    root_trusted(actual)
    if not os.access(actual, os.X_OK):
        raise GateError("tool-untrusted", True, tool_reason="not-executable")
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


# Separate fixed raise-site observation; the shared leaf-proof engine below is unchanged.
LINK_DETAIL_SCHEMA = "forge.stdlib-unsupported-link-diagnostic/1"
LINK_DETAIL_OUTPUT = "stdlib-unsupported-link-diagnostic.json"
LINK_DETAIL_LIMIT = 2048
LINK_DETAIL_ENGINE_PIN = {"bytes": 30623, "sha256": "0364b0892a31f7d80ab0b38e70a85a7cba285645f2966838eb42f8735122aa65"}
LINK_DETAIL_INPUTS = ("scripts/verify_workspace_os_denial.py", "scripts/test_workspace_os_denial.py",
                      "scripts/test_workspace_stdlib_link_diagnostic.py", ".github/workflows/workspace-verification.yml")
# Offsets are relative to the actual maintained method's def line, not its filename or a caller label.
LINK_DETAIL_SITES = {
    "trusted": {6: "required-kind-symlink"},
    "read_link": {3: "inventoried-link-size", 10: "held-link-size", 17: "link-text-shape", 21: "link-text-utf8"},
    "resolve": {5: "origin-not-in-link-inventory", 9: "physical-link-cycle", 35: "lexical-parent-escape",
                40: "target-not-in-inventory", 47: "held-parent-escape", 62: "final-component-shape",
                76: "resolved-target-not-in-inventory", 110: "link-hop-limit"},
}
LINK_DETAIL_CODES = frozenset(code for sites in LINK_DETAIL_SITES.values() for code in sites.values())


def link_rejection_site(error):
    """Classify only the actual innermost maintained raise site; never read frame locals, paths or exception prose."""
    if type(error) is not LeafClosureError or (error.phase, error.reason) != ("stdlib-entry", "unsupported-link"):
        return None
    frame = error.__traceback__
    last = None
    for _ in range(16):
        if frame is None:
            break
        last, frame = frame, frame.tb_next
    if frame is not None or last is None:
        return None
    for name, offsets in LINK_DETAIL_SITES.items():
        code = getattr(LeafClosure, name).__code__
        if last.tb_frame.f_code is code:
            return offsets.get(last.tb_lineno - code.co_firstlineno)
    return None


def link_diagnostic_record(root, destination, receipt, subcondition):
    """Bind a fixed site to actual unchanged inputs and published outer bytes; no object or target facts are inferred."""
    if (subcondition not in LINK_DETAIL_CODES or receipt["status"] != "incomplete"
            or receipt["failure"] != "tool-untrusted" or receipt["input_stability"] != "unchanged"
            or receipt["diagnostic"] != {"phase": "stdlib-entry", "reason": "unsupported-link", "exit_code": None}
            or receipt["producer"] != {"status": "not-run", "exit_code": None, "failure": None,
                                       "receipt": None, "receipt_pin": None}
            or receipt["cleanup"] != {"state": "verified-not-created", "forced": False}):
        return None
    inputs = {key: pin(receipt["identity"]["inputs"][key]) for key in LINK_DETAIL_INPUTS}
    raw = read_bytes(root / LINK_DETAIL_INPUTS[0])
    if pin_bytes(raw) != inputs[LINK_DETAIL_INPUTS[0]]:
        return None
    first = raw.index(b"\nLEAF_PROOF_FORMAT =") + 1
    last = raw.index(b"\n\ndef stdlib_inventory", first)
    actual_engine = pin_bytes(raw[first:last])
    if actual_engine != LINK_DETAIL_ENGINE_PIN:
        return None
    outer = shared.canonical_bytes(receipt)
    if read_bytes(destination / OUTPUT) != outer:
        return None
    checkout = receipt["checkout"]
    requested, tested = checkout["requested_head"], checkout["tested_commit"]
    if (requested is not None and (type(requested) is not str or re.fullmatch(r"[0-9a-f]{40}", requested) is None)
            or type(tested) is not str or re.fullmatch(r"[0-9a-f]{40}", tested) is None):
        return None
    return {"schema_version": LINK_DETAIL_SCHEMA, "scope": "ordinary-stdlib-rejection-raise-site",
            "truth_state": "synthetic-development", "acceptance_eligible": False,
            "status": "observed-rejection", "phase": "stdlib-entry", "reason": "unsupported-link",
            "subcondition": subcondition, "requested_commit": requested, "tested_commit": tested,
            "source_inputs": inputs, "engine_pin": actual_engine, "outer_receipt_pin": pin_bytes(outer)}


def publish_link_diagnostic(root, destination, receipt, subcondition, deadline, published):
    """Keep this bounded, no-replace observation secondary to every original failure, fence and cleanup outcome."""
    try:
        if subcondition is None or time.monotonic() >= deadline:
            return False
        record = link_diagnostic_record(root, destination, receipt, subcondition)
        if record is None or len(shared.canonical_bytes(record)) > LINK_DETAIL_LIMIT or time.monotonic() >= deadline:
            return False
        # The maintained publisher owns its staging descriptor/name and refuses any existing destination.
        shared.atomic_receipt(destination / LINK_DETAIL_OUTPUT, record)
        if published is not None:
            published()
        return True
    except Exception:
        # No secondary parser/publication/callback fault changes the already published primary receipt or exit.
        return False


def emit_link_diagnostic_publication_flag():
    """Append only the fixed runner flag after successful fresh sidecar publication; existence is never authority."""
    descriptor = None
    accepted = False
    try:
        name = os.environ.get("GITHUB_OUTPUT")
        if not name:
            return False
        path = Path(name)
        before = path.lstat()
        if not stat.S_ISREG(before.st_mode) or before.st_nlink != 1 or before.st_size > 1024 * 1024:
            return False
        flags = os.O_WRONLY | os.O_APPEND | getattr(os, "O_NOFOLLOW", 0)
        flags |= getattr(os, "O_NONBLOCK", 0) | getattr(os, "O_CLOEXEC", 0)
        flags |= getattr(os, "O_BINARY", 0)
        descriptor = os.open(path, flags)
        held = os.fstat(descriptor)
        identity = (before.st_dev, before.st_ino)
        if (not stat.S_ISREG(held.st_mode) or held.st_nlink != 1 or
                (held.st_dev, held.st_ino) != identity or held.st_size > 1024 * 1024):
            return False
        line = b"stdlib_link_detail_published=true\n"
        if os.write(descriptor, line) != len(line):
            return False
        after = path.lstat()
        accepted = stat.S_ISREG(after.st_mode) and (after.st_dev, after.st_ino) == identity
    except Exception:
        accepted = False
    finally:
        if descriptor is not None:
            try:
                os.close(descriptor)
            except Exception:
                accepted = False
    # A complete flag already written cannot be retracted after a later fault.
    # Its prerequisite remains successful NEW bounded publication, never a stale file.
    return accepted


# Both qualifiers contain this literal proof engine; the engine imports no checkout helper.
# Its independently computed proof is compared through the closed private plan.
LEAF_PROOF_FORMAT = "forge.stdlib-leaf-closure/1"
LEAF_ENTRY_LIMIT = 20000
LEAF_BYTE_LIMIT = 268435456
LEAF_DEPTH_LIMIT = 32
LEAF_PATH_LIMIT = 4096
LEAF_HOP_LIMIT = 16
LEAF_WORK_LIMIT = 640000
LEAF_FD_LIMIT = 64
LEAF_STATE_LIMIT = 10485760
LEAF_READ_SIZE = 32768


class LeafClosureError(Exception):
    """Carry only a fixed qualification reason; OS details and private paths stay private."""

    def __init__(self, reason, phase="stdlib-entry"):
        """Keep the first fixed failure and its existing root/entry diagnostic phase."""
        self.reason, self.phase = reason, phase
        super().__init__(reason)


def leaf_identity(info):
    """Pin owner, mode and full observed generation without interpreting symlink permissions."""
    return [info.st_dev, info.st_ino, info.st_mode, info.st_uid, info.st_gid,
            info.st_size, info.st_mtime_ns, info.st_ctime_ns]


def leaf_canonical(value):
    """Encode private proof rows deterministically; no private row is a public receipt."""
    return (json.dumps(value, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False) + "\n").encode("ascii")


def leaf_encoded_size(value):
    """Measure the bounded private JSON tree before whole-row/proof encoding or list retention."""
    if type(value) is str:
        size = 2
        for character in value:
            code = ord(character)
            size += (2 if character in '\\"' or character in '\b\f\n\r\t' else
                     6 if code < 32 or 126 < code <= 65535 else 12 if code > 65535 else 1)
        return size
    if type(value) is int:
        return len(str(value))
    if type(value) is list:
        return 2 + max(0, len(value) - 1) + sum(leaf_encoded_size(item) for item in value)
    if type(value) is dict:
        return 2 + max(0, len(value) - 1) + sum(leaf_encoded_size(key) + 1 + leaf_encoded_size(item)
                                             for key, item in value.items())
    raise LeafClosureError("entry-observation-unverified")


def leaf_summary_valid(value):
    """Require a closed aggregate proof summary before privileged comparison or dispatch."""
    if type(value) is not dict or set(value) != {"format", "pin", "entries", "charged_bytes", "link_hops"}:
        return False
    pin = value["pin"]
    return (value["format"] == LEAF_PROOF_FORMAT and type(pin) is dict
            and set(pin) == {"bytes", "sha256"} and type(pin["bytes"]) is int
            and 1 <= pin["bytes"] <= LEAF_STATE_LIMIT
            and type(pin["sha256"]) is str and re.fullmatch(r"[0-9a-f]{64}", pin["sha256"]) is not None
            and type(value["entries"]) is int and 1 <= value["entries"] <= LEAF_ENTRY_LIMIT
            and type(value["charged_bytes"]) is int and 0 <= value["charged_bytes"] <= LEAF_BYTE_LIMIT
            and type(value["link_hops"]) is int and 0 <= value["link_hops"] <= LEAF_ENTRY_LIMIT * LEAF_HOP_LIMIT)


class LeafBudget:
    """Own one nonrenewing deadline and aggregate work, read, proof and descriptor charges."""

    def __init__(self, deadline):
        """Start one budget for both complete inventories and every logical alias resolution."""
        self.deadline = deadline
        self.work = self.entries = self.bytes = self.state = self.hops = 0
        self.held = set()
        self.streams = {}
        self.uncertain_fds = 0

    def check(self):
        """Fence every blocking primitive cooperatively; this is not syscall preemption."""
        if time.monotonic() >= self.deadline:
            raise LeafClosureError("deadline-expired", "qualification-budget")

    def charge(self, field, amount, maximum, reason):
        """Charge monotonically before growth, opening or reading; no per-root renewal exists."""
        self.check()
        value = getattr(self, field) + amount
        if amount < 0 or value > maximum:
            raise LeafClosureError(reason)
        setattr(self, field, value)

    def open(self, path, flags, parent=None):
        """Reserve a bounded owned handle before open and keep it through every later fault."""
        self.charge("work", 1, LEAF_WORK_LIMIT, "entry-bound")
        if len(self.held) + len(self.streams) + self.uncertain_fds >= LEAF_FD_LIMIT:
            raise LeafClosureError("entry-bound")
        fd = os.open(path, flags, dir_fd=parent)
        self.held.add(fd)
        return fd

    def scandir(self, fd):
        """Reserve one native directory-stream handle before creation alongside every held descriptor."""
        self.charge("work", 1, LEAF_WORK_LIMIT, "entry-bound")
        if len(self.held) + len(self.streams) + self.uncertain_fds >= LEAF_FD_LIMIT:
            raise LeafClosureError("entry-bound")
        token = object()
        self.streams[token] = None
        try:
            self.streams[token] = os.scandir(fd)
        except BaseException:
            del self.streams[token]
            raise
        return LeafDirectoryStream(self, token)

    def close_stream(self, token):
        """Credit a stream reservation only after its owned iterator successfully closes."""
        if token in self.streams:
            try:
                self.streams[token].close()
            except BaseException:
                raise LeafClosureError("entry-observation-unverified") from None
            del self.streams[token]

    def close(self, fd):
        """Consume one close attempt; uncertainty stays charged without retrying a possibly reused numeric FD."""
        if fd in self.held:
            self.held.remove(fd)
            try:
                os.close(fd)
            except BaseException:
                self.uncertain_fds += 1
                raise LeafClosureError("entry-observation-unverified") from None

    def close_all(self):
        """Attempt every independent owned close even when one fails; unknown cleanup cannot pass."""
        failed = False
        for token in tuple(self.streams):
            try:
                self.close_stream(token)
            except BaseException:
                failed = True
        for fd in tuple(self.held):
            try:
                self.close(fd)
            except BaseException:
                failed = True
        if failed or self.uncertain_fds:
            raise LeafClosureError("entry-observation-unverified")
        self.check()


class LeafDirectoryStream:
    """Keep an actual scandir iterator charged until close succeeds, including exceptional traversal."""

    def __init__(self, budget, token):
        """Bind this context to the single reservation already made before iterator creation."""
        self.budget = budget
        self.token = token

    def __enter__(self):
        """Expose the owned iterator without creating another stream or changing its charge."""
        return self.budget.streams[self.token]

    def __exit__(self, exc_type, exc, traceback):
        """Close even on traversal failure; uncertainty remains charged and cannot qualify."""
        self.budget.close_stream(self.token)
        return False


class LeafClosure:
    """Build a complete private stdlib proof using held no-follow directories, leaves and link text."""

    def __init__(self, paths, deadline):
        """Validate fixed root spellings before any open; callers cannot add an alternative root."""
        if (type(paths) is not list or not 2 <= len(paths) <= 8
                or any(type(p) is not str for p in paths) or len(paths) != len(set(paths))):
            raise LeafClosureError("root-shape-invalid", "stdlib-roots")
        self.paths = []
        for path in paths:
            try:
                raw = path.encode("utf-8", "strict")
            except UnicodeError:
                raise LeafClosureError("root-shape-invalid", "stdlib-roots") from None
            if (not raw.startswith(b"/") or len(raw) > LEAF_PATH_LIMIT
                    or b"\0" in raw or b"//" in raw or raw.endswith(b"/")
                    or any(p in (b".", b"..") for p in raw.split(b"/")[1:])):
                raise LeafClosureError("root-shape-invalid", "stdlib-roots")
            self.paths.append(raw)
        if not any(p.rsplit(b"/", 1)[-1] == b"lib-dynload" for p in self.paths):
            raise LeafClosureError("dynload-missing", "stdlib-roots")
        self.budget = LeafBudget(deadline)
        self.slash = None
        self.slash_generation = None
        self.roots = []
        self.scan_entries = 0

    def trusted(self, info, kind):
        """Require UID0 first, refuse incorrect kinds, and apply 022 only to nonlinks."""
        if info.st_uid != 0:
            raise LeafClosureError("not-root-owned")
        expected = {"directory": stat.S_ISDIR, "file": stat.S_ISREG, "link": stat.S_ISLNK}[kind]
        if not expected(info.st_mode):
            raise LeafClosureError("unsupported-link" if stat.S_ISLNK(info.st_mode) else
                                   "not-directory" if kind == "directory" else "not-regular")
        if kind != "link" and info.st_mode & 0o022:
            raise LeafClosureError("worker-writable")

    def verify_slash(self):
        """Bind held slash to the current no-follow root identity before and after complete qualification."""
        self.budget.check()
        if (leaf_identity(os.fstat(self.slash)) != self.slash_generation
                or leaf_identity(os.stat(b"/", follow_symlinks=False)) != self.slash_generation):
            raise LeafClosureError("entry-observation-unverified")

    def verify(self, parent, name, fd, first):
        """Reconcile the held object with its current no-follow parent entry and full generation."""
        self.budget.check()
        if (leaf_identity(os.fstat(fd)) != first
                or leaf_identity(os.stat(name, dir_fd=parent, follow_symlinks=False)) != first):
            raise LeafClosureError("entry-observation-unverified")

    def directory(self, parent, name):
        """Open one trusted directory component without following a root/intermediate alias."""
        self.budget.charge("work", 1, LEAF_WORK_LIMIT, "entry-bound")
        before = os.stat(name, dir_fd=parent, follow_symlinks=False)
        self.trusted(before, "directory")
        fd = self.budget.open(name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC, parent)
        info = os.fstat(fd)
        if leaf_identity(info) != leaf_identity(before):
            raise LeafClosureError("entry-observation-unverified")
        self.trusted(info, "directory")
        identity = leaf_identity(info)
        self.verify(parent, name, fd, identity)
        return fd, identity

    def absolute_directory(self, raw):
        """Walk from held slash, recording every qualified ancestor and closing superseded handles."""
        components = raw.split(b"/")[1:] if raw != b"/" else []
        if len(components) > LEAF_DEPTH_LIMIT:
            raise LeafClosureError("depth-bound")
        fd, identities, partial = self.slash, [[b"/".hex(), leaf_identity(os.fstat(self.slash))]], b""
        try:
            for name in components:
                child, identity = self.directory(fd, name)
                if fd != self.slash:
                    self.budget.close(fd)
                fd = child
                partial += b"/" + name
                self.row(identities, [partial.hex(), identity])
            return fd, identities
        except BaseException:
            if fd != self.slash:
                self.budget.close(fd)
            raise

    def root_inventory(self):
        """Hold every present root and pin trusted absence of optional zip roots without following them."""
        rows = []
        for raw in self.paths:
            parent_raw, name = raw.rsplit(b"/", 1)
            parent, ancestry = self.absolute_directory(parent_raw or b"/")
            try:
                try:
                    info = os.stat(name, dir_fd=parent, follow_symlinks=False)
                except FileNotFoundError:
                    if not name.endswith(b".zip"):
                        raise LeafClosureError("missing-root", "stdlib-roots") from None
                    self.row(rows, [raw.hex(), "absent", ancestry])
                    continue
                self.trusted(info, "directory")
                fd, identity = self.directory(parent, name)
                self.roots.append((raw, fd, identity, ancestry))
                self.row(rows, [raw.hex(), "directory", identity, ancestry])
            finally:
                if parent != self.slash:
                    self.budget.close(parent)
        if (len(self.roots) < 2 or not any(raw.endswith(b"/lib-dynload") for raw, *_ in self.roots)):
            raise LeafClosureError("dynload-missing", "stdlib-roots")
        return rows

    def qualified_roots(self):
        """Preserve root diagnostic classification while retaining fixed aggregate-bound/observation failures."""
        try:
            return self.root_inventory()
        except LeafClosureError as error:
            if error.reason in {"missing-root", "not-root-owned", "worker-writable", "unsupported-link",
                                "not-directory", "not-regular", "dynload-missing"}:
                raise LeafClosureError(error.reason, "stdlib-roots") from None
            raise

    def hash_leaf(self, parent, name, expected):
        """Precharge full size, hash the held regular file to exact EOF, then reconcile its parent entry."""
        self.budget.charge("bytes", expected[5], LEAF_BYTE_LIMIT, "byte-bound")
        fd = self.budget.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK, parent)
        try:
            info = os.fstat(fd)
            self.trusted(info, "file")
            if leaf_identity(info) != expected:
                raise LeafClosureError("entry-observation-unverified")
            digest, length = hashlib.sha256(), 0
            while True:
                self.budget.check()
                block = os.read(fd, min(LEAF_READ_SIZE, expected[5] - length + 1))
                if not block:
                    break
                length += len(block)
                if length > expected[5]:
                    raise LeafClosureError("entry-observation-unverified")
                digest.update(block)
            if length != expected[5]:
                raise LeafClosureError("entry-observation-unverified")
            self.verify(parent, name, fd, expected)
            return {"bytes": length, "sha256": digest.hexdigest()}
        finally:
            self.budget.close(fd)

    def read_link(self, parent, name, expected):
        """Read raw Linux link text through its held O_PATH identity; no automatic target traversal occurs."""
        if not 1 <= expected[5] <= LEAF_PATH_LIMIT:
            raise LeafClosureError("unsupported-link")
        self.budget.charge("state", expected[5], LEAF_STATE_LIMIT, "byte-bound")
        fd = self.budget.open(name, os.O_PATH | os.O_NOFOLLOW | os.O_CLOEXEC, parent)
        try:
            info = os.fstat(fd)
            self.trusted(info, "link")
            if not 1 <= info.st_size <= LEAF_PATH_LIMIT:
                raise LeafClosureError("unsupported-link")
            if leaf_identity(info) != expected:
                raise LeafClosureError("entry-observation-unverified")
            self.budget.charge("work", 1, LEAF_WORK_LIMIT, "entry-bound")
            raw = os.readlink(b"", dir_fd=fd)
            if (type(raw) is not bytes or not raw or len(raw) > LEAF_PATH_LIMIT
                    or b"\0" in raw or b"//" in raw or raw.endswith(b"/")):
                raise LeafClosureError("unsupported-link")
            try:
                raw.decode("utf-8", "strict")
            except UnicodeError:
                raise LeafClosureError("unsupported-link") from None
            if len(raw) != expected[5]:
                raise LeafClosureError("entry-observation-unverified")
            self.verify(parent, name, fd, expected)
            return raw
        finally:
            self.budget.close(fd)

    def row(self, rows, value):
        """Charge canonical row bytes before retaining each bounded private proof record."""
        self.budget.charge("state", leaf_encoded_size(value) + 1, LEAF_STATE_LIMIT, "byte-bound")
        rows.append(value)

    def visit(self, index, raw, fd, relative, depth, rows, files, links):
        """Enumerate complete logical entries in root order, preserving overlapping-root charges and all aliases."""
        if depth > LEAF_DEPTH_LIMIT:
            raise LeafClosureError("depth-bound")
        first = leaf_identity(os.fstat(fd))
        self.trusted(os.fstat(fd), "directory")
        with self.budget.scandir(fd) as entries:
            for entry in entries:
                self.budget.charge("entries", 1, LEAF_ENTRY_LIMIT * 2, "entry-bound")
                self.scan_entries += 1
                if self.scan_entries > LEAF_ENTRY_LIMIT:
                    raise LeafClosureError("entry-bound")
                name = os.fsencode(entry.name)
                full = raw + b"/" + name
                rel = relative + (b"/" if relative else b"") + name
                if len(full) > LEAF_PATH_LIMIT or name in (b"", b".", b"..") or b"/" in name:
                    raise LeafClosureError("entry-bound")
                info = os.stat(name, dir_fd=fd, follow_symlinks=False)
                identity = leaf_identity(info)
                if stat.S_ISDIR(info.st_mode):
                    child, opened = self.directory(fd, name)
                    try:
                        if opened != identity:
                            raise LeafClosureError("entry-observation-unverified")
                        self.row(rows, [index, rel.hex(), "directory", identity])
                        self.visit(index, full, child, rel, depth + 1, rows, files, links)
                        self.verify(fd, name, child, identity)
                    finally:
                        self.budget.close(child)
                elif stat.S_ISREG(info.st_mode):
                    self.trusted(info, "file")
                    pin = self.hash_leaf(fd, name, identity)
                    if full in files and files[full] != (identity, pin):
                        raise LeafClosureError("entry-observation-unverified")
                    self.budget.charge("state", len(full), LEAF_STATE_LIMIT, "byte-bound")
                    files[full] = (identity, pin)
                    self.row(rows, [index, rel.hex(), "file", identity, pin])
                elif stat.S_ISLNK(info.st_mode):
                    self.trusted(info, "link")
                    target = self.read_link(fd, name, identity)
                    if full in links and links[full] != (identity, target):
                        raise LeafClosureError("entry-observation-unverified")
                    self.budget.charge("state", len(full) + len(target), LEAF_STATE_LIMIT, "byte-bound")
                    links[full] = (identity, target)
                    self.row(rows, [index, rel.hex(), "link", identity,
                                    hashlib.sha256(target).hexdigest(), len(target)])
                else:
                    if info.st_uid != 0:
                        raise LeafClosureError("not-root-owned")
                    if info.st_mode & 0o022:
                        raise LeafClosureError("worker-writable")
                    raise LeafClosureError("unsupported-kind")
        if leaf_identity(os.fstat(fd)) != first:
            raise LeafClosureError("entry-observation-unverified")

    def resolve(self, original, files, links):
        """Resolve only inventoried leaf chains, qualifying every raw dot/dot-dot ancestor operation."""
        current, seen, chain, steps = original, set(), [], 0
        for _ in range(LEAF_HOP_LIMIT):
            if current not in links:
                raise LeafClosureError("unsupported-link")
            identity, target = links[current]
            token = (identity[0], identity[1])
            if token in seen:
                raise LeafClosureError("unsupported-link")
            seen.add(token)
            self.budget.charge("hops", 1, LEAF_ENTRY_LIMIT * LEAF_HOP_LIMIT, "entry-bound")
            parent_raw, origin_name = current.rsplit(b"/", 1)
            origin_parent, origin_ancestry = self.absolute_directory(parent_raw or b"/")
            origin_link = None
            try:
                origin_link = self.budget.open(origin_name, os.O_PATH | os.O_NOFOLLOW | os.O_CLOEXEC, origin_parent)
                self.trusted(os.fstat(origin_link), "link")
                self.verify(origin_parent, origin_name, origin_link, identity)
                if self.read_link(origin_parent, origin_name, identity) != target:
                    raise LeafClosureError("entry-observation-unverified")
                parts = ([] if target.startswith(b"/") else current.rsplit(b"/", 1)[0].split(b"/")[1:]) + target.split(b"/")[1 if target.startswith(b"/") else 0:]
                stack, names, opened_generations, ancestry = [self.slash], [], [], [[b"/".hex(), leaf_identity(os.fstat(self.slash))]]
                try:
                    steps += len(parts)
                    if steps > 512:
                        raise LeafClosureError("entry-bound")
                    # Lexical membership is only a pre-I/O refusal; the raw component walk below
                    # independently qualifies dots, parent traversal and every actual directory.
                    candidate = []
                    for component in parts:
                        if component == b".":
                            continue
                        if component == b"..":
                            if not candidate:
                                raise LeafClosureError("unsupported-link")
                            candidate.pop()
                        else:
                            candidate.append(component)
                    if b"/" + b"/".join(candidate) not in files and b"/" + b"/".join(candidate) not in links:
                        raise LeafClosureError("unsupported-link")
                    for component in parts[:-1]:
                        self.budget.charge("work", 1, LEAF_WORK_LIMIT, "entry-bound")
                        if component == b".":
                            continue
                        if component == b"..":
                            if not names:
                                raise LeafClosureError("unsupported-link")
                            self.verify(stack[-2], names[-1], stack[-1], opened_generations[-1])
                            self.budget.close(stack.pop())
                            names.pop()
                            opened_generations.pop()
                            continue
                        if not component or len(names) >= LEAF_DEPTH_LIMIT:
                            raise LeafClosureError("depth-bound")
                        child, opened = self.directory(stack[-1], component)
                        stack.append(child)
                        opened_generations.append(opened)
                        names.append(component)
                        self.row(ancestry, [(b"/" + b"/".join(names)).hex(), opened])
                    leaf = parts[-1]
                    if leaf in (b"", b".", b".."):
                        raise LeafClosureError("unsupported-link")
                    resolved = b"/" + b"/".join([*names, leaf])
                    if len(resolved) > LEAF_PATH_LIMIT:
                        raise LeafClosureError("entry-bound")
                    observed = os.stat(leaf, dir_fd=stack[-1], follow_symlinks=False)
                    if resolved in files:
                        expected, pin = files[resolved]
                        self.trusted(observed, "file")
                        if leaf_identity(observed) != expected or self.hash_leaf(stack[-1], leaf, expected) != pin:
                            raise LeafClosureError("entry-observation-unverified")
                        self.row(chain, [current.hex(), identity, hashlib.sha256(target).hexdigest(),
                                      origin_ancestry, ancestry, resolved.hex(), expected, pin])
                        return chain
                    if resolved not in links:
                        raise LeafClosureError("unsupported-link")
                    self.trusted(observed, "link")
                    expected, text = links[resolved]
                    if leaf_identity(observed) != expected or self.read_link(stack[-1], leaf, expected) != text:
                        raise LeafClosureError("entry-observation-unverified")
                    self.row(chain, [current.hex(), identity, hashlib.sha256(target).hexdigest(),
                                  origin_ancestry, ancestry, resolved.hex(), expected])
                    current = resolved
                finally:
                    try:
                        # Reconcile every retained component before releasing the resolved path.
                        for ordinal in range(1, len(stack)):
                            self.verify(stack[ordinal - 1], names[ordinal - 1], stack[ordinal],
                                        opened_generations[ordinal - 1])
                    finally:
                        for fd in reversed(stack[1:]):
                            self.budget.close(fd)
            finally:
                try:
                    if origin_link is not None:
                        self.verify(origin_parent, origin_name, origin_link, identity)
                    fresh_parent, fresh_ancestry = self.absolute_directory(parent_raw or b"/")
                    try:
                        if (fresh_ancestry != origin_ancestry
                                or leaf_identity(os.fstat(fresh_parent)) != leaf_identity(os.fstat(origin_parent))):
                            raise LeafClosureError("entry-observation-unverified")
                    finally:
                        if fresh_parent != self.slash:
                            self.budget.close(fresh_parent)
                finally:
                    if origin_link is not None:
                        self.budget.close(origin_link)
                    if origin_parent != self.slash:
                        self.budget.close(origin_parent)
        raise LeafClosureError("unsupported-link")

    def scan(self, root_rows):
        """Create one complete canonical proof; link targets must already be independent regular members."""
        rows, files, links = [], {}, {}
        count_before = self.budget.entries
        self.scan_entries = 0
        for index, (raw, fd, expected, _) in enumerate(self.roots):
            if leaf_identity(os.fstat(fd)) != expected:
                raise LeafClosureError("entry-observation-unverified")
            self.visit(index, raw, fd, b"", 0, rows, files, links)
        count = self.budget.entries - count_before
        if (not 1 <= count <= LEAF_ENTRY_LIMIT
                or not any(raw + b"/os.py" in files or raw + b"/os.py" in links
                           for raw, *_ in self.roots)):
            raise LeafClosureError("entry-observation-unverified")
        # Resolve each logical alias row, even when overlapping roots share its spelling.
        for row in tuple(rows):
            if row[2] == "link":
                original = self.roots[row[0]][0] + b"/" + bytes.fromhex(row[1])
                self.row(rows, [row[0], row[1], "resolution", self.resolve(original, files, links)])
        proof = [LEAF_PROOF_FORMAT, root_rows, sorted(rows)]
        if leaf_encoded_size(proof) + 1 > LEAF_STATE_LIMIT:
            raise LeafClosureError("byte-bound")
        raw = leaf_canonical(proof)
        if len(raw) > LEAF_STATE_LIMIT:
            raise LeafClosureError("byte-bound")
        return raw, count, files

    def run(self):
        """Re-enumerate complete membership and bytes with the same budget, then close every held handle."""
        try:
            self.budget.check()
            if (sys.platform != "linux" or not hasattr(os, "O_PATH")
                    or os.open not in os.supports_dir_fd or os.stat not in os.supports_dir_fd
                    or os.readlink not in os.supports_dir_fd or os.scandir not in os.supports_fd
                    or os.stat not in os.supports_follow_symlinks):
                raise LeafClosureError("entry-observation-unverified")
            self.slash = self.budget.open(b"/", os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)
            self.trusted(os.fstat(self.slash), "directory")
            self.slash_generation = leaf_identity(os.fstat(self.slash))
            self.verify_slash()
            roots = self.qualified_roots()
            first, count, files = self.scan(roots)
            # Every root/absence spelling is independently reopened from held slash.
            old_roots = self.roots
            self.roots = []
            again = self.qualified_roots()
            if roots != again:
                raise LeafClosureError("entry-observation-unverified")
            second, second_count, _ = self.scan(again)
            if first != second or count != second_count:
                raise LeafClosureError("entry-observation-unverified")
            # Reconcile root ancestors/absence after the second full pass; closing an
            # ancestor during traversal does not make a replaced spelling continuous.
            middle_roots = self.roots
            self.roots = []
            final_roots = self.qualified_roots()
            if roots != final_roots:
                raise LeafClosureError("entry-observation-unverified")
            # Retained original and second-pass roots remain independent owned witnesses.
            if any(leaf_identity(os.fstat(fd)) != identity
                   for _, fd, identity, _ in (*old_roots, *middle_roots)):
                raise LeafClosureError("entry-observation-unverified")
            self.verify_slash()
            summary = {"format": LEAF_PROOF_FORMAT,
                       "pin": {"bytes": len(first), "sha256": hashlib.sha256(first).hexdigest()},
                       "entries": count, "charged_bytes": self.budget.bytes, "link_hops": self.budget.hops}
            if not leaf_summary_valid(summary):
                raise LeafClosureError("entry-observation-unverified")
            return summary, files
        finally:
            self.budget.close_all()


def leaf_closure(paths, deadline):
    """Independently execute the fixed closure algorithm; no selected-root or proof-prefix fallback exists."""
    return LeafClosure(paths, deadline).run()


def stdlib_inventory(paths, deadline):
    """Qualify complete stdlib files and in-union leaf aliases; retain the independently comparable private proof."""
    try:
        proof, _files = leaf_closure(paths, deadline)
        return {"pin": proof["pin"], "entries": proof["entries"], "proof": proof}
    except LeafClosureError as error:
        if error.reason == "deadline-expired":
            raise GateError("command-timeout", True, diagnostic=tool_diagnostic(error.phase, error.reason)) from None
        failure = GateError("tool-untrusted", True, diagnostic=tool_diagnostic(error.phase, error.reason))
        failure.link_subcondition = link_rejection_site(error)
        raise failure from None


def capture_tool_version(name, executable, root, deadline):
    """Keep the existing fixed command, regex and hash gate, with status-bound diagnostics for that component only."""
    phase = name + "-version"
    observed = tool_step(phase, command, [str(executable), "-V" if name == "ip" else "--version"], root, min(deadline, time.monotonic() + 10))
    pattern = rb"ip utility, iproute2-([0-9]+(?:\.[0-9]+){1,2}(?:-[A-Za-z0-9.]+)?)(?:, [A-Za-z0-9., -]+)?" if name == "ip" else rb"Sudo version ([0-9]+(?:\.[0-9]+){1,2}(?:p[0-9]+)?)"
    line = observed["output"].split(b"\n", 1)[0]
    match = re.fullmatch(pattern, line)
    if observed["failure"] is not None or observed["exit_code"] != 0 or match is None:
        diagnostic = command_diagnostic(phase, observed) if observed["failure"] is not None or observed["exit_code"] != 0 else tool_diagnostic(phase, "version-invalid", observed_exit(observed["exit_code"]))
        raise GateError("tool-unavailable", True, diagnostic=diagnostic)
    return {"pin": tool_step(phase, shared.hash_file, executable), "version": match[1].decode("ascii"), "root_trust": True}


def capture_tools(root, deadline):
    """Qualify fixed existing administrative paths and complete trusted distro stdlib; ordinary PATH evidence stays separate."""
    tool_budget(deadline)
    if sys.platform != "linux" or os.uname().machine != "x86_64":
        raise GateError("unsupported-platform", True)
    if os.getuid() == 0 or os.geteuid() != os.getuid() or os.getegid() != os.getgid():
        raise GateError("privilege-unavailable", True)
    python = tool_step("python-path", administration_tool, Path(_ADMINISTRATIVE_PATHS["python"]))
    resolved = {name: tool_step(name + "-path", administration_tool, Path(_ADMINISTRATIVE_PATHS[name])) for name in ("ip", "sudo")}
    observed = tool_step("python-probe", command, [str(python), "-I", "-S", "-B", "-c", PYTHON_PROBE], root, min(deadline, time.monotonic() + 10))
    if observed["failure"] is not None or observed["exit_code"] != 0:
        raise GateError("tool-unavailable", True, diagnostic=command_diagnostic("python-probe", observed))
    observation = tool_step("python-probe", strict_json, observed["output"], reason="probe-json-invalid", exit_code=observed["exit_code"])
    tool_step("python-probe", closed, observation, ("version", "paths"), reason="probe-shape-invalid", exit_code=observed["exit_code"])
    if type(observation["version"]) is not str or not re.fullmatch(r"3\.[0-9]{1,2}\.[0-9]{1,3}", observation["version"]) or tuple(map(int, observation["version"].split("."))) < (3, 11, 0):
        raise GateError("tool-untrusted", True, diagnostic=tool_diagnostic("python-probe", "version-invalid", observed_exit(observed["exit_code"])))
    inventory = tool_step("stdlib-roots", stdlib_inventory, observation["paths"], deadline)
    tool_budget(deadline)
    tools = {"ordinary": tool_step("ordinary-tools", shared.tool_versions, root, max(0.1, min(10, deadline - time.monotonic()))),
        "privileged_python": {"pin": tool_step("python-path", shared.hash_file, python), "version": observation["version"], "root_trust": True,
            "stdlib_pin": inventory["pin"], "stdlib_entries": inventory["entries"]}}
    for name, executable in resolved.items():
        tools[name] = tool_step(name + "-version", capture_tool_version, name, executable, root, deadline)
    if any(tools["ordinary"].get(name) is None for name in ("cargo", "rustc", "rust_host")):
        raise GateError("tool-unavailable", True, diagnostic=tool_diagnostic("ordinary-tools", "required-identity-missing"))
    tool_budget(deadline)
    return tools, {"python": python, **resolved, "stdlib_proof": inventory["proof"],
                   "interpreter_pin": tools["privileged_python"]["pin"]}


def capture_identity(root, forge, timeout):
    """Supplement unchanged shared commit-object and checkout inputs with this slice's additional source inputs."""
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
        plan = {"schema": "forge.os-denial-private-plan/2", "root": str(root), "forge": str(forge),
            "output_dir": private, "ip": str(paths["ip"]), "python": str(paths["python"]),
            "target_uid": os.getuid(), "target_gid": os.getgid(), "deadline_monotonic_ns": int((deadline - 5) * 1000000000),
            "source_pins": {name: identity["inputs"][name] for name in SOURCE_KEYS}, "release_pin": identity["provided_release_binary"],
            "stdlib_proof": paths["stdlib_proof"], "interpreter_pin": paths["interpreter_pin"]}
        if not leaf_summary_valid(plan["stdlib_proof"]):
            raise GateError("source-invalid")
        pin(plan["interpreter_pin"])
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
           checkout_kind="local", requested_head=None, requested_base=None, link_diagnostic_published=None):
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
        "status": "failed", "failure": "execution-unverified", "diagnostic": None, "identity": None, "checkout": None, "tools": None,
        "build": {"outcome": build_outcome, "profile": "release", "features": "default", "locked": True, "offline": True, "binding": "workflow-step-assertion"},
        "input_stability": "unverified", "tool_stability": "unverified",
        "producer": {"status": "not-run", "exit_code": None, "failure": None, "receipt": None, "receipt_pin": None},
        "cleanup": {"state": "verified-not-created", "forced": False},
        "attempted_egress": {"state": "unmeasured", "count": None}, "pending_gates": list(PENDING)}
    dispatched = False
    before = None
    tools = None
    paths = None
    link_subcondition = None
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
        link_subcondition = error.link_subcondition
        retain_tool_diagnostic(receipt, error)
        receipt.update(status="incomplete" if error.incomplete and not dispatched else "failed", failure=error.code)
    except Exception as error:
        retain_tool_diagnostic(receipt, error)
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
    except Exception as error:
        retain_tool_diagnostic(receipt, error)
        receipt.update(status="failed", failure="verification-input-invalid")
    if len(shared.canonical_bytes(receipt)) > MAX_CAPTURE:
        raise ValueError("outer bound")
    shared.atomic_receipt(destination / OUTPUT, receipt)
    publish_link_diagnostic(root, destination, receipt, link_subcondition, deadline, link_diagnostic_published)
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
            requested_head=args.requested_head, requested_base=args.requested_base,
            link_diagnostic_published=emit_link_diagnostic_publication_flag)
    except Exception:
        print("OS-denial prerequisite receipt could not be published.", file=sys.stderr)
        return 1
    print("OS-denial prerequisite: " + value["status"] + "; wider gates remain open.")
    return {"passed": 0, "failed": 1, "incomplete": 2}[value["status"]]


if __name__ == "__main__":
    sys.exit(main())
