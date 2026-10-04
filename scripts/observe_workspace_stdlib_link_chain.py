#!/usr/bin/env python3
"""Observe one freshly rejected Linux stdlib link without changing its refusal.

This ordinary-user sidecar never reads target file contents or invokes native,
sudo, ip or Forge. Nullable facts cover only the actual observed prefix. They
do not establish stdlib closure, loaded bytes, target admission or acceptance.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import sys
import time
import types

sys.dont_write_bytecode = True
SCHEMA = "forge.stdlib-link-chain-observation/1"
OUTPUT = "stdlib-link-chain-observation.json"
FLAG = b"stdlib_link_chain_published=true\n"
MAX_SIDECAR = 2048
MAX_SOURCE = 131072
MAX_PATH = 4096
MAX_HOPS = 16
MAX_COMPONENTS = 512
MAX_FDS = 64
MAX_RECORDS = 512
MAX_PROOF = 1048576
INPUT_KEYS = ("scripts/observe_workspace_stdlib_link_chain.py",
              "scripts/observe_workspace_stdlib_gate.py",
              "scripts/verify_workspace_os_denial.py",
              "scripts/test_workspace_os_denial.py",
              ".github/workflows/workspace-verification.yml")
EXPECTED_GATE = {"bytes": 28184, "sha256": "01b209a586c9399ed5ec48b0023412600d7d2fc3a8771751f4211d8cd5ec8b4a"}
EXPECTED_PROTECTED = {
    "scripts/verify_workspace_os_denial.py": {"bytes": 44362, "sha256": "5919a15920b60e8dcf4cdb063c1120a4d0b3077ae8099c8de8768651bdf2644d"},
    "scripts/test_workspace_os_denial.py": {"bytes": 68516, "sha256": "b32293bc02c8fa4cff93fabb91798a70b9c51bd57a391c3cdce8cb5754ece86c"},
    "scripts/observe_workspace_stdlib_gate.py": EXPECTED_GATE,
}
OBJECT_FIELDS = ("root_index", "entry_id", "first_link_form", "hops_observed",
                 "cycle_detected", "chain_id", "target_id", "target_kind",
                 "target_uid_is_root", "target_mode_022_bits", "target_in_roots",
                 "parents_root_owned", "parents_nonwritable", "links_root_owned",
                 "object_stat_stable")
FAULTS = frozenset(("target-observed", "entry-not-selected", "source-unverified",
    "unsupported-platform", "unsupported-primitive", "deadline-expired",
    "entry-bound", "depth-bound", "byte-bound", "path-bound", "hop-bound",
    "component-bound", "descriptor-bound", "target-missing", "target-outside-roots",
    "target-not-root-owned", "target-worker-writable", "target-kind-unsupported",
    "ancestor-not-root-owned", "ancestor-worker-writable", "ancestor-kind-unsupported",
    "link-not-root-owned", "intermediate-link-unsupported", "cycle-observed",
    "identity-changed", "object-observation-unverified", "descriptor-close-unverified"))


class Stop(Exception):
    """Carry a fixed unavailable code without raw filesystem or exception text."""

    def __init__(self, code, changed=False):
        """Validate a closed fault and actual replacement indicator before storage."""
        if code not in FAULTS or type(changed) is not bool:
            raise ValueError("closed observation fault")
        self.code, self.changed = code, changed
        super().__init__(code)


def canonical(value):
    """Return compact ASCII JSON plus LF; private strings never become public facts."""
    return (json.dumps(value, sort_keys=True, separators=(",", ":"),
                       ensure_ascii=True, allow_nan=False) + "\n").encode("ascii")


def digest(value):
    """Hash private metadata/link records, never arbitrary terminal file contents."""
    return hashlib.sha256(canonical(value)).hexdigest()


def metadata(info):
    """Retain exact native generation/ownership/mode/size/link-count metadata."""
    names = ("st_dev", "st_ino", "st_mode", "st_uid", "st_gid", "st_size",
             "st_mtime_ns", "st_ctime_ns", "st_nlink")
    values = tuple(getattr(info, name) for name in names)
    if any(type(value) is not int for value in values) or info.st_size < 0:
        raise Stop("object-observation-unverified")
    return values


def empty(code="object-observation-unverified", changed=False):
    """Return unobserved object fields; unknown is never a false or zero substitute."""
    value = {"schema_version": SCHEMA, "outcome": "unavailable", "first_fault": code,
             "identity": None, **dict.fromkeys(OBJECT_FIELDS)}
    if changed:
        value["object_stat_stable"] = False
    return value


def budget(deadline, now=time.monotonic):
    """Fence each new operation with one absolute deadline without claiming preemption."""
    if now() >= deadline:
        raise Stop("deadline-expired")


def bounded_path(value):
    """Admit a bounded absolute UTF8 spelling without resolving or erasing ancestry."""
    if type(value) is not str or not value.startswith("/"):
        raise Stop("path-bound")
    try:
        raw = value.encode("utf-8", "strict")
    except UnicodeError:
        raise Stop("path-bound") from None
    if not 1 <= len(raw) <= MAX_PATH or any(ord(c) < 32 for c in value):
        raise Stop("path-bound")
    return value


class Inspector:
    """Hold no-follow metadata handles and monotonically bounded private proof rows."""

    def __init__(self, deadline, *, api=os, now=time.monotonic):
        """Require explicit Linux descriptor flags instead of permissive zero fallbacks."""
        self.api, self.now, self.deadline = api, now, deadline
        for name in ("O_PATH", "O_NOFOLLOW", "O_DIRECTORY", "O_CLOEXEC"):
            if type(getattr(api, name, None)) is not int or getattr(api, name) == 0:
                raise Stop("unsupported-primitive")
        if not callable(getattr(api, "readlink", None)):
            raise Stop("unsupported-primitive")
        self.fds, self.records, self.absences, self.hops, self.roots = set(), {}, {}, [], []
        self.operations, self.proof_bytes = 0, 0
        self.close_failed = False
        self.failed_closes = set()
        self.facts = dict.fromkeys(OBJECT_FIELDS)

    def fence(self):
        """Charge component work before each open or metadata/link re-observation."""
        budget(self.deadline, self.now)
        if self.close_failed:
            raise Stop("descriptor-close-unverified")
        if self.operations >= MAX_COMPONENTS:
            raise Stop("component-bound")
        self.operations += 1

    def open(self, name, parent=None, directory=False):
        """Reserve a descriptor slot before no-follow O_PATH opening without content IO."""
        self.fence()
        if len(self.fds) >= MAX_FDS:
            raise Stop("descriptor-bound")
        flags = self.api.O_PATH | self.api.O_NOFOLLOW | self.api.O_CLOEXEC
        if directory:
            flags |= self.api.O_DIRECTORY
        fd = self.api.open(name, flags, dir_fd=parent)
        self.fds.add(fd)
        return fd

    def close(self, fd):
        """Keep failed descriptor ownership recorded and still allow independent closes."""
        if fd in self.failed_closes:
            return  # Unknown close state is not permission to retry a possibly recycled number.
        try:
            self.api.close(fd)
        except BaseException:
            self.close_failed = True
            self.failed_closes.add(fd)
        else:
            self.fds.discard(fd)

    def close_all(self):
        """Attempt every remaining owned reference; unknown closure revokes facts."""
        for fd in list(self.fds):
            self.close(fd)
        return not self.close_failed and not self.fds

    def remember(self, path, info, text=None):
        """Precharge complete encoded private metadata before retaining a distinct row."""
        row = {"path": path, "metadata": list(metadata(info))}
        if text is not None:
            row["link_bytes_hex"] = text.hex()
        previous = self.records.get(path)
        if previous is not None:
            if previous["metadata"] != row["metadata"] or (text is not None and
                    previous.get("link_bytes_hex", row.get("link_bytes_hex")) != row["link_bytes_hex"]):
                raise Stop("identity-changed", True)
            if text is None or "link_bytes_hex" in previous:
                return
        size = len(canonical(row))
        old_size = len(canonical(previous)) if previous is not None else 0
        if previous is None and len(self.records) + len(self.absences) >= MAX_RECORDS:
            raise Stop("component-bound")
        if self.proof_bytes + size - old_size + len(canonical(self.hops)) > MAX_PROOF:
            raise Stop("byte-bound")
        self.proof_bytes += size - old_size
        self.records[path] = row

    def absent(self, path):
        """Precharge a typed missing-component proof without claiming a present target."""
        if path in self.absences:
            return
        row = {"path": path, "absent": True}
        size = len(canonical(row))
        if len(self.records) + len(self.absences) >= MAX_RECORDS:
            raise Stop("component-bound")
        if self.proof_bytes + size + len(canonical(self.hops)) > MAX_PROOF:
            raise Stop("byte-bound")
        self.proof_bytes += size
        self.absences[path] = row

    def directory(self, info):
        """Record actual checked-parent prefix facts and stop before unsafe descent."""
        if not stat.S_ISDIR(info.st_mode):
            raise Stop("ancestor-kind-unsupported")
        if info.st_uid != 0:
            self.facts["parents_root_owned"] = False
            raise Stop("ancestor-not-root-owned")
        if self.facts["parents_root_owned"] is None:
            self.facts["parents_root_owned"] = True
        if info.st_mode & 0o022:
            self.facts["parents_nonwritable"] = False
            raise Stop("ancestor-worker-writable")
        if self.facts["parents_nonwritable"] is None:
            self.facts["parents_nonwritable"] = True

    def walk(self, path, *, retain=True):
        """Open every original component including dotdot parents without automatic links."""
        bounded_path(path)
        components = path.split("/")[1:]
        if len(components) > MAX_COMPONENTS:
            raise Stop("component-bound")
        stack, names, observations = [], [], {}
        try:
            first = self.open("/", directory=True)
            stack.append(first)
            root_info = self.api.fstat(first)
            observations["/"] = metadata(root_info)
            if retain:
                self.remember("/", root_info)
            self.directory(root_info)
            for index, component in enumerate(components):
                if component in ("", "."):
                    continue
                if component == "..":
                    if len(stack) > 1:
                        self.close(stack.pop())
                        names.pop()
                    continue
                final = not any(c not in ("", ".") for c in components[index + 1:])
                if not final:
                    self.fence()
                    try:
                        item = self.api.stat(component, dir_fd=stack[-1], follow_symlinks=False)
                    except FileNotFoundError:
                        if retain:
                            self.absent("/" + "/".join(names + [component]))
                        raise
                    if stat.S_ISLNK(item.st_mode):
                        link = self.open(component, stack[-1])
                        try:
                            opened = self.api.fstat(link)
                            if metadata(opened) != metadata(item):
                                raise Stop("identity-changed", True)
                            if retain:
                                self.remember("/" + "/".join(names + [component]), opened)
                        finally:
                            self.close(link)
                        raise Stop("intermediate-link-unsupported")
                try:
                    fd = self.open(component, stack[-1], not final)
                except FileNotFoundError:
                    if retain:
                        self.absent("/" + "/".join(names + [component]))
                    raise
                info = self.api.fstat(fd)
                actual_path = "/" + "/".join(names + [component])
                observations[actual_path] = metadata(info)
                if retain:
                    self.remember(actual_path, info)
                if final:
                    return fd, info, actual_path, observations
                self.directory(info)
                stack.append(fd)
                names.append(component)
            fd = self.open(".", stack[-1], True)
            info = self.api.fstat(fd)
            return fd, info, "/" + "/".join(names), observations
        finally:
            for fd in stack:
                self.close(fd)

    def link_text(self, fd, info):
        """Cap held link stat before readlink and check exact returned bytes and generation."""
        metadata(info)
        if not stat.S_ISLNK(info.st_mode) or not 1 <= info.st_size <= MAX_PATH:
            raise Stop("object-observation-unverified")
        self.fence()
        # Python returns a complete string/bytes result; it is not a bounded-buffer API.
        # A stable nonzero held size is admitted before read; exact returned size follows.
        raw = self.api.readlink(b"", dir_fd=fd)
        if type(raw) is not bytes or len(raw) != info.st_size or not 1 <= len(raw) <= MAX_PATH:
            raise Stop("object-observation-unverified")
        if metadata(self.api.fstat(fd)) != metadata(info):
            raise Stop("identity-changed", True)
        try:
            text = raw.decode("utf-8", "strict")
        except UnicodeError:
            raise Stop("object-observation-unverified") from None
        if any(ord(c) < 32 for c in text):
            raise Stop("path-bound")
        return raw, text

    def qualify_roots(self, roots):
        """Retain actual no-follow root generations without promoting them to library trust."""
        if type(roots) is not list or not 2 <= len(roots) <= 8 or len(roots) != len(set(roots)):
            raise Stop("entry-not-selected")
        for value in roots:
            try:
                fd, info, actual, _observations = self.walk(value)
            except FileNotFoundError:
                if value.endswith(".zip"):
                    continue  # The unchanged discovery gate already checked its parent.
                raise Stop("object-observation-unverified") from None
            try:
                self.directory(info)
                self.roots.append((actual, metadata(info)))
            finally:
                self.close(fd)
        if len(self.roots) < 2 or not any(Path(path).name == "lib-dynload" for path, _meta in self.roots):
            raise Stop("entry-not-selected")

    def membership(self, path, observations):
        """Require an actual matching held ancestor generation rather than lexical prefix alone."""
        for root, expected in self.roots:
            if path == root or path.startswith(root.rstrip("/") + "/"):
                actual = observations.get(root)
                if actual is None or actual != expected:
                    raise Stop("identity-changed", True)
                return True
        return False

    def target(self, path, info, observations):
        """Store only actual held-terminal stat facts; no contents, exact IDs or paths escape."""
        kind = ("regular" if stat.S_ISREG(info.st_mode) else "directory" if stat.S_ISDIR(info.st_mode)
                else "symlink" if stat.S_ISLNK(info.st_mode) else "other")
        self.facts.update(target_id=digest({"path": path, "metadata": metadata(info)}),
                          target_kind=kind, target_uid_is_root=info.st_uid == 0,
                          target_mode_022_bits=info.st_mode & 0o022,
                          target_in_roots=self.membership(path, observations))

    def trace(self, selection, roots):
        """Observe the selected chain prefix and fixed first endpoint without admitting it."""
        self.qualify_roots(roots)
        path, original, index, original_root = selection
        bounded_path(str(path))
        self.facts.update(root_index=index, entry_id=digest({"root": str(original_root),
            "object": str(path), "dev": original.st_dev, "ino": original.st_ino,
            "ctime_ns": original.st_ctime_ns}), hops_observed=0)
        seen = set()
        current = str(path)
        first = True
        while True:
            try:
                fd, info, actual, observations = self.walk(current)
            except FileNotFoundError:
                if first:
                    raise Stop("identity-changed", True) from None
                self.facts["target_kind"] = "missing"
                return "target-missing"
            except Stop as error:
                if not first and error.code in ("ancestor-not-root-owned", "ancestor-worker-writable",
                        "ancestor-kind-unsupported", "intermediate-link-unsupported"):
                    return error.code
                raise
            try:
                if first and (metadata(info) != metadata(original) or not stat.S_ISLNK(info.st_mode)):
                    raise Stop("identity-changed", True)
                first = False
                if not stat.S_ISLNK(info.st_mode):
                    self.target(actual, info, observations)
                    if info.st_uid != 0:
                        return "target-not-root-owned"
                    if info.st_mode & 0o022:
                        return "target-worker-writable"
                    if not stat.S_ISREG(info.st_mode):
                        return "target-kind-unsupported"
                    return "target-observed" if self.facts["target_in_roots"] else "target-outside-roots"
                if info.st_uid != 0:
                    self.facts["links_root_owned"] = False
                    self.target(actual, info, observations)
                    return "link-not-root-owned"
                self.facts["links_root_owned"] = True
                instance = (info.st_dev, info.st_ino, actual)
                if instance in seen:
                    self.facts["cycle_detected"] = True
                    self.target(actual, info, observations)
                    return "cycle-observed"
                if len(self.hops) >= MAX_HOPS:
                    self.target(actual, info, observations)
                    return "hop-bound"
                raw, text = self.link_text(fd, info)
                self.remember(actual, info, raw)
                seen.add(instance)
                hop = {"path": actual, "metadata": list(metadata(info)),
                       "link_bytes_sha256": hashlib.sha256(raw).hexdigest()}
                if self.proof_bytes + len(canonical(self.hops + [hop])) > MAX_PROOF:
                    raise Stop("byte-bound")
                self.hops.append(hop)
                self.facts.update(hops_observed=len(self.hops), cycle_detected=False,
                                  chain_id=digest(self.hops))
                if self.facts["first_link_form"] is None:
                    self.facts["first_link_form"] = "absolute" if text.startswith("/") else "relative"
                current = text if text.startswith("/") else str(Path(actual).parent / text)
                bounded_path(current)
            finally:
                self.close(fd)

    def recheck(self):
        """Reopen bounded original spellings without file reads and reconcile exact observed generations."""
        for path, row in list(self.records.items()):
            fd, info, _actual, _observations = self.walk(path, retain=False)
            try:
                if list(metadata(info)) != row["metadata"]:
                    raise Stop("identity-changed", True)
                if "link_bytes_hex" in row:
                    raw, _text = self.link_text(fd, info)
                    if raw.hex() != row["link_bytes_hex"]:
                        raise Stop("identity-changed", True)
            finally:
                self.close(fd)
        for path in list(self.absences):
            try:
                fd, _info, _actual, _observations = self.walk(path, retain=False)
            except FileNotFoundError:
                continue
            self.close(fd)
            raise Stop("identity-changed", True)


def load_gate(root, deadline):
    """Load only the exact previously reviewed ordinary observer buffer, never a privileged checkout import."""
    budget(deadline)
    path = root / INPUT_KEYS[1]
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK)
    try:
        first = os.fstat(fd)
        if not stat.S_ISREG(first.st_mode) or first.st_size != EXPECTED_GATE["bytes"]:
            raise Stop("source-unverified")
        raw = bytearray()
        while len(raw) <= EXPECTED_GATE["bytes"]:
            budget(deadline)
            block = os.read(fd, min(8192, EXPECTED_GATE["bytes"] + 1 - len(raw)))
            if not block:
                break
            raw.extend(block)
        if (len(raw) != first.st_size or hashlib.sha256(raw).hexdigest() != EXPECTED_GATE["sha256"]
                or metadata(first) != metadata(os.fstat(fd))
                or metadata(first) != metadata(path.lstat())):
            raise Stop("source-unverified")
    finally:
        os.close(fd)
    gate = types.ModuleType("_sealed_stdlib_gate")
    gate.__file__ = str(path)
    budget(deadline)
    exec(compile(bytes(raw), "<sealed-stdlib-gate>", "exec"), gate.__dict__)
    return gate


def capture_identity(gate, root, requested, tested, deadline):
    """Bind the five fixed ordinary observer/source inputs before and after metadata work."""
    budget(deadline)
    value = gate.capture_identity(root, requested, tested, deadline)
    budget(deadline)
    value["inputs"][INPUT_KEYS[0]] = gate.read_source(root / INPUT_KEYS[0])
    if set(value["inputs"]) != set(INPUT_KEYS):
        raise Stop("source-unverified")
    budget(deadline)
    return value


def discover(gate, paths, deadline):
    """Capture actual first rejected-object arguments and retain the unchanged original predicate call."""
    budget(deadline)
    original = gate.reject_object
    selected = []
    def record(path, info, phase, reason, predicate, index, root, end):
        """Save only the original UID0 entry-kind link before unchanged identity reconciliation."""
        if (phase == "stdlib-entry" and reason == "unsupported-link" and predicate == "entry-kind"
                and info.st_uid == 0 and stat.S_ISLNK(info.st_mode)):
            selected.append((path, info, index, root))
        return original(path, info, phase, reason, predicate, index, root, end)
    gate.reject_object = record
    try:
        gate.stdlib_walk(paths, deadline)
    except gate.ObservationStop as error:
        value = error.value
        if (len(selected) == 1 and value["outcome"] == "rejected"
                and value["object_stat_stable"] is True
                and value["reason"] == "unsupported-link"
                and value["source_predicate"] == "entry-kind"):
            return selected[0]
        raise Stop("entry-not-selected") from None
    finally:
        gate.reject_object = original
    raise Stop("entry-not-selected")


def validate(value):
    """Consume closed shape, exact typed nullable facts and endpoint/source pairing before publication."""
    keys = {"schema_version", "outcome", "first_fault", "identity", *OBJECT_FIELDS}
    if type(value) is not dict or set(value) != keys or value["schema_version"] != SCHEMA:
        raise ValueError("closed sidecar")
    if value["outcome"] not in ("observed", "unavailable") or value["first_fault"] not in FAULTS:
        raise ValueError("closed outcome")
    for name in ("cycle_detected", "target_uid_is_root", "target_in_roots", "parents_root_owned",
                 "parents_nonwritable", "links_root_owned", "object_stat_stable"):
        if value[name] is not None and type(value[name]) is not bool:
            raise ValueError("typed fact")
    for name, upper in (("root_index", 7), ("hops_observed", MAX_HOPS)):
        if value[name] is not None and (type(value[name]) is not int or not 0 <= value[name] <= upper):
            raise ValueError("typed count")
    for name in ("entry_id", "chain_id", "target_id"):
        if value[name] is not None and (type(value[name]) is not str or not re.fullmatch(r"[0-9a-f]{64}", value[name])):
            raise ValueError("opaque identity")
    if value["first_link_form"] not in (None, "relative", "absolute") or value["target_kind"] not in (None, "regular", "directory", "symlink", "other", "missing"):
        raise ValueError("closed kind")
    if value["target_mode_022_bits"] is not None and (type(value["target_mode_022_bits"]) is not int or value["target_mode_022_bits"] not in (0, 2, 16, 18)):
        raise ValueError("typed mode")
    identity = value["identity"]
    if identity is not None:
        if type(identity) is not dict or set(identity) != {"requested_commit", "tested_commit", "before_after", "inputs"}:
            raise ValueError("closed source identity")
        if any(type(identity[k]) is not str or not re.fullmatch(r"(?:[0-9a-f]{40}|[0-9a-f]{64})", identity[k]) for k in ("requested_commit", "tested_commit")):
            raise ValueError("commit identity")
        if identity["before_after"] not in ("unchanged", "changed", "unverified") or type(identity["inputs"]) is not dict or set(identity["inputs"]) != set(INPUT_KEYS):
            raise ValueError("source identity")
        for row in identity["inputs"].values():
            if (type(row) is not dict or set(row) != {"bytes", "sha256"} or type(row["bytes"]) is not int
                    or not 1 <= row["bytes"] <= MAX_SOURCE or type(row["sha256"]) is not str
                    or not re.fullmatch(r"[0-9a-f]{64}", row["sha256"])):
                raise ValueError("source pin")
        if any(identity["inputs"][name] != expected for name, expected in EXPECTED_PROTECTED.items()):
            raise ValueError("protected source drift")
    if value["outcome"] == "unavailable":
        if any(value[name] is not None for name in OBJECT_FIELDS if name != "object_stat_stable") or value["object_stat_stable"] not in (None, False):
            raise ValueError("unavailable facts")
        if value["first_fault"] == "target-observed":
            raise ValueError("unobserved endpoint")
    else:
        observed_reasons = {"target-observed", "target-missing", "target-outside-roots",
            "target-not-root-owned", "target-worker-writable", "target-kind-unsupported",
            "ancestor-not-root-owned", "ancestor-worker-writable", "ancestor-kind-unsupported",
            "link-not-root-owned", "intermediate-link-unsupported", "cycle-observed", "hop-bound"}
        reason, kind = value["first_fault"], value["target_kind"]
        if (reason not in observed_reasons or identity is None or identity["before_after"] != "unchanged"
                or value["root_index"] is None or value["entry_id"] is None
                or value["hops_observed"] is None or value["hops_observed"] < 1
                or value["object_stat_stable"] is not True or value["chain_id"] is None
                or value["first_link_form"] is None or value["cycle_detected"] is None):
            raise ValueError("unbound or unreachable facts")
        expected_parents = (reason != "ancestor-not-root-owned", reason != "ancestor-worker-writable")
        if (value["parents_root_owned"] is not expected_parents[0]
                or value["parents_nonwritable"] is not expected_parents[1]
                or value["links_root_owned"] is not (reason != "link-not-root-owned")
                or value["cycle_detected"] is not (reason == "cycle-observed")):
            raise ValueError("prefix predicate")
        if kind in (None, "missing") and any(value[k] is not None for k in ("target_id", "target_uid_is_root", "target_mode_022_bits", "target_in_roots")):
            raise ValueError("unobserved target")
        if kind not in (None, "missing") and any(value[k] is None for k in ("target_id", "target_uid_is_root", "target_mode_022_bits", "target_in_roots")):
            raise ValueError("partial terminal")
        if reason in ("target-observed", "target-outside-roots"):
            if (kind != "regular" or value["target_uid_is_root"] is not True
                    or value["target_mode_022_bits"] != 0
                    or value["target_in_roots"] is not (reason == "target-observed")):
                raise ValueError("terminal predicate order")
        elif reason == "target-not-root-owned":
            if kind not in ("regular", "directory", "other") or value["target_uid_is_root"] is not False:
                raise ValueError("owner predicate")
        elif reason == "target-worker-writable":
            if (kind not in ("regular", "directory", "other") or value["target_uid_is_root"] is not True
                    or value["target_mode_022_bits"] == 0):
                raise ValueError("mode predicate")
        elif reason == "target-kind-unsupported":
            if (kind not in ("directory", "other") or value["target_uid_is_root"] is not True
                    or value["target_mode_022_bits"] != 0):
                raise ValueError("kind predicate order")
        elif reason == "target-missing":
            if kind != "missing":
                raise ValueError("missing predicate")
        elif reason in ("cycle-observed", "hop-bound", "link-not-root-owned"):
            if kind != "symlink" or value["target_uid_is_root"] is not (reason != "link-not-root-owned"):
                raise ValueError("link endpoint predicate")
            if reason == "hop-bound" and value["hops_observed"] != MAX_HOPS:
                raise ValueError("hop predicate")
        elif kind is not None:
            raise ValueError("unobserved ancestor endpoint")
    if len(canonical(value)) > MAX_SIDECAR:
        raise ValueError("sidecar bound")
    return value


def observe(root, requested, tested, *, gate=None, inspector_type=Inspector, now=time.monotonic):
    """Discover/observe/reconcile one fresh prefix and revoke facts on any unknown source/handle state."""
    deadline = now() + 600
    result, before, inspector = empty(), None, None
    primary = None
    try:
        budget(deadline, now)
        if sys.platform != "linux" or os.uname().machine != "x86_64" or os.getuid() == 0 or os.geteuid() != os.getuid() or os.getegid() != os.getgid():
            raise Stop("unsupported-platform")
        gate = load_gate(root, deadline) if gate is None else gate
        before = capture_identity(gate, root, requested, tested, deadline)
        paths = gate.probe_paths(root, deadline)
        selection = discover(gate, paths, deadline)
        inspector = inspector_type(deadline, now=now)
        primary = inspector.trace(selection, paths)
        inspector.recheck()
        result = {"schema_version": SCHEMA, "outcome": "observed", "first_fault": primary,
                  "identity": None, **inspector.facts, "object_stat_stable": True}
    except Stop as error:
        primary = error.code
        result = empty(error.code, error.changed)
    except Exception:
        result = empty()
    finally:
        if inspector is not None and not inspector.close_all():
            result = empty(primary if primary not in (None, "target-observed") else "descriptor-close-unverified")
    if before is not None:
        try:
            budget(deadline, now)
            after = capture_identity(gate, root, requested, tested, deadline)
            before["before_after"] = "unchanged" if before == after else "changed"
            if before["before_after"] != "unchanged":
                result = empty("source-unverified")
        except Exception:
            before["before_after"] = "unverified"
            result = empty("source-unverified")
        result["identity"] = before
    return validate(result)


def publish(destination, value):
    """Publish only a fresh bounded canonical file with explicit raw descriptor ownership."""
    raw = canonical(validate(value))
    destination = Path(destination)
    destination.mkdir(mode=0o700, parents=False, exist_ok=False)
    directory = os.open(destination, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)
    temporary, created, file_fd = None, False, None
    try:
        temporary = ".chain-observation-" + os.urandom(16).hex()
        original = os.fstat(directory)
        if (not stat.S_ISDIR(original.st_mode) or original.st_uid != os.getuid()
                or original.st_mode & 0o777 != 0o700
                or metadata(original) != metadata(destination.lstat())):
            raise OSError("publication directory")
        file_fd = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW | os.O_CLOEXEC,
                          0o600, dir_fd=directory)
        created = True
        offset = 0
        while offset < len(raw):
            written = os.write(file_fd, raw[offset:])
            if type(written) is not int or not 1 <= written <= len(raw) - offset:
                raise OSError("publication")
            offset += written  # At most len(raw)<=2048 iterations; zero cannot spin indefinitely.
        os.fsync(file_fd)
        owned, file_fd = file_fd, None
        os.close(owned)  # Unknown closure stops publication; never retry a possibly recycled fd.
        os.link(temporary, OUTPUT, src_dir_fd=directory, dst_dir_fd=directory, follow_symlinks=False)
        os.fsync(directory)
        current = destination.lstat()
        if ((original.st_dev, original.st_ino, original.st_uid, original.st_mode)
                != (current.st_dev, current.st_ino, current.st_uid, current.st_mode)):
            raise OSError("publication directory changed")
    finally:
        try:
            if file_fd is not None:
                os.close(file_fd)
        finally:
            try:
                if created:
                    os.unlink(temporary, dir_fd=directory)
                    os.fsync(directory)
            finally:
                os.close(directory)


def publication_flag(environment):
    """Append one fixed positive line only to a qualified runner file after fresh publication."""
    if environment.get("GITHUB_ACTIONS") != "true":
        return False
    output, runner_temp = environment.get("GITHUB_OUTPUT"), environment.get("RUNNER_TEMP")
    if not output or not runner_temp:
        return False
    output, runner_temp = Path(bounded_path(output)), Path(bounded_path(runner_temp))
    # Qualify the original spelling. Canonicalization before the no-follow check would hide links.
    if ".." in output.parts or ".." in runner_temp.parts or not output.is_relative_to(runner_temp):
        return False
    if len(output.parts) > MAX_COMPONENTS:
        return False
    parent = os.open("/", os.O_PATH | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)
    held = {parent}
    attempted = set()
    fd = None
    try:
        root_info = os.fstat(parent)
        if not stat.S_ISDIR(root_info.st_mode) or root_info.st_uid not in (0, os.getuid()) or root_info.st_mode & 0o022:
            return False
        for component in output.parts[1:-1]:
            child = os.open(component, os.O_PATH | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC, dir_fd=parent)
            held.add(child)
            try:
                info = os.fstat(child)
                if not stat.S_ISDIR(info.st_mode) or info.st_uid not in (0, os.getuid()) or info.st_mode & 0o022:
                    return False
            finally:
                attempted.add(parent)
                os.close(parent)
                held.remove(parent)
                attempted.discard(parent)
                parent = child
        before = os.stat(output.name, dir_fd=parent, follow_symlinks=False)
        if (not stat.S_ISREG(before.st_mode) or before.st_uid != os.getuid() or before.st_nlink != 1
                or before.st_mode & 0o022 or not 0 <= before.st_size <= 1048576):
            return False
        fd = os.open(output.name, os.O_WRONLY | os.O_APPEND | os.O_NOFOLLOW | os.O_CLOEXEC, dir_fd=parent)
        held.add(fd)
        if metadata(before) != metadata(os.fstat(fd)):
            return False
        # A complete line may persist despite a later close fault. It still follows actual new publication.
        return os.write(fd, FLAG) == len(FLAG)
    finally:
        failed = False
        for handle in held:
            if handle in attempted:
                failed = True
                continue
            try:
                os.close(handle)
            except BaseException:
                failed = True
        if failed:
            raise OSError("flag reference")


def main():
    """Expose fixed context choices only; publication faults never authorize stale-file upload."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--requested-commit", required=True)
    parser.add_argument("--tested-commit", required=True)
    args = parser.parse_args()
    try:
        root = Path(__file__).resolve().parents[1]
        value = observe(root, args.requested_commit, args.tested_commit)
        publish(args.output_dir, value)
        if os.environ.get("GITHUB_ACTIONS") == "true" and not publication_flag(os.environ):
            raise OSError("publication flag")
    except Exception:
        print("Stdlib link-chain observation could not be published.", file=sys.stderr)
        return 1
    print("Stdlib link-chain observation retained; native qualification remains unchanged.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
