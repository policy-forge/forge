#!/usr/bin/env python3
"""Exercise the separate stat observer with stdlib-only mocked seams; these controls never establish Linux/native qualification."""
import contextlib
import copy
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import stat
import tempfile
import types
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("stdlib_observer_under_test", ROOT / "scripts/observe_workspace_stdlib_gate.py")
OBSERVER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(OBSERVER)
COMMIT = "a" * 40


def info(mode=stat.S_IFREG | 0o644, uid=0, size=10, inode=2):
    """Create a synthetic stat instance with every metadata field used by the actual recheck and source reader."""
    return types.SimpleNamespace(st_dev=1, st_ino=inode, st_mode=mode, st_uid=uid, st_gid=0,
        st_size=size, st_mtime_ns=3, st_ctime_ns=4)


def identity(state="unchanged"):
    """Construct only a schema-shaped source identity; it makes no authentic checkout or execution claim."""
    pins = {name: {"bytes": 1, "sha256": "b" * 64} for name in OBSERVER.INPUT_KEYS}
    pins.update(copy.deepcopy(OBSERVER.EXPECTED_PROTECTED))
    return {"requested_commit": COMMIT, "tested_commit": COMMIT, "before_after": state, "inputs": pins}


def rejected():
    """Build one source-bound synthetic first-mode rejection for closed-envelope and revocation controls."""
    value = OBSERVER.empty_observation("rejected", "stdlib-entry", "worker-writable", True)
    value.update(source_predicate="entry-mode-022", root_index=0, object_id="c" * 64,
        object_kind="regular", uid_is_root=True, mode_022_bits=2, identity=identity())
    return value


def capture():
    """Return the before-comparison form expected from capture_identity, with no mock authenticated-parent evidence."""
    return identity("unverified")


class Entry:
    """Expose one fake scandir entry and record whether the real walk avoided following symlinks."""

    def __init__(self, path, observed=None, error=None):
        """Attach one stat result or failure without allocating any real stdlib fixture."""
        self.path = str(path)
        self.observed = observed if observed is not None else info()
        self.error = error
        self.calls = []

    def stat(self, *, follow_symlinks=True):
        """Record the actual no-follow flag and reproduce a stat failure without private exception publication."""
        self.calls.append(follow_symlinks)
        if self.error is not None:
            raise self.error
        return self.observed


class Scandir:
    """Represent a bounded iterator context consumed by the production walk rather than a prebuilt source inventory."""

    def __init__(self, entries):
        """Retain only the test-supplied iterable of synthetic entries."""
        self.entries = entries

    def __enter__(self):
        """Yield the supplied iterator exactly as a scandir context would."""
        return iter(self.entries)

    def __exit__(self, *_args):
        """Close the synthetic context without converting a body exception into success."""
        return False


class Stream:
    """Track a fake owned pipe independently so close failures cannot hide other required close attempts."""

    def __init__(self, descriptor, close_error=False):
        """Set a unique fake descriptor and an optional deterministic close fault."""
        self.descriptor = descriptor
        self.close_error = close_error
        self.close_calls = 0

    def fileno(self):
        """Expose the stable fake descriptor only to the patched actual command seam."""
        return self.descriptor

    def close(self):
        """Record every independent attempt, optionally failing after it was observed."""
        self.close_calls += 1
        if self.close_error:
            raise OSError("private-close-detail")


class Child:
    """Model only a sole fake Popen owner; no subprocess or native experiment is launched by this seam."""

    def __init__(self, code=0, stdin=False):
        """Provide distinct stdout/stderr and optional stdin handles plus a deterministic exit state."""
        self.stdout = Stream(101)
        self.stderr = Stream(102)
        self.stdin = Stream(103) if stdin else None
        self.code = code
        self.kills = 0
        self.waits = []

    def poll(self):
        """Return the fake child's exact observed state without deriving cleanup from a signal."""
        return self.code

    def kill(self):
        """Mark only this original fake child terminated for the owner's later wait control."""
        self.kills += 1
        self.code = -9

    def wait(self, timeout=None):
        """Record the owner's reap attempt and return the exact fake exit code."""
        self.waits.append(timeout)
        return self.code


class Selector:
    """Expose separate readiness/EOF channels to the actual command loop and independent selector close accounting."""

    def __init__(self, close_error=False):
        """Start with no registered streams and an optional deterministic final-close fault."""
        self.mapping = {}
        self.close_error = close_error
        self.close_calls = 0

    def register(self, stream, _event):
        """Retain each exact fake stream registration for later readiness delivery."""
        self.mapping[stream] = stream

    def unregister(self, stream):
        """Remove only the stream whose actual loop observed EOF or finished writing."""
        del self.mapping[stream]

    def get_map(self):
        """Return the live fake registration mapping so both EOFs govern loop completion."""
        return self.mapping

    def select(self, _timeout):
        """Deliver readiness for currently registered streams without spawning a scheduler or clock thread."""
        return [(types.SimpleNamespace(fileobj=stream), 0) for stream in list(self.mapping)]

    def close(self):
        """Record close independently of pipe closes and optionally raise a private fault."""
        self.close_calls += 1
        if self.close_error:
            raise OSError("private-selector-detail")


