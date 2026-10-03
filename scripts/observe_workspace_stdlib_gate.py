#!/usr/bin/env python3
"""Observe one bounded stdlib stat rejection in an ordinary checkout; never run the native experiment or qualify inventory."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import selectors
import stat
import subprocess
import sys
import tempfile
import time

SCHEMA = "forge.stdlib-gate-observation/1"
OUTPUT = "stdlib-gate-observation.json"
MAX_SIDECAR = 2048
MAX_CAPTURE = 262144
MAX_ENTRIES = 20000
MAX_STDLIB_BYTES = 268435456
SOURCE_LIMIT = 131072
BUDGET_SECONDS = 600
INPUT_KEYS = ("scripts/observe_workspace_stdlib_gate.py", "scripts/verify_workspace_os_denial.py",
              "scripts/test_workspace_os_denial.py", ".github/workflows/workspace-verification.yml")
EXPECTED_PROTECTED = {
    "scripts/verify_workspace_os_denial.py": {"bytes": 44351, "sha256": "1b15a062772ec6b0ac8e584992aa9201ede8e35c4704100d37c4e8e0af9dd30c"},
    "scripts/test_workspace_os_denial.py": {"bytes": 68516, "sha256": "b32293bc02c8fa4cff93fabb91798a70b9c51bd57a391c3cdce8cb5754ece86c"},
}
ADMINISTRATIVE_PATHS = {"python": "/usr/bin/python3", "ip": "/usr/bin/ip", "sudo": "/usr/bin/sudo"}
PATH_REASONS = ("missing", "not-absolute", "not-root-owned", "worker-writable", "unsupported-link",
               "not-directory", "not-regular", "link-bound", "not-executable", "path-observation-unverified")
COMMAND_REASONS = ("command-failed", "command-timeout", "output-bound", "command-cleanup-unverified")
PYTHON_PROBE = ("import json,sys;print(json.dumps({'version':'.'.join(map(str,sys.version_info[:3])),"
                "'paths':sys.path},sort_keys=True,separators=(',',':')))")
OBJECT_FIELDS = ("source_predicate", "root_index", "object_id", "object_kind", "uid_is_root", "mode_022_bits", "object_stat_stable")
PREDICATES = ("entry-uid", "entry-mode-022", "entry-kind", "root-ancestor-uid", "root-ancestor-mode-022", "root-ancestor-kind")

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


class ObservationStop(ValueError):
    """Carry only the closed diagnostic record while private exceptions and filesystem identities stay unprinted."""

    def __init__(self, value):
        """Freeze a sidecar value without attaching raw paths, command output or exception text."""
        self.value = value
        super().__init__("observation unavailable")


def canonical_bytes(value):
    """Encode one sorted compact ASCII JSON value with LF; this is evidence serialization, not qualification."""
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True) + "\n").encode()


def empty_observation(outcome="unavailable", phase=None, reason=None, stable=None):
    """Create the closed null observation without inventing object, tool, native or attempted-egress facts."""
    return {"schema_version": SCHEMA, "outcome": outcome, "phase": phase, "reason": reason,
            **{name: None for name in OBJECT_FIELDS}, "object_stat_stable": stable, "identity": None}


def unavailable(phase=None, reason=None, stable=None):
    """Stop the diagnostic with a fixed optional pair, never expose the underlying failure details."""
    raise ObservationStop(empty_observation(phase=phase, reason=reason, stable=stable))


def budget(deadline):
    """Reject equality/expiry without renewing the absolute budget or dispatching another command."""
    if time.monotonic() >= deadline:
        unavailable("qualification-budget", "deadline-expired")


def metadata(info):
    """Compare the full original lstat identity while retaining no raw filesystem identifiers in the sidecar."""
    return (info.st_dev, info.st_ino, info.st_mode, info.st_uid, info.st_gid,
            info.st_size, info.st_mtime_ns, info.st_ctime_ns)


def object_kind(mode):
    """Name only the coarse original stat kind without following a link or reading any file content."""
    if stat.S_ISREG(mode): return "regular"
    if stat.S_ISDIR(mode): return "directory"
    if stat.S_ISLNK(mode): return "symlink"
    return "other"


def reject_object(path, info, phase, reason, predicate, root_index, root, deadline):
    """Record the first original stat predicate only after one bounded identity recheck; replacement revokes object claims."""
    if root_index is None or root is None:
        unavailable(phase, reason)
    budget(deadline)
    try:
        checked = Path(path).lstat()
    except Exception:
        unavailable(phase, "entry-observation-unverified" if phase == "stdlib-entry" else "path-observation-unverified")
    if metadata(info) != metadata(checked):
        unavailable(phase, "entry-observation-unverified" if phase == "stdlib-entry" else "path-observation-unverified", False)
    budget(deadline)
    private_id = {"root": str(root), "object": str(path), "dev": info.st_dev,
                  "ino": info.st_ino, "ctime_ns": info.st_ctime_ns}
    value = empty_observation("rejected", phase, reason, True)
    value.update(source_predicate=predicate, root_index=root_index,
                 object_id=hashlib.sha256(canonical_bytes(private_id)).hexdigest(),
                 object_kind=object_kind(info.st_mode), uid_is_root=info.st_uid == 0,
                 mode_022_bits=info.st_mode & 0o022)
    raise ObservationStop(value)


def root_trusted(path, directory, phase, root_index=None, root=None, deadline=None):
    """Mirror root/ancestor UID, write-bit and kind ordering; an administrative failure has no stdlib object claim."""
    path = Path(path)
    if not path.is_absolute():
        unavailable(phase, "not-absolute" if phase in ("python-path", "ip-path", "sudo-path") else "root-shape-invalid")
    item = path
    while True:
        budget(deadline)
        info = item.lstat()
        if info.st_uid != 0:
            reject_object(item, info, phase, "not-root-owned", "root-ancestor-uid", root_index, root, deadline)
        if info.st_mode & 0o022:
            reject_object(item, info, phase, "worker-writable", "root-ancestor-mode-022", root_index, root, deadline)
        if stat.S_ISLNK(info.st_mode):
            reject_object(item, info, phase, "unsupported-link", "root-ancestor-kind", root_index, root, deadline)
        if item == path:
            if directory and not stat.S_ISDIR(info.st_mode) or not directory and not stat.S_ISREG(info.st_mode):
                reject_object(item, info, phase, "not-directory" if directory else "not-regular", "root-ancestor-kind", root_index, root, deadline)
        elif not stat.S_ISDIR(info.st_mode):
            reject_object(item, info, phase, "not-directory", "root-ancestor-kind", root_index, root, deadline)
        if item.parent == item:
            break
        item = item.parent
    return path


def administrative_tool(path, phase, deadline=None):
    """Mirror existing fixed-tool symlink/ancestor qualification without executing ip, sudo or any fallback."""
    budget(deadline)
    path = Path(path)
    if not path.exists(): unavailable(phase, "missing")
    link = path.lstat()
    if link.st_uid != 0: unavailable(phase, "not-root-owned")
    if link.st_mode & 0o022 and not stat.S_ISLNK(link.st_mode): unavailable(phase, "worker-writable")
    root_trusted(path.parent, True, phase, deadline=deadline)
    current = path
    for _link in range(16):
        budget(deadline)
        root_trusted(current.parent, True, phase, deadline=deadline)
        info = current.lstat()
        if info.st_uid != 0: unavailable(phase, "not-root-owned")
        if not stat.S_ISLNK(info.st_mode): break
        target = Path(os.readlink(current))
        current = target if target.is_absolute() else current.parent / target
    else:
        unavailable(phase, "link-bound")
    actual = current.resolve(strict=True)
    root_trusted(actual, False, phase, deadline=deadline)
    if not os.access(actual, os.X_OK): unavailable(phase, "not-executable")
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



def strict_json(raw):
    """Decode bounded duplicate-safe finite JSON with the original postdecode depth cap; no allocation-security claim."""
    if type(raw) is not bytes or not raw or len(raw) > MAX_CAPTURE:
        unavailable("python-probe", "output-bound")
    def pairs(items):
        """Reject decoded duplicate keys before a dict can erase a probe defect."""
        value = {}
        for key, item in items:
            if key in value: unavailable("python-probe", "probe-json-invalid")
            value[key] = item
        return value
    def constant(_item):
        """Reject nonfinite tokens without exposing raw input in an exception or sidecar."""
        unavailable("python-probe", "probe-json-invalid")
    try:
        value = json.loads(raw, object_pairs_hook=pairs, parse_constant=constant)
    except ObservationStop:
        raise
    except Exception:
        unavailable("python-probe", "probe-json-invalid")
    def depth(item, level):
        """Retain the original decoded depth limit without equating byte caps with predecode allocation confinement."""
        if level > 16: unavailable("python-probe", "output-bound")
        if isinstance(item, dict):
            for child in item.values(): depth(child, level + 1)
        elif isinstance(item, list):
            for child in item: depth(child, level + 1)
    depth(value, 0)
    return value


def probe_paths(root, deadline):
    """Run only the fixed isolated trusted distro-Python sys.path/version probe and discard all private command bytes afterward."""
    budget(deadline)
    python = administrative_tool(Path(ADMINISTRATIVE_PATHS["python"]), "python-path", deadline)
    for name in ("ip", "sudo"):
        administrative_tool(Path(ADMINISTRATIVE_PATHS[name]), name + "-path", deadline)
    budget(deadline)
    observed = command([str(python), "-I", "-S", "-B", "-c", PYTHON_PROBE], root, min(deadline, time.monotonic() + 10))
    if observed["failure"] is not None or observed["exit_code"] != 0:
        reason = {"command-timeout": "command-timeout", "output-bound": "output-bound", "cleanup-unverified": "command-cleanup-unverified"}.get(observed["failure"], "command-failed")
        unavailable("python-probe", reason)
    value = strict_json(observed["output"])
    if type(value) is not dict or set(value) != {"version", "paths"}:
        unavailable("python-probe", "probe-shape-invalid")
    if type(value["version"]) is not str or not re.fullmatch(r"3\.[0-9]{1,2}\.[0-9]{1,3}", value["version"]) or tuple(map(int, value["version"].split("."))) < (3, 11, 0):
        unavailable("python-probe", "version-invalid")
    budget(deadline)
    return value["paths"]


def stdlib_walk(paths, deadline):
    """Stream original stat gates/caps in order, omitting content hashes and yielding no inventory or qualification result."""
    if type(paths) is not list or not 2 <= len(paths) <= 8 or any(type(path) is not str or not Path(path).is_absolute() for path in paths) or len(paths) != len(set(paths)):
        unavailable("stdlib-roots", "root-shape-invalid")
    roots = []
    for value in paths:
        budget(deadline)
        path = Path(value)
        if not path.exists():
            if path.suffix != ".zip": unavailable("stdlib-roots", "missing-root")
            root_trusted(path.parent, True, "stdlib-roots", len(roots), path, deadline)
            continue
        root_trusted(path, True, "stdlib-roots", len(roots), path, deadline)
        roots.append(path)
    if len(roots) < 2 or not any(path.name == "lib-dynload" for path in roots):
        unavailable("stdlib-roots", "dynload-missing")
    count = 0
    total = 0
    def visit(root, directory, level, index):
        """Check count before each original stat and size before descent; no regular file content is opened."""
        nonlocal count, total
        if level > 32: unavailable("stdlib-entry", "depth-bound")
        root_trusted(directory, True, "stdlib-entry", index, root, deadline)
        with os.scandir(directory) as entries:
            for entry in entries:
                budget(deadline)
                count += 1
                if count > MAX_ENTRIES: unavailable("stdlib-entry", "entry-bound")
                path = Path(entry.path)
                try:
                    info = entry.stat(follow_symlinks=False)
                except Exception:
                    unavailable("stdlib-entry", "entry-observation-unverified")
                if info.st_uid != 0:
                    reject_object(path, info, "stdlib-entry", "not-root-owned", "entry-uid", index, root, deadline)
                if info.st_mode & 0o022:
                    reject_object(path, info, "stdlib-entry", "worker-writable", "entry-mode-022", index, root, deadline)
                if stat.S_ISLNK(info.st_mode):
                    reject_object(path, info, "stdlib-entry", "unsupported-link", "entry-kind", index, root, deadline)
                if stat.S_ISDIR(info.st_mode):
                    visit(root, path, level + 1, index)
                elif stat.S_ISREG(info.st_mode):
                    total += info.st_size
                    if total > MAX_STDLIB_BYTES: unavailable("stdlib-entry", "byte-bound")
                else:
                    reject_object(path, info, "stdlib-entry", "unsupported-kind", "entry-kind", index, root, deadline)
    for index, root in enumerate(roots): visit(root, root, 0, index)
    if count == 0: unavailable("stdlib-entry", "entry-observation-unverified")
    budget(deadline)


def read_source(path):
    """Hash only fixed repository-source bytes from a bounded unchanged regular descriptor, never stdlib content."""
    path = Path(path); before = path.lstat()
    if not stat.S_ISREG(before.st_mode) or not 0 < before.st_size <= SOURCE_LIMIT:
        unavailable()
    descriptor = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | os.O_NONBLOCK)
    try:
        opened = os.fstat(descriptor)
        if metadata(opened) != metadata(before): unavailable()
        chunks = []; total = 0
        while total <= SOURCE_LIMIT:
            chunk = os.read(descriptor, min(8192, SOURCE_LIMIT + 1 - total))
            if not chunk: break
            total += len(chunk); chunks.append(chunk)
        if total != opened.st_size or total > SOURCE_LIMIT or metadata(opened) != metadata(os.fstat(descriptor)) or metadata(opened) != metadata(path.lstat()):
            unavailable()
        raw = b"".join(chunks)
        return {"bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}
    finally:
        os.close(descriptor)


def capture_identity(root, requested_commit, tested_commit, deadline):
    """Bind ordinary checkout HEAD and four source pins without claiming loaded bytes, privileged trust or authenticated parents."""
    budget(deadline)
    if any(type(value) is not str or not re.fullmatch(r"(?:[0-9a-f]{40}|[0-9a-f]{64})", value) for value in (requested_commit, tested_commit)):
        unavailable()
    observed = command(["git", "rev-parse", "--verify", "HEAD"], root, min(deadline, time.monotonic() + 10))
    if observed["failure"] is not None or observed["exit_code"] != 0:
        unavailable()
    if not re.fullmatch(rb"(?:[0-9a-f]{40}|[0-9a-f]{64})\n", observed["output"]): unavailable()
    actual = observed["output"][:-1].decode("ascii")
    if actual != tested_commit: unavailable()
    pins = {}
    for name in INPUT_KEYS:
        budget(deadline)
        pins[name] = read_source(Path(root) / name)
    for name, expected in EXPECTED_PROTECTED.items():
        if pins[name] != expected: unavailable()
    budget(deadline)
    return {"requested_commit": requested_commit, "tested_commit": actual, "before_after": "unverified", "inputs": pins}


def validate_observation(value):
    """Reject extra fields, guessed object facts and malformed source identity before any sidecar can be published."""
    if type(value) is not dict or set(value) != set(empty_observation()): raise ValueError("invalid observation")
    if value["schema_version"] != SCHEMA or value["outcome"] not in ("rejected", "unavailable", "no-rejection"): raise ValueError("invalid observation")
    phase, reason = value["phase"], value["reason"]
    if (phase is None) != (reason is None) or phase is not None and (type(phase) is not str or phase not in TOOL_PHASE_REASONS or type(reason) is not str or reason not in TOOL_PHASE_REASONS[phase]): raise ValueError("invalid observation")
    identity = value["identity"]
    if identity is not None:
        if type(identity) is not dict or set(identity) != {"requested_commit", "tested_commit", "before_after", "inputs"}: raise ValueError("invalid identity")
        if any(type(identity[k]) is not str or not re.fullmatch(r"(?:[0-9a-f]{40}|[0-9a-f]{64})", identity[k]) for k in ("requested_commit", "tested_commit")): raise ValueError("invalid identity")
        if identity["before_after"] not in ("unchanged", "changed", "unverified") or type(identity["inputs"]) is not dict or set(identity["inputs"]) != set(INPUT_KEYS): raise ValueError("invalid identity")
        for row in identity["inputs"].values():
            if type(row) is not dict or set(row) != {"bytes", "sha256"} or type(row["bytes"]) is not int or not 1 <= row["bytes"] <= SOURCE_LIMIT or type(row["sha256"]) is not str or not re.fullmatch(r"[0-9a-f]{64}", row["sha256"]): raise ValueError("invalid identity")
        if any(identity["inputs"][name] != expected for name, expected in EXPECTED_PROTECTED.items()): raise ValueError("invalid identity")
    if value["outcome"] == "rejected":
        if identity is None or identity["before_after"] != "unchanged" or phase not in ("stdlib-roots", "stdlib-entry"): raise ValueError("unbound object")
        if value["source_predicate"] not in PREDICATES or type(value["root_index"]) is not int or not 0 <= value["root_index"] <= 7 or type(value["object_id"]) is not str or not re.fullmatch(r"[0-9a-f]{64}", value["object_id"]): raise ValueError("invalid object")
        if value["object_kind"] not in ("regular", "directory", "symlink", "other") or type(value["uid_is_root"]) is not bool or type(value["mode_022_bits"]) is not int or value["mode_022_bits"] not in (0, 2, 16, 18) or value["object_stat_stable"] is not True: raise ValueError("invalid object")
        predicate = value["source_predicate"]
        if predicate.startswith("entry-") and phase != "stdlib-entry": raise ValueError("invalid predicate phase")
        if predicate == "entry-kind" and reason not in ("unsupported-link", "unsupported-kind"): raise ValueError("invalid entry kind")
        if predicate == "root-ancestor-kind" and reason not in ("unsupported-link", "not-directory"): raise ValueError("invalid ancestor kind")
        if predicate.endswith("uid") and (reason != "not-root-owned" or value["uid_is_root"] is not False): raise ValueError("invalid predicate")
        if predicate.endswith("mode-022") and (reason != "worker-writable" or value["uid_is_root"] is not True or value["mode_022_bits"] == 0): raise ValueError("invalid predicate")
        if predicate.endswith("kind") and (reason not in ("unsupported-link", "not-directory", "not-regular", "unsupported-kind") or value["uid_is_root"] is not True or value["mode_022_bits"] != 0): raise ValueError("invalid predicate")
        if predicate.endswith("kind") and ((reason == "unsupported-link" and value["object_kind"] != "symlink") or (reason == "not-directory" and value["object_kind"] not in ("regular", "other")) or (reason == "not-regular" and value["object_kind"] == "regular") or (reason == "unsupported-kind" and value["object_kind"] != "other")): raise ValueError("invalid kind predicate")
    else:
        if any(value[name] is not None for name in OBJECT_FIELDS if name != "object_stat_stable") or value["object_stat_stable"] is not None and value["object_stat_stable"] is not False: raise ValueError("invented object")
        if value["outcome"] == "no-rejection" and (phase is not None or identity is None or identity["before_after"] != "unchanged" or value["object_stat_stable"] is not None): raise ValueError("unbound no-rejection")
    if len(canonical_bytes(value)) > MAX_SIDECAR: raise ValueError("sidecar bound")
    return value


def observe(root, requested_commit, tested_commit):
    """Observe one first rejection, then revoke all object claims if ordinary source/HEAD identity cannot be re-established."""
    deadline = time.monotonic() + BUDGET_SECONDS
    value = empty_observation(); before = None
    try:
        before = capture_identity(root, requested_commit, tested_commit, deadline)
        if sys.platform != "linux" or os.uname().machine != "x86_64" or os.getuid() == 0 or os.geteuid() != os.getuid() or os.getegid() != os.getgid(): unavailable()
        paths = probe_paths(root, deadline)
        stdlib_walk(paths, deadline)
        value = empty_observation("no-rejection")
    except ObservationStop as error:
        value = error.value
    except Exception:
        value = empty_observation()
    if before is not None:
        try:
            after = capture_identity(root, requested_commit, tested_commit, deadline)
            before["before_after"] = "unchanged" if before == after else "changed"
            if before["before_after"] != "unchanged": value = empty_observation()
        except Exception:
            before["before_after"] = "unverified"
            value = empty_observation()
        value["identity"] = before
    return validate_observation(value)


def publish(destination, value):
    """Publish one private no-replacement canonical sidecar; cleanup failure exits nonzero even if a linked file remains."""
    validate_observation(value)
    destination = Path(destination)
    destination.mkdir(mode=0o700, parents=True, exist_ok=False)
    descriptor, temporary = tempfile.mkstemp(prefix=".observation-", dir=destination)
    try:
        with os.fdopen(descriptor, "wb") as output:
            output.write(canonical_bytes(value)); output.flush(); os.fsync(output.fileno())
        os.link(temporary, destination / OUTPUT)
    finally:
        os.unlink(temporary)


def main(argv=None):
    """Report only diagnostic availability with closed exit classes; no passed native, inventory or cause result exists."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--requested-commit", required=True)
    parser.add_argument("--tested-commit", required=True)
    args = parser.parse_args(argv)
    try:
        value = observe(Path(__file__).resolve().parents[1], args.requested_commit, args.tested_commit)
        publish(args.output_dir, value)
    except Exception:
        print("Stdlib gate observation unavailable.", file=sys.stderr)
        return 1
    print("Stdlib gate observation: " + value["outcome"] + "; qualification remains incomplete.")
    return 0 if value["outcome"] == "rejected" else 2


if __name__ == "__main__":
    sys.exit(main())