@contextlib.contextmanager
def command_seam(stdout=(b"ok", b""), stderr=(b"warning", b""), child=None, selector=None, clock=None):
    """Patch OS endpoints around the real command function while preserving its actual drainage/exit/finally logic."""
    child = child if child is not None else Child()
    selector = selector if selector is not None else Selector()
    chunks = {101: iter(stdout), 102: iter(stderr)}
    def read(descriptor, _size):
        """Supply separate finite pipe chunks and EOFs to the production loop."""
        return next(chunks[descriptor])
    with contextlib.ExitStack() as stack:
        spawn = stack.enter_context(mock.patch.object(OBSERVER.subprocess, "Popen", return_value=child))
        stack.enter_context(mock.patch.object(OBSERVER.selectors, "DefaultSelector", return_value=selector))
        stack.enter_context(mock.patch.object(OBSERVER.os, "set_blocking"))
        stack.enter_context(mock.patch.object(OBSERVER.os, "read", side_effect=read))
        stack.enter_context(mock.patch.object(OBSERVER.time, "monotonic", side_effect=clock, return_value=0))
        yield child, selector, spawn


@contextlib.contextmanager
def walk_seam(entries, *, stat_value=None):
    """Drive real walk predicates using synthetic scandir/stat values and forbid any regular-content open."""
    roots = ["/stdlib/base", "/stdlib/lib-dynload"]
    def scan(path):
        """Expose supplied entries for the first root and empty subsequent roots unless a map requests recursion."""
        selected = entries.get(str(path), []) if isinstance(entries, dict) else entries if str(path) == roots[0] else []
        return Scandir(selected)
    with contextlib.ExitStack() as stack:
        stack.enter_context(mock.patch.object(OBSERVER, "root_trusted"))
        stack.enter_context(mock.patch.object(OBSERVER.Path, "exists", return_value=True))
        stack.enter_context(mock.patch.object(OBSERVER.os, "scandir", side_effect=scan))
        stack.enter_context(mock.patch.object(OBSERVER.os, "open", side_effect=AssertionError("stdlib content must not open")))
        stack.enter_context(mock.patch.object(OBSERVER.time, "monotonic", return_value=0))
        if stat_value is not None:
            stack.enter_context(mock.patch.object(OBSERVER.Path, "lstat", return_value=stat_value))
        yield roots


@contextlib.contextmanager
def observe_seam(before=None, after=None, failure=None, platform="linux"):
    """Exercise the actual before/after revocation path while keeping probes and filesystem/native work mocked."""
    before = capture() if before is None else before
    after = capture() if after is None else after
    with contextlib.ExitStack() as stack:
        captures = stack.enter_context(mock.patch.object(OBSERVER, "capture_identity", side_effect=[copy.deepcopy(before), after]))
        probe = stack.enter_context(mock.patch.object(OBSERVER, "probe_paths", return_value=["/stdlib/base", "/stdlib/lib-dynload"]))
        stack.enter_context(mock.patch.object(OBSERVER, "stdlib_walk", side_effect=failure))
        stack.enter_context(mock.patch.object(OBSERVER.sys, "platform", platform))
        stack.enter_context(mock.patch.object(OBSERVER.os, "uname", return_value=types.SimpleNamespace(machine="x86_64")))
        stack.enter_context(mock.patch.object(OBSERVER.os, "getuid", return_value=1000))
        stack.enter_context(mock.patch.object(OBSERVER.os, "geteuid", return_value=1000))
        stack.enter_context(mock.patch.object(OBSERVER.os, "getgid", return_value=1000))
        stack.enter_context(mock.patch.object(OBSERVER.os, "getegid", return_value=1000))
        stack.enter_context(mock.patch.object(OBSERVER.time, "monotonic", return_value=0))
        yield captures, probe


class ObservationControls(unittest.TestCase):
    """Cover closed records, original stat ordering/caps, source revocation and private no-replacement publication."""

    def stop(self, function, *args, **kwargs):
        """Capture the actual typed observation without accepting arbitrary exceptions as the expected fixed result."""
        with self.assertRaises(OBSERVER.ObservationStop) as caught:
            function(*args, **kwargs)
        return caught.exception.value

    def assert_unavailable(self, value, phase=None, reason=None):
        """Require unavailable and null object claims, optionally checking the exact retained closed phase/reason."""
        self.assertEqual(value["outcome"], "unavailable")
        for name in OBSERVER.OBJECT_FIELDS:
            if name != "object_stat_stable":
                self.assertIsNone(value[name])
        if phase is not None:
            self.assertEqual((value["phase"], value["reason"]), (phase, reason))

    def test_closed_rejection_and_size(self):
        """Accept one bound synthetic record, preserve exact twelve fields and keep its canonical bytes within 2KiB."""
        value = rejected()
        self.assertIs(OBSERVER.validate_observation(value), value)
        self.assertEqual(len(value), 12)
        self.assertLessEqual(len(OBSERVER.canonical_bytes(value)), 2048)
        self.assertEqual(sum(map(len, OBSERVER.TOOL_PHASE_REASONS.values())), 71)
        self.assertTrue(OBSERVER.canonical_bytes(value).endswith(b"\n"))

    def test_record_types_pairs_and_predicates_closed(self):
        """Reject booleans masquerading as counters, private fields and inconsistent first-predicate facts."""
        changes = [("root_index", True), ("root_index", 8), ("mode_022_bits", True), ("uid_is_root", 1),
            ("object_stat_stable", 1), ("reason", "private-error"), ("object_id", "x" * 64),
            ("phase", "stdlib-roots"), ("mode_022_bits", 0), ("outcome", "passed")]
        for key, item in changes:
            with self.subTest(key=key, item=item):
                value = rejected(); value[key] = item
                with self.assertRaises((ValueError, TypeError)):
                    OBSERVER.validate_observation(value)
        value = rejected(); value["private_path"] = "/private/name"
        with self.assertRaises(ValueError): OBSERVER.validate_observation(value)
        value = rejected(); value.update(source_predicate="entry-kind", reason="unsupported-link", mode_022_bits=0)
        with self.assertRaises(ValueError): OBSERVER.validate_observation(value)
        value["object_kind"] = "symlink"
        self.assertIs(OBSERVER.validate_observation(value), value)
        for predicate, reason, kind in (("entry-kind","not-directory","regular"),
                ("entry-kind","not-regular","directory"),
                ("root-ancestor-kind","unsupported-kind","other"),
                ("root-ancestor-kind","not-regular","directory"),
                ("root-ancestor-kind","not-directory","symlink")):
            value = rejected(); value.update(source_predicate=predicate, reason=reason, object_kind=kind, mode_022_bits=0)
            with self.subTest(predicate=predicate, reason=reason), self.assertRaises(ValueError):
                OBSERVER.validate_observation(value)

    def test_v2_link_kind_mask_roundtrip_and_v1_mode_rejection(self):
        """Retain masked link metadata under /2 while refusing stale /1 and impossible link-mode rejection facts."""
        for predicate, phase in (("entry-kind", "stdlib-entry"), ("root-ancestor-kind", "stdlib-roots")):
            for mask in (0, 2, 16, 18):
                value = rejected()
                value.update(source_predicate=predicate, phase=phase, reason="unsupported-link", object_kind="symlink", mode_022_bits=mask)
                self.assertEqual(OBSERVER.validate_observation(json.loads(OBSERVER.canonical_bytes(value))), value)
                stale = copy.deepcopy(value); stale["schema_version"] = "forge.stdlib-gate-observation/1"
                with self.assertRaises(ValueError): OBSERVER.validate_observation(stale)
                mislabeled = copy.deepcopy(value)
                mislabeled.update(source_predicate="entry-mode-022" if phase == "stdlib-entry" else "root-ancestor-mode-022", reason="worker-writable")
                with self.assertRaises(ValueError): OBSERVER.validate_observation(mislabeled)
                nonlink = copy.deepcopy(value); nonlink["object_kind"] = "regular"
                with self.assertRaises(ValueError): OBSERVER.validate_observation(nonlink)

    def test_source_identity_closed_and_protected(self):
        """Reject malformed hashes/types, source-map extras and changed or missing protected identity before publishing facts."""
        for change in ("extra", "boolean-size", "wrong-native", "head", "changed", "missing"):
            with self.subTest(change=change):
                value = rejected()
                if change == "extra": value["identity"]["extra"] = True
                elif change == "boolean-size": value["identity"]["inputs"][OBSERVER.INPUT_KEYS[0]]["bytes"] = True
                elif change == "wrong-native": value["identity"]["inputs"][OBSERVER.INPUT_KEYS[2]]["sha256"] = "0" * 64
                elif change == "head": value["identity"]["tested_commit"] = True
                elif change == "changed": value["identity"]["before_after"] = "changed"
                else: value["identity"] = None
                with self.assertRaises((ValueError, TypeError)): OBSERVER.validate_observation(value)

    def test_unavailable_and_no_rejection_are_not_pass(self):
        """Keep incomplete alternatives free of object claims and require unchanged identity only for the no-rejection alternative."""
        value = OBSERVER.empty_observation()
        self.assertIs(OBSERVER.validate_observation(value), value)
        value["object_stat_stable"] = 0
        with self.assertRaises(ValueError): OBSERVER.validate_observation(value)
        value = OBSERVER.empty_observation("no-rejection")
        with self.assertRaises(ValueError): OBSERVER.validate_observation(value)
        value["identity"] = identity()
        self.assertIs(OBSERVER.validate_observation(value), value)
        value["object_stat_stable"] = False
        with self.assertRaises(ValueError): OBSERVER.validate_observation(value)

    def test_strict_probe_decoding(self):
        """Reject nested/escaped duplicates, nonfinite scalars, malformed UTF-8 and decoded depth beyond the original bound."""
        for raw in (b'{"x":{"k":1,"k":2}}', b'{"k":1,"\\u006b":2}', b'{"x":NaN}', b'\xff', b'[' * 17 + b'0' + b']' * 17):
            with self.subTest(raw=repr(raw)):
                value = self.stop(OBSERVER.strict_json, raw)
                self.assert_unavailable(value)
        self.assertEqual(OBSERVER.strict_json(b'{"version":"3.11.0","paths":[]}'), {"version":"3.11.0","paths":[]})

    def test_deadline_fences_never_spawn_expired_commands(self):
        """Treat equality as expiry both before selector allocation and immediately before the actual spawn seam."""
        for now in (10, 11):
            with self.subTest(now=now), mock.patch.object(OBSERVER.time, "monotonic", return_value=now), mock.patch.object(OBSERVER.subprocess, "Popen") as spawn, mock.patch.object(OBSERVER.selectors, "DefaultSelector") as select:
                self.assertEqual(OBSERVER.command(["git"], ROOT, 10), {"exit_code":None,"failure":"command-timeout","output":b""})
                spawn.assert_not_called(); select.assert_not_called()
        with mock.patch.object(OBSERVER.time, "monotonic", side_effect=[9, 10]), mock.patch.object(OBSERVER.subprocess, "Popen") as spawn, mock.patch.object(OBSERVER.selectors, "DefaultSelector", return_value=Selector()):
            self.assertEqual(OBSERVER.command(["git"], ROOT, 10)["failure"], "command-timeout")
            spawn.assert_not_called()
        with mock.patch.object(OBSERVER.time, "monotonic", return_value=9): OBSERVER.budget(10)
        with mock.patch.object(OBSERVER.time, "monotonic", return_value=10):
            self.assert_unavailable(self.stop(OBSERVER.budget, 10), "qualification-budget", "deadline-expired")

    def test_stable_recheck_redacts_object_identity(self):
        """Retain only a digest/coarse stat tuple after the actual lstat metadata matches the original instance."""
        observed = info(mode=stat.S_IFREG | 0o666)
        with mock.patch.object(OBSERVER.Path, "lstat", return_value=observed) as checked, mock.patch.object(OBSERVER.time, "monotonic", return_value=0):
            value = self.stop(OBSERVER.reject_object, Path("/private/secret"), observed, "stdlib-entry", "worker-writable", "entry-mode-022", 1, Path("/stdlib"), 10)
        self.assertEqual(checked.call_count, 1)
        self.assertEqual(value["outcome"], "rejected")
        self.assertIs(value["object_stat_stable"], True)
        self.assertEqual(value["mode_022_bits"], 18)
        self.assertEqual(len(value["object_id"]), 64)
        self.assertNotIn(b"secret", OBSERVER.canonical_bytes(value))
        self.assertNotIn(b"/stdlib", OBSERVER.canonical_bytes(value))

    def test_replaced_or_unobserved_object_revokes_facts(self):
        """Discard all identifiers when the original instance changes or the second stat cannot be established."""
        for replacement in (info(inode=3), OSError("private-recheck-detail")):
            with self.subTest(replacement=type(replacement).__name__), mock.patch.object(OBSERVER.Path, "lstat", side_effect=replacement if isinstance(replacement, Exception) else None, return_value=replacement), mock.patch.object(OBSERVER.time, "monotonic", return_value=0):
                value = self.stop(OBSERVER.reject_object, Path("/private/name"), info(), "stdlib-entry", "worker-writable", "entry-mode-022", 0, Path("/stdlib"), 10)
                self.assert_unavailable(value, "stdlib-entry", "entry-observation-unverified")
                self.assertIs(value["object_stat_stable"], None if isinstance(replacement, Exception) else False)

    def test_entry_uid_link_kind_nonlink_mode_order(self):
        """Exercise the real streamed predicate order on competing non-root, writable and symlink facts."""
        cases = [(info(stat.S_IFLNK | 0o777, uid=42), "entry-uid", "not-root-owned"),
            (info(stat.S_IFLNK | 0o777), "entry-kind", "unsupported-link"),
            (info(stat.S_IFLNK | 0o755), "entry-kind", "unsupported-link"),
            (info(stat.S_IFREG | 0o666), "entry-mode-022", "worker-writable"),
            (info(stat.S_IFDIR | 0o777), "entry-mode-022", "worker-writable"),
            (info(stat.S_IFIFO | 0o644), "entry-kind", "unsupported-kind")]
        for observed, predicate, reason in cases:
            entry = Entry("/stdlib/base/private", observed)
            with self.subTest(predicate=predicate), walk_seam([entry], stat_value=observed) as roots:
                value = self.stop(OBSERVER.stdlib_walk, roots, 10)
                self.assertEqual((value["source_predicate"], value["reason"]), (predicate, reason))
                self.assertEqual(entry.calls, [False])

    def test_ancestor_link_precedes_mode_and_admin_has_no_object(self):
        """Keep ancestor ordering identical and prevent an administrative path rejection from inventing a stdlib identifier."""
        observed = info(stat.S_IFLNK | 0o777)
        with mock.patch.object(OBSERVER.Path, "lstat", return_value=observed), mock.patch.object(OBSERVER.time, "monotonic", return_value=0):
            value = self.stop(OBSERVER.root_trusted, Path("/stdlib/root"), True, "stdlib-roots", 0, Path("/stdlib/root"), 10)
            self.assertEqual((value["reason"], value["source_predicate"]), ("unsupported-link", "root-ancestor-kind"))
            value = self.stop(OBSERVER.root_trusted, Path("/usr/bin"), True, "python-path", deadline=10)
            self.assert_unavailable(value, "python-path", "unsupported-link")

    def test_size_bound_and_no_content_read(self):
        """Accept the exact aggregate stat-size bound and reject one byte over without opening regular stdlib content."""
        for size, failure in ((OBSERVER.MAX_STDLIB_BYTES, None), (OBSERVER.MAX_STDLIB_BYTES + 1, "byte-bound")):
            entry = Entry("/stdlib/base/a", info(size=size))
            with self.subTest(size=size), walk_seam([entry]) as roots:
                if failure is None: OBSERVER.stdlib_walk(roots, 10)
                else: self.assert_unavailable(self.stop(OBSERVER.stdlib_walk, roots, 10), "stdlib-entry", failure)
                self.assertEqual(entry.calls, [False])

    def test_streamed_entry_bound_before_excess_stat(self):
        """Exercise the actual inclusive 20,000-entry cap with no stat of the 20,001st fake entry."""
        entries = [Entry("/stdlib/base/" + str(index)) for index in range(OBSERVER.MAX_ENTRIES + 1)]
        with walk_seam(entries) as roots:
            self.assert_unavailable(self.stop(OBSERVER.stdlib_walk, roots, 10), "stdlib-entry", "entry-bound")
        self.assertEqual(sum(len(entry.calls) for entry in entries), 20000)
        self.assertEqual(entries[-1].calls, [])
        with walk_seam(entries[:-1]) as roots: OBSERVER.stdlib_walk(roots, 10)

    def test_root_shape_dynload_empty_and_stat_failure(self):
        """Reject unsupported root sets, missing dynload, empty observations and stat faults without object facts."""
        for roots in ([], ["/one"], ["/same", "/same"], ["relative", "/absolute"], ["/a" + str(n) for n in range(9)], [True, "/base"]):
            self.assert_unavailable(self.stop(OBSERVER.stdlib_walk, roots, 10), "stdlib-roots", "root-shape-invalid")
        with walk_seam([]):
            self.assert_unavailable(self.stop(OBSERVER.stdlib_walk, ["/a", "/b"], 10), "stdlib-roots", "dynload-missing")
        with walk_seam([]) as roots:
            self.assert_unavailable(self.stop(OBSERVER.stdlib_walk, roots, 10), "stdlib-entry", "entry-observation-unverified")
        with walk_seam([Entry("/stdlib/base/a", error=OSError("private-stat-detail"))]) as roots:
            self.assert_unavailable(self.stop(OBSERVER.stdlib_walk, roots, 10), "stdlib-entry", "entry-observation-unverified")

    def test_depth_bound_before_deeper_scan(self):
        """Reject depth 33 before scanning that directory while consuming only bounded fake owned entries."""
        directory = "/stdlib/base"; mapping = {}
        for _ in range(33):
            deeper = directory + "/d"; mapping[directory] = [Entry(deeper, info(stat.S_IFDIR | 0o755))]; directory = deeper
        with walk_seam(mapping) as roots:
            self.assert_unavailable(self.stop(OBSERVER.stdlib_walk, roots, 10), "stdlib-entry", "depth-bound")

    def test_actual_administrative_tool_trust_and_link_bounds(self):
        """Exercise real fixed-path admission, ancestors, relative/absolute links and closed failure reasons without tool execution."""
        fixed = Path("/usr/bin/python3")
        resolved = Path("/usr/bin/python3.real")
        for case in ("regular", "absolute-link", "relative-link", "missing", "uid", "write", "loop", "not-executable", "kind", "ancestor"):
            with self.subTest(case=case):
                def observed(path):
                    """Supply only stat metadata for this case while the actual trust/ancestor predicates run."""
                    if path == fixed:
                        if case == "uid": return info(stat.S_IFLNK | 0o777, uid=42)
                        if case == "write": return info(stat.S_IFREG | 0o666)
                        if case in ("absolute-link", "relative-link", "loop"): return info(stat.S_IFLNK | 0o777)
                        if case == "kind": return info(stat.S_IFIFO | 0o755)
                        return info(stat.S_IFREG | 0o755)
                    if path == resolved: return info(stat.S_IFREG | 0o755)
                    return info(stat.S_IFDIR | 0o755, uid=42 if case == "ancestor" and path == fixed.parent else 0)
                target = str(fixed) if case == "loop" else resolved.name if case == "relative-link" else str(resolved)
                actual = resolved if case in ("absolute-link", "relative-link") else fixed
                with contextlib.ExitStack() as stack:
                    stack.enter_context(mock.patch.object(OBSERVER.time, "monotonic", return_value=0))
                    stack.enter_context(mock.patch.object(OBSERVER.Path, "exists", return_value=case != "missing"))
                    stack.enter_context(mock.patch.object(OBSERVER.Path, "lstat", autospec=True, side_effect=observed))
                    resolution = stack.enter_context(mock.patch.object(OBSERVER.Path, "resolve", return_value=actual))
                    links = stack.enter_context(mock.patch.object(OBSERVER.os, "readlink", return_value=target))
                    access = stack.enter_context(mock.patch.object(OBSERVER.os, "access", return_value=case != "not-executable"))
                    spawn = stack.enter_context(mock.patch.object(OBSERVER.subprocess, "Popen"))
                    opened = stack.enter_context(mock.patch.object(OBSERVER.os, "open"))
                    if case in ("regular", "absolute-link", "relative-link"):
                        self.assertEqual(OBSERVER.administrative_tool(fixed, "python-path", 10), actual)
                        resolution.assert_called_once_with(strict=True)
                        access.assert_called_once_with(actual, os.X_OK)
                        self.assertEqual(links.call_count, 0 if case == "regular" else 1)
                    else:
                        reason = {"missing":"missing", "uid":"not-root-owned", "write":"worker-writable", "loop":"link-bound",
                            "not-executable":"not-executable", "kind":"not-regular", "ancestor":"not-root-owned"}[case]
                        self.assert_unavailable(self.stop(OBSERVER.administrative_tool, fixed, "python-path", 10), "python-path", reason)
                        if case == "loop":
                            self.assertEqual(links.call_count, 16)
                            resolution.assert_not_called()
                    spawn.assert_not_called()
                    opened.assert_not_called()


    def test_fixed_probe_ignores_path_and_never_executes_ip_sudo(self):
        """Require all fixed administrative qualifications and dispatch only the isolated fixed-Python probe once."""
        def qualified(path, _phase, _deadline):
            """Return only the exact supplied fixed path; no PATH search or substitute interpreter is accepted."""
            return Path(path)
        with mock.patch.dict(os.environ, {"PATH":"/malicious/private"}), mock.patch.object(OBSERVER, "administrative_tool", side_effect=qualified) as qualify, mock.patch.object(OBSERVER, "command", return_value={"exit_code":0,"failure":None,"output":b'{"version":"3.11.0","paths":["/a","/b"]}'}) as command, mock.patch.object(OBSERVER.time, "monotonic", return_value=0):
            self.assertEqual(OBSERVER.probe_paths(ROOT, 10), ["/a", "/b"])
            self.assertEqual([str(row.args[0]) for row in qualify.call_args_list], ["/usr/bin/python3", "/usr/bin/ip", "/usr/bin/sudo"])
            self.assertEqual(command.call_count, 1)
            self.assertEqual(command.call_args.args[0], ["/usr/bin/python3", "-I", "-S", "-B", "-c", OBSERVER.PYTHON_PROBE])

    def test_missing_ip_sudo_stops_before_probe(self):
        """Prove a missing prerequisite is unavailable and cannot trigger Python, sudo, ip or a fallback command."""
        for absent in ("ip", "sudo"):
            def qualified(path, phase, _deadline):
                """Fail exactly the selected fixed prerequisite while returning other stat-qualified paths."""
                if phase == absent + "-path": OBSERVER.unavailable(phase, "missing")
                return Path(path)
            with self.subTest(absent=absent), mock.patch.object(OBSERVER, "administrative_tool", side_effect=qualified), mock.patch.object(OBSERVER, "command") as command, mock.patch.object(OBSERVER.time, "monotonic", return_value=0):
                self.assert_unavailable(self.stop(OBSERVER.probe_paths, ROOT, 10), absent + "-path", "missing")
                command.assert_not_called()

    def test_probe_faults_closed_and_output_private(self):
        """Keep invalid JSON/shape/version and command failure in the exact unavailable vocabulary with no private output."""
        rows = [(b'private\xff', "probe-json-invalid"), (b'{"version":"3.11.0","paths":[],"private":"secret"}', "probe-shape-invalid"),
            (b'{"version":"3.10.9","paths":[]}', "version-invalid")]
        for raw, reason in rows:
            with self.subTest(reason=reason), mock.patch.object(OBSERVER, "administrative_tool", side_effect=lambda path, *_args: Path(path)), mock.patch.object(OBSERVER, "command", return_value={"exit_code":0,"failure":None,"output":raw}), mock.patch.object(OBSERVER.time, "monotonic", return_value=0):
                value = self.stop(OBSERVER.probe_paths, ROOT, 10)
                self.assert_unavailable(value, "python-probe", reason)
                self.assertNotIn(b"secret", OBSERVER.canonical_bytes(value))
        for failure, reason in (("command-timeout","command-timeout"), ("output-bound","output-bound"), ("cleanup-unverified","command-cleanup-unverified"), (None,"command-failed")):
            with self.subTest(failure=failure), mock.patch.object(OBSERVER, "administrative_tool", side_effect=lambda path, *_args: Path(path)), mock.patch.object(OBSERVER, "command", return_value={"exit_code":1,"failure":failure,"output":b"secret"}), mock.patch.object(OBSERVER.time, "monotonic", return_value=0):
                self.assert_unavailable(self.stop(OBSERVER.probe_paths, ROOT, 10), "python-probe", reason)

    def test_actual_command_separate_streams_and_full_eof(self):
        """Exercise the real command loop with private stderr and require both EOFs while returning only stdout."""
        with command_seam(stdout=(b"ok",b""), stderr=(b"private-warning",b"")) as (child, selector, spawn):
            self.assertEqual(OBSERVER.command(["python"], ROOT, 10), {"exit_code":0,"failure":None,"output":b"ok"})
            self.assertEqual(selector.mapping, {})
            self.assertEqual((child.stdout.close_calls, child.stderr.close_calls, selector.close_calls), (1,1,1))
            self.assertIs(spawn.call_args.kwargs["stdout"], OBSERVER.subprocess.PIPE)
            self.assertIs(spawn.call_args.kwargs["stderr"], OBSERVER.subprocess.PIPE)
            self.assertEqual(child.kills, 0)

    def test_actual_command_stderr_bound_discards_stdout(self):
        """Keep aggregate accounting across both pipes and erase otherwise-valid stdout when stderr exceeds the cap."""
        with command_seam(stdout=(b"ok",b""), stderr=(b"x" * OBSERVER.MAX_CAPTURE,b"")):
            self.assertEqual(OBSERVER.command(["python"], ROOT, 10), {"exit_code":0,"failure":"output-bound","output":b""})

    def test_actual_command_independent_close_and_first_fault(self):
        """Attempt every owned pipe close after selector failure and retain an earlier bound failure over cleanup detail."""
        child = Child(stdin=True); child.stdin.close_error = True; child.stdout.close_error = True
        selector = Selector(close_error=True)
        with command_seam(child=child, selector=selector), mock.patch.object(OBSERVER.os, "write", return_value=0):
            result = OBSERVER.command(["python"], ROOT, 10, input_bytes=b"")
            self.assertEqual((result["failure"],result["output"]), ("execution-unverified",b""))
            self.assertGreaterEqual(child.stdin.close_calls, 1)
            self.assertEqual((child.stdout.close_calls,child.stderr.close_calls,selector.close_calls),(1,1,1))
        child = Child(); child.stderr.close_error = True
        with command_seam(child=child, stderr=(b"x" * OBSERVER.MAX_CAPTURE,b"")):
            self.assertEqual(OBSERVER.command(["python"], ROOT, 10)["failure"], "output-bound")

    def test_actual_command_timeout_owned_wait_and_discard(self):
        """Require the original fake owner kill/wait observation after expiry and never infer cleanup from the signal alone."""
        child = Child(code=None)
        with command_seam(child=child, clock=[0,0,11]) as (_child, _selector, _spawn):
            result = OBSERVER.command(["python"], ROOT, 10)
            self.assertEqual(result, {"exit_code":-9,"failure":"command-timeout","output":b""})
            self.assertEqual(child.kills, 1)
            self.assertEqual(child.waits, [1,0])

    def test_observe_retains_first_rejection_only_with_unchanged_sources(self):
        """Let the actual source fence retain the first stat tuple only after both source observations agree."""
        value = rejected(); value["identity"] = None
        with observe_seam(failure=OBSERVER.ObservationStop(value)) as (captures, probe):
            result = OBSERVER.observe(ROOT, COMMIT, COMMIT)
            self.assertEqual(result, rejected())
            self.assertEqual(captures.call_count, 2)
            self.assertEqual(probe.call_count, 1)

    def test_changed_or_unverified_sources_revoke_object_claims(self):
        """A changed pin or failed post-observation source capture must erase first-rejection facts and retain explicit identity state."""
        after = capture(); after["inputs"][OBSERVER.INPUT_KEYS[0]]["sha256"] = "d" * 64
        for later, state in ((after,"changed"), (OSError("private-after-detail"),"unverified")):
            with self.subTest(state=state), observe_seam(after=later, failure=OBSERVER.ObservationStop(rejected())):
                result = OBSERVER.observe(ROOT, COMMIT, COMMIT)
                self.assert_unavailable(result)
                self.assertEqual(result["identity"]["before_after"], state)

    def test_no_rejection_and_unsupported_host_remain_unqualified(self):
        """The empty diagnostic alternative is not a pass, and unsupported ordinary hosts dispatch no Python probe."""
        with observe_seam():
            value = OBSERVER.observe(ROOT, COMMIT, COMMIT)
            self.assertEqual(value["outcome"], "no-rejection")
            self.assertEqual(value["identity"]["before_after"], "unchanged")
        with observe_seam(platform="darwin") as (_captures, probe):
            self.assert_unavailable(OBSERVER.observe(ROOT, COMMIT, COMMIT))
            probe.assert_not_called()

    def test_real_public_source_hashes_and_tested_head_fence(self):
        """Keep historical production refusal; mock fixture pins only for real reads and HEAD/hash fences, never current observer qualification."""
        historical = copy.deepcopy(OBSERVER.EXPECTED_PROTECTED)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for name in OBSERVER.INPUT_KEYS:
                destination = root / name; destination.parent.mkdir(parents=True,exist_ok=True)
                shutil.copyfile(ROOT / name, destination)
            fixture_protected = {name: OBSERVER.read_source(root / name) for name in historical}
            for name in historical:
                self.assertNotEqual(fixture_protected[name], historical[name])
            response = {"exit_code":0,"failure":None,"output":(COMMIT + "\n").encode()}
            with mock.patch.object(OBSERVER, "command", return_value=response), mock.patch.object(OBSERVER.time,"monotonic",return_value=0):
                self.assert_unavailable(self.stop(OBSERVER.capture_identity,root,COMMIT,COMMIT,10))
                self.assertEqual(OBSERVER.EXPECTED_PROTECTED, historical)
                # The override is this synthetic source-reader fixture only.
                # Production bytes/pins and the historical stat algorithm stay unchanged.
                with mock.patch.object(OBSERVER, "EXPECTED_PROTECTED", fixture_protected):
                    captured = OBSERVER.capture_identity(root, COMMIT, COMMIT, 10)
                    self.assertEqual(set(captured["inputs"]), set(OBSERVER.INPUT_KEYS))
                    for name in OBSERVER.INPUT_KEYS:
                        self.assertEqual(captured["inputs"][name], OBSERVER.read_source(root/name))
                    with mock.patch.object(OBSERVER, "command", return_value={**response,"output":("b" * 40 + "\n").encode()}), mock.patch.object(OBSERVER, "read_source", wraps=OBSERVER.read_source) as reader:
                        self.assert_unavailable(self.stop(OBSERVER.capture_identity,root,COMMIT,COMMIT,10))
                        reader.assert_not_called()
                    changed_path = root / OBSERVER.INPUT_KEYS[2]
                    changed = bytearray(changed_path.read_bytes()); changed[0] ^= 1
                    changed_path.write_bytes(changed)
                    self.assert_unavailable(self.stop(OBSERVER.capture_identity,root,COMMIT,COMMIT,10))
            self.assertEqual(OBSERVER.EXPECTED_PROTECTED, historical)

    def test_identity_expiry_prevents_command(self):
        """An already-expired source observation cannot dispatch even the ordinary Git identity command."""
        with mock.patch.object(OBSERVER.time,"monotonic",return_value=10), mock.patch.object(OBSERVER,"command") as command:
            self.assert_unavailable(self.stop(OBSERVER.capture_identity,ROOT,COMMIT,COMMIT,10), "qualification-budget", "deadline-expired")
            command.assert_not_called()

    def test_private_canonical_no_replacement_publication(self):
        """Create one private canonical sidecar and reject an existing output directory without replacing its bytes."""
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary) / "out"
            value = rejected(); OBSERVER.publish(destination,value)
            published = destination / OBSERVER.OUTPUT
            self.assertEqual(published.read_bytes(),OBSERVER.canonical_bytes(value))
            self.assertEqual(stat.S_IMODE(destination.stat().st_mode),0o700)
            self.assertEqual(stat.S_IMODE(published.stat().st_mode),0o600)
            with self.assertRaises(FileExistsError): OBSERVER.publish(destination,OBSERVER.empty_observation())
            self.assertEqual(published.read_bytes(),OBSERVER.canonical_bytes(value))

    def test_main_exit_classes_and_generic_private_fault(self):
        """Require availability-only exit0, unqualified exit2, and generic exit1 without printing private failure details."""
        for value, status in ((rejected(),0),(OBSERVER.empty_observation(),2),({**OBSERVER.empty_observation("no-rejection"),"identity":identity()},2)):
            with self.subTest(status=status), tempfile.TemporaryDirectory() as temporary, mock.patch.object(OBSERVER,"observe",return_value=value), contextlib.redirect_stdout(io.StringIO()) as output:
                self.assertEqual(OBSERVER.main(["--output-dir",str(Path(temporary)/"out"),"--requested-commit",COMMIT,"--tested-commit",COMMIT]),status)
                self.assertNotIn("passed",output.getvalue())
        with mock.patch.object(OBSERVER,"observe",side_effect=OSError("private-path-token")), contextlib.redirect_stderr(io.StringIO()) as output:
            self.assertEqual(OBSERVER.main(["--output-dir","unused","--requested-commit",COMMIT,"--tested-commit",COMMIT]),1)
            self.assertEqual(output.getvalue(),"Stdlib gate observation unavailable.\n")

    def test_post_link_cleanup_fault_cannot_earn_availability(self):
        """Keep actual nonzero producer status authoritative when an unlink fault leaves a canonical-looking hardlink behind."""
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary)/"out"
            with mock.patch.object(OBSERVER,"observe",return_value=rejected()), mock.patch.object(OBSERVER.os,"unlink",side_effect=OSError("private-cleanup-detail")), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(OBSERVER.main(["--output-dir",str(destination),"--requested-commit",COMMIT,"--tested-commit",COMMIT]),1)
            self.assertEqual((destination/OBSERVER.OUTPUT).read_bytes(),OBSERVER.canonical_bytes(rejected()))


if __name__ == "__main__":
    unittest.main()
