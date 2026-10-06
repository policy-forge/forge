#!/usr/bin/env python3
"""Mocked metadata/publication controls; these do not qualify Linux or native denial."""
import copy
import json
import os
from pathlib import Path
import stat
import sys
import tempfile
import time
import types
import unittest
from unittest import mock

import observe_workspace_stdlib_link_chain as subject


ROOT = "/trusted/lib/python"
ALIAS = ROOT + "/alias.py"
TARGET = ROOT + "/target.py"
ROOTS = [ROOT, ROOT + "/lib-dynload"]


def info(kind, inode, *, uid=0, mode=0o755, size=0, links=1):
    """Produce explicitly synthetic native stat values without supplying file contents."""
    return types.SimpleNamespace(st_dev=1, st_ino=inode, st_mode=kind | mode, st_uid=uid,
        st_gid=0, st_size=size, st_mtime_ns=10, st_ctime_ns=20, st_nlink=links)


class MemoryOS:
    """Model only descriptor-relative metadata and link text; regular-byte reads fail."""

    O_PATH, O_NOFOLLOW, O_DIRECTORY, O_CLOEXEC = 1, 2, 4, 8

    def __init__(self):
        """Create a complete small synthetic root union and one root-owned leaf link."""
        self.nodes, self.links, self.fds = {}, {}, {}
        self.next_fd, self.opens, self.close_attempts = 10, [], []
        self.readlink_calls, self.fail_close, self.readlink_override = 0, False, None
        for index, path in enumerate(("/", "/trusted", "/trusted/lib", ROOT, ROOTS[1]), 1):
            self.nodes[path] = info(stat.S_IFDIR, index)
        self.nodes[TARGET] = info(stat.S_IFREG, 100, mode=0o644, size=57)
        self.link(ALIAS, b"target.py", 101)

    def link(self, path, raw, inode):
        """Register synthetic link text and matching stat size, never target bytes."""
        self.links[path] = raw
        self.nodes[path] = info(stat.S_IFLNK, inode, mode=0o777, size=len(raw))

    def path(self, name, parent):
        """Resolve an API argument within an already-held synthetic directory."""
        text = os.fsdecode(name)
        if text.startswith("/"):
            return os.path.normpath(text)
        return os.path.normpath(os.path.join(self.fds[parent], text))

    def open(self, name, flags, dir_fd=None):
        """Require no-follow metadata flags and refuse missing/wrong directory objects."""
        path = self.path(name, dir_fd)
        self.opens.append((path, flags))
        if path not in self.nodes:
            raise FileNotFoundError()
        if not flags & self.O_PATH or not flags & self.O_NOFOLLOW:
            raise AssertionError("not a metadata no-follow open")
        if flags & self.O_DIRECTORY and not stat.S_ISDIR(self.nodes[path].st_mode):
            raise NotADirectoryError()
        fd = self.next_fd
        self.next_fd += 1
        self.fds[fd] = path
        return fd

    def fstat(self, fd):
        """Return actual model generation metadata for the held descriptor."""
        return copy.copy(self.nodes[self.fds[fd]])

    def stat(self, name, dir_fd=None, follow_symlinks=True):
        """Reject any automatic link-follow request in the observation path."""
        if follow_symlinks:
            raise AssertionError("automatic follow")
        path = self.path(name, dir_fd)
        if path not in self.nodes:
            raise FileNotFoundError()
        return copy.copy(self.nodes[path])

    def readlink(self, name, dir_fd=None):
        """Allow only empty-name readlink of a held link descriptor."""
        if name != b"":
            raise AssertionError("not held readlink")
        self.readlink_calls += 1
        if self.readlink_override is not None:
            return self.readlink_override
        return self.links[self.fds[dir_fd]]

    def close(self, fd):
        """Track every independent close attempt and inject unknown ownership when requested."""
        self.close_attempts.append(fd)
        if self.fail_close:
            raise OSError()
        del self.fds[fd]

    def read(self, _fd, _size):
        """Make any terminal file-content access a genuine mocked-control failure."""
        raise AssertionError("target contents were read")


def selected(api):
    """Return the synthetic actual discovery tuple for its currently held generation."""
    return Path(ALIAS), copy.copy(api.nodes[ALIAS]), 0, Path(ROOT)


def run_trace(api):
    """Run actual production chain/recheck functions against metadata-only fakes."""
    inspector = subject.Inspector(time.monotonic() + 100, api=api)
    try:
        reason = inspector.trace(selected(api), ROOTS)
        inspector.recheck()
        return reason, dict(inspector.facts), inspector
    finally:
        inspector.close_all()


def identity():
    """Return an explicitly synthetic exact closed source context for decoder controls."""
    return {"requested_commit": "a" * 40, "tested_commit": "b" * 40,
            "before_after": "unchanged", "inputs": {
                name: copy.deepcopy(subject.EXPECTED_PROTECTED.get(name, {"bytes": 100, "sha256": "c" * 64}))
                for name in subject.INPUT_KEYS}}


def observed():
    """Build a synthetic valid shape, not a claimed filesystem or hosted observation."""
    result = subject.empty()
    result.update(outcome="observed", first_fault="target-observed", root_index=0,
        entry_id="e" * 64, first_link_form="relative", hops_observed=1,
        cycle_detected=False, chain_id="f" * 64, target_id="d" * 64,
        target_kind="regular", target_uid_is_root=True, target_mode_022_bits=0,
        target_in_roots=True, parents_root_owned=True, parents_nonwritable=True,
        links_root_owned=True, object_stat_stable=True, identity=identity())
    return result


class GateStop(Exception):
    """Model the original observer's exact first-rejection exception interface."""

    def __init__(self, value):
        """Keep only synthetic original observer fields for discovery delegation."""
        self.value = value


class FakeGate:
    """Expose the already-consumed old observer ports without launching any command."""

    ObservationStop = GateStop

    def __init__(self, api):
        """Retain source-change and actual-first-rejection fault switches."""
        self.api, self.calls, self.rejections = api, 0, 0
        self.changed, self.stable, self.first_nonlink = False, True, False

    def capture_identity(self, _root, _requested, _tested, _deadline):
        """Return the original four-source context with an optional second-pass change."""
        self.calls += 1
        value = identity()
        value["before_after"] = "unverified"
        del value["inputs"][subject.INPUT_KEYS[0]]
        if self.changed and self.calls > 1:
            value["inputs"][".github/workflows/workspace-verification.yml"]["sha256"] = "1" * 64
        return value

    def read_source(self, _path):
        """Supply a synthetic new observer source pin, never a target content hash."""
        return {"bytes": 100, "sha256": "c" * 64}

    def probe_paths(self, _root, _deadline):
        """Return fixed synthetic roots; no Python/native tool is launched by this fake."""
        return ROOTS

    def reject_object(self, _path, _info, phase, reason, predicate, _index, _root, _end):
        """Record original call preservation before raising its unchanged first outcome."""
        self.rejections += 1
        raise GateStop({"outcome": "rejected", "object_stat_stable": self.stable,
                       "phase": phase, "reason": reason, "source_predicate": predicate})

    def stdlib_walk(self, _paths, deadline):
        """Call only the actual first rejection, refusing a search past unrelated failure."""
        if self.first_nonlink:
            self.reject_object(Path(TARGET), self.api.nodes[TARGET], "stdlib-entry", "worker-writable",
                               "entry-mode-022", 0, Path(ROOT), deadline)
        self.reject_object(Path(ALIAS), self.api.nodes[ALIAS], "stdlib-entry", "unsupported-link",
                           "entry-kind", 0, Path(ROOT), deadline)


class ChainControls(unittest.TestCase):
    """Exercise actual bounded descriptor/chain functions without Linux qualification."""

    def test_relative_owned_leaf_is_observed_without_content_read(self):
        """A genuine production trace records one relative metadata-only in-root endpoint."""
        api = MemoryOS()
        reason, facts, _inspector = run_trace(api)
        self.assertEqual(reason, "target-observed")
        self.assertEqual(facts["hops_observed"], 1)
        self.assertEqual(facts["first_link_form"], "relative")
        self.assertIs(facts["target_in_roots"], True)
        self.assertEqual(api.fds, {})

    def test_absolute_external_target_is_observed_but_not_admitted(self):
        """Held metadata can distinguish a terminal outside the exact root union."""
        api = MemoryOS()
        api.nodes["/outside"] = info(stat.S_IFDIR, 200)
        api.nodes["/outside/value.py"] = info(stat.S_IFREG, 201, mode=0o644)
        api.link(ALIAS, b"/outside/value.py", 101)
        reason, facts, _inspector = run_trace(api)
        self.assertEqual(reason, "target-outside-roots")
        self.assertEqual(facts["first_link_form"], "absolute")
        self.assertIs(facts["target_in_roots"], False)

    def test_dotdot_checks_actual_parent_before_resolution(self):
        """A safe parent traversal is processed component-wise rather than stripped early."""
        api = MemoryOS()
        api.link(ALIAS, b"lib-dynload/../target.py", 101)
        reason, _facts, _inspector = run_trace(api)
        self.assertEqual(reason, "target-observed")
        self.assertIn(ROOTS[1], [path for path, _flags in api.opens])

    def test_intermediate_directory_link_is_not_followed(self):
        """A link used as a directory endpoint yields a fixed prefix observation only."""
        api = MemoryOS()
        api.link(ALIAS, b"through/target.py", 101)
        api.link(ROOT + "/through", b"lib-dynload", 202)
        reason, facts, _inspector = run_trace(api)
        self.assertEqual(reason, "intermediate-link-unsupported")
        self.assertIsNone(facts["target_kind"])

    def test_target_owner_and_write_bits_remain_actual_metadata(self):
        """Wrong-owner and nonlink022 targets are separate observed refusal facts."""
        for uid, mode, reason in ((42, 0o644, "target-not-root-owned"), (0, 0o666, "target-worker-writable")):
            with self.subTest(uid=uid, mode=mode):
                api = MemoryOS()
                api.nodes[TARGET] = info(stat.S_IFREG, 100, uid=uid, mode=mode)
                actual, facts, _inspector = run_trace(api)
                self.assertEqual(actual, reason)
                self.assertEqual(facts["target_uid_is_root"], uid == 0)
                self.assertEqual(facts["target_mode_022_bits"], mode & 0o022)

    def test_symlink_0777_does_not_become_target_worker_writable(self):
        """Link permission bits are not substituted for the final regular-file predicate."""
        api = MemoryOS()
        self.assertEqual(api.nodes[ALIAS].st_mode & 0o022, 18)
        reason, facts, _inspector = run_trace(api)
        self.assertEqual(reason, "target-observed")
        self.assertEqual(facts["target_mode_022_bits"], 0)

    def test_target_unsafe_ancestor_stops_before_child_open(self):
        """A checked nonroot or writable parent is retained as a prefix refusal."""
        for uid, mode, reason in ((42, 0o755, "ancestor-not-root-owned"), (0, 0o777, "ancestor-worker-writable")):
            with self.subTest(uid=uid, mode=mode):
                api = MemoryOS()
                api.nodes["/outside"] = info(stat.S_IFDIR, 203, uid=uid, mode=mode)
                api.link(ALIAS, b"/outside/not-opened.py", 101)
                actual, facts, _inspector = run_trace(api)
                self.assertEqual(actual, reason)
                self.assertNotIn("/outside/not-opened.py", [path for path, _flags in api.opens])
                self.assertIsNone(facts["target_kind"])

    def test_dangling_terminal_has_null_owner_and_rechecked_absence(self):
        """A missing terminal produces no fabricated zero/false target facts."""
        api = MemoryOS()
        api.link(ALIAS, b"missing.py", 101)
        reason, facts, inspector = run_trace(api)
        self.assertEqual(reason, "target-missing")
        self.assertEqual(facts["target_kind"], "missing")
        self.assertIsNone(facts["target_uid_is_root"])
        self.assertIsNone(facts["target_in_roots"])
        self.assertIn(ROOT + "/missing.py", inspector.absences)

    def test_missing_component_creation_revokes_absence(self):
        """The first actual missing component becoming present changes the proof."""
        api = MemoryOS()
        api.link(ALIAS, b"missing/target.py", 101)
        inspector = subject.Inspector(time.monotonic() + 100, api=api)
        try:
            self.assertEqual(inspector.trace(selected(api), ROOTS), "target-missing")
            api.nodes[ROOT + "/missing"] = info(stat.S_IFDIR, 204)
            with self.assertRaises(subject.Stop) as caught:
                inspector.recheck()
            self.assertEqual(caught.exception.code, "identity-changed")
        finally:
            inspector.close_all()

    def test_removed_original_is_change_not_a_missing_target(self):
        """Discovery's original generation must still exist before chain facts are retained."""
        api = MemoryOS()
        selection = selected(api)
        del api.nodes[ALIAS]
        inspector = subject.Inspector(time.monotonic() + 100, api=api)
        try:
            with self.assertRaises(subject.Stop) as caught:
                inspector.trace(selection, ROOTS)
            self.assertEqual(caught.exception.code, "identity-changed")
        finally:
            inspector.close_all()

    def test_cycle_is_actual_repeated_instance_and_spelling(self):
        """A self-reference is a cycle observation, not an inferred kernel error."""
        api = MemoryOS()
        api.link(ALIAS, b"alias.py", 101)
        reason, facts, _inspector = run_trace(api)
        self.assertEqual(reason, "cycle-observed")
        self.assertIs(facts["cycle_detected"], True)
        self.assertEqual(facts["hops_observed"], 1)

    def test_hardlink_alias_with_different_parent_is_not_a_cycle(self):
        """Same inode under a different relative-link context does not invent a repeated state."""
        api = MemoryOS()
        api.nodes[ROOT + "/inner"] = info(stat.S_IFDIR, 205)
        api.nodes[ROOT + "/inner/inner"] = info(stat.S_IFDIR, 206)
        api.nodes[ROOT + "/inner/inner/alias.py"] = info(stat.S_IFREG, 207, mode=0o644)
        api.link(ALIAS, b"inner/alias.py", 101)
        api.link(ROOT + "/inner/alias.py", b"inner/alias.py", 101)
        api.nodes[ALIAS].st_nlink = 2
        api.nodes[ROOT + "/inner/alias.py"].st_nlink = 2
        reason, facts, _inspector = run_trace(api)
        self.assertEqual(reason, "target-observed")
        self.assertEqual(facts["hops_observed"], 2)
        self.assertIs(facts["cycle_detected"], False)

    def test_sixteen_hops_and_seventeenth_have_distinct_endpoints(self):
        """The exact chain cap admits a terminal observation but never mislabels overflow a cycle."""
        for count, expected in ((16, "target-observed"), (17, "hop-bound")):
            with self.subTest(count=count):
                api = MemoryOS()
                for index in range(count):
                    path = ALIAS if index == 0 else ROOT + "/hop" + str(index)
                    target = b"target.py" if index == count - 1 else ("hop" + str(index + 1)).encode()
                    api.link(path, target, 300 + index)
                reason, facts, _inspector = run_trace(api)
                self.assertEqual(reason, expected)
                self.assertEqual(facts["hops_observed"], min(count, 16))
                self.assertIs(facts["cycle_detected"], False)

    def test_zero_size_or_different_returned_link_length_is_unavailable(self):
        """Python readlink is not treated as a bounded-buffer or exact-size primitive."""
        for size, raw in ((0, b"target.py"), (9, b"target.py-extra")):
            with self.subTest(size=size):
                api = MemoryOS()
                api.nodes[ALIAS].st_size = size
                api.readlink_override = raw
                with self.assertRaises(subject.Stop) as caught:
                    run_trace(api)
                self.assertEqual(caught.exception.code, "object-observation-unverified")
                self.assertEqual(api.readlink_calls, 0 if size == 0 else 1)

    def test_link_size_cap_prevents_readlink_dispatch(self):
        """An oversized held link is rejected before reading its text."""
        api = MemoryOS()
        api.nodes[ALIAS].st_size = subject.MAX_PATH + 1
        with self.assertRaises(subject.Stop):
            run_trace(api)
        self.assertEqual(api.readlink_calls, 0)

    def test_full_generation_change_with_same_bytes_is_refused(self):
        """A changed inode with unchanged link text revokes original identity."""
        api = MemoryOS()
        selection = selected(api)
        api.nodes[ALIAS].st_ino += 1
        inspector = subject.Inspector(time.monotonic() + 100, api=api)
        try:
            with self.assertRaises(subject.Stop) as caught:
                inspector.trace(selection, ROOTS)
            self.assertTrue(caught.exception.changed)
        finally:
            inspector.close_all()

    def test_retarget_and_terminal_generation_change_revoke_recheck(self):
        """Stable-looking endpoint counts do not hide changed link or terminal generations."""
        for changed_link in (True, False):
            with self.subTest(link=changed_link):
                api = MemoryOS()
                inspector = subject.Inspector(time.monotonic() + 100, api=api)
                try:
                    self.assertEqual(inspector.trace(selected(api), ROOTS), "target-observed")
                    if changed_link:
                        api.links[ALIAS] = b"other_.py"
                    else:
                        api.nodes[TARGET].st_ctime_ns += 1
                    with self.assertRaises(subject.Stop):
                        inspector.recheck()
                finally:
                    inspector.close_all()

    def test_deadline_and_component_budget_fence_before_open(self):
        """Expired or exhausted work never begins a new descriptor operation."""
        api = MemoryOS()
        inspector = subject.Inspector(0, api=api, now=lambda: 0)
        with self.assertRaises(subject.Stop):
            inspector.open("/")
        self.assertEqual(api.opens, [])
        inspector = subject.Inspector(100, api=api, now=lambda: 0)
        inspector.operations = subject.MAX_COMPONENTS
        with self.assertRaises(subject.Stop):
            inspector.open("/")
        self.assertEqual(api.opens, [])

    def test_descriptor_and_encoded_record_caps_precede_growth(self):
        """A full descriptor ledger or complete encoded row budget refuses retention."""
        api = MemoryOS()
        inspector = subject.Inspector(time.monotonic() + 100, api=api)
        inspector.fds = set(range(subject.MAX_FDS))
        with self.assertRaises(subject.Stop):
            inspector.open("/")
        self.assertEqual(api.opens, [])
        inspector.fds.clear()
        with mock.patch.object(subject, "MAX_PROOF", 1), self.assertRaises(subject.Stop):
            inspector.remember("/", api.nodes["/"])
        self.assertEqual(inspector.records, {})

    def test_missing_linux_flag_never_falls_back(self):
        """Unavailable O_PATH prevents a weaker automatic-follow observation."""
        api = MemoryOS()
        api.O_PATH = 0
        with self.assertRaises(subject.Stop) as caught:
            subject.Inspector(100, api=api)
        self.assertEqual(caught.exception.code, "unsupported-primitive")
        self.assertEqual(api.opens, [])

    def test_independent_closes_preserve_unknown_ownership(self):
        """All references receive close attempts even after an earlier close fails."""
        api = MemoryOS()
        inspector = subject.Inspector(time.monotonic() + 100, api=api)
        first, second = inspector.open("/"), inspector.open("/")
        api.fail_close = True
        self.assertFalse(inspector.close_all())
        self.assertIn(first, api.close_attempts)
        self.assertIn(second, api.close_attempts)
        self.assertEqual(inspector.fds, {first, second})
        with self.assertRaises(subject.Stop) as caught:
            inspector.open("/")
        self.assertEqual(caught.exception.code, "descriptor-close-unverified")
        self.assertEqual(len(api.opens), 2)


class ContextControls(unittest.TestCase):
    """Exercise actual discovery/source lifetime and closed DTO validation with no tool execution."""

    def test_load_exact_protected_gate_buffer_without_tool_dispatch(self):
        """The actual hash/read/compile loader consumes its exact five-file ordinary fixture."""
        source = Path(subject.__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for relative in subject.INPUT_KEYS:
                original = source / relative
                with original.open("rb") as stream:
                    raw = stream.read(subject.MAX_SOURCE + 1)
                self.assertLessEqual(len(raw), subject.MAX_SOURCE)
                target = root / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(raw)
            self.assertEqual(sorted(path.relative_to(root).as_posix() for path in root.rglob("*") if path.is_file()),
                             sorted(subject.INPUT_KEYS))
            with mock.patch("subprocess.Popen", side_effect=AssertionError("unexpected tool dispatch")) as dispatched:
                gate = subject.load_gate(root, time.monotonic() + 10)
            dispatched.assert_not_called()
            self.assertIsInstance(gate, types.ModuleType)
            self.assertEqual(gate.__name__, "_sealed_stdlib_gate")
            self.assertEqual(gate.__file__, str(root / subject.INPUT_KEYS[1]))
            self.assertEqual(gate.SCHEMA, "forge.stdlib-gate-observation/2")
            self.assertEqual(gate.INPUT_KEYS, subject.INPUT_KEYS[1:])
            self.assertEqual(gate.EXPECTED_PROTECTED,
                {key: subject.EXPECTED_PROTECTED[key] for key in gate.EXPECTED_PROTECTED})

    def test_current_fixed_helper_pins_match_the_actual_source_buffers(self):
        """Bind both diagnostic readers to the same actual helpers after integration."""
        import hashlib
        root = Path(subject.__file__).resolve().parents[1]
        gate = subject.load_gate(root, time.monotonic() + 10)
        for name, expected in gate.EXPECTED_PROTECTED.items():
            raw = (root / name).read_bytes()
            actual = {"bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}
            self.assertEqual(expected, actual)
            self.assertEqual(subject.EXPECTED_PROTECTED[name], actual)

    def test_same_size_gate_corruption_is_refused_before_compilation(self):
        """Same-size changed source never becomes an executable gate or dispatched tool."""
        source = Path(subject.__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for relative in subject.INPUT_KEYS:
                with (source / relative).open("rb") as stream:
                    raw = stream.read(subject.MAX_SOURCE + 1)
                self.assertLessEqual(len(raw), subject.MAX_SOURCE)
                target = root / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(raw)
            target = root / subject.INPUT_KEYS[1]
            raw = target.read_bytes()
            self.assertEqual(len(raw), subject.EXPECTED_GATE["bytes"])
            changed = bytes([raw[0] ^ 1]) + raw[1:]
            self.assertEqual(len(changed), len(raw))
            self.assertNotEqual(changed, raw)
            target.write_bytes(changed)
            with mock.patch("builtins.compile", wraps=compile) as compiled, mock.patch("subprocess.Popen", side_effect=AssertionError("unexpected tool dispatch")) as dispatched:
                with self.assertRaises(subject.Stop) as caught:
                    subject.load_gate(root, time.monotonic() + 10)
            self.assertEqual(caught.exception.code, "source-unverified")
            compiled.assert_not_called()
            dispatched.assert_not_called()

    def test_discovery_preserves_original_first_rejection_call(self):
        """Only the freshly invoked original link rejection is captured, then callback restored."""
        gate = FakeGate(MemoryOS())
        original = gate.reject_object
        selection = subject.discover(gate, ROOTS, time.monotonic() + 10)
        self.assertEqual(str(selection[0]), ALIAS)
        self.assertEqual(gate.rejections, 1)
        self.assertEqual(gate.reject_object, original)

    def test_unrelated_or_unstable_first_failure_never_searches_for_a_link(self):
        """A different first predicate or unstable source object yields no later guessed entry."""
        for first_nonlink, stable in ((True, True), (False, False)):
            with self.subTest(first_nonlink=first_nonlink):
                gate = FakeGate(MemoryOS())
                gate.first_nonlink, gate.stable = first_nonlink, stable
                with self.assertRaises(subject.Stop) as caught:
                    subject.discover(gate, ROOTS, time.monotonic() + 10)
                self.assertEqual(caught.exception.code, "entry-not-selected")
                self.assertEqual(gate.rejections, 1)

    def test_actual_observe_source_change_discards_all_object_facts(self):
        """Second source context mismatch revokes otherwise valid mocked prefix metadata."""
        api = MemoryOS()
        gate = FakeGate(api)
        gate.changed = True
        with mock.patch.object(subject.sys, "platform", "linux"), mock.patch.object(subject.os, "uname", return_value=types.SimpleNamespace(machine="x86_64")), mock.patch.object(subject.os, "getuid", return_value=1000), mock.patch.object(subject.os, "geteuid", return_value=1000), mock.patch.object(subject.os, "getgid", return_value=1000), mock.patch.object(subject.os, "getegid", return_value=1000):
            value = subject.observe(Path("/repo"), "a" * 40, "b" * 40, gate=gate,
                inspector_type=lambda deadline, now: subject.Inspector(deadline, api=api, now=now))
        self.assertEqual(value["outcome"], "unavailable")
        self.assertEqual(value["first_fault"], "source-unverified")
        self.assertEqual(value["identity"]["before_after"], "changed")
        self.assertTrue(all(value[name] is None for name in subject.OBJECT_FIELDS))

    def test_closed_decoder_accepts_observed_and_nullable_unavailable(self):
        """Synthetic complete shape and explicit null unknown remain distinguishable."""
        self.assertEqual(subject.validate(observed())["outcome"], "observed")
        value = subject.empty("unsupported-primitive")
        self.assertIsNone(subject.validate(value)["target_uid_is_root"])

    def test_unknown_private_fields_and_bool_counts_are_rejected(self):
        """Closed DTO refuses path disclosure and boolean masquerading as numeric metadata."""
        for field, value in (("private_path", "/secret"), ("root_index", True), ("hops_observed", True), ("target_mode_022_bits", True)):
            with self.subTest(field=field):
                record = observed()
                record[field] = value
                with self.assertRaises(ValueError):
                    subject.validate(record)

    def test_false_is_not_an_unknown_replacement(self):
        """Unavailable objects cannot gain fabricated false permission or membership facts."""
        value = subject.empty("unsupported-primitive")
        value["target_in_roots"] = False
        with self.assertRaises(ValueError):
            subject.validate(value)

    def test_cycle_owner_mode_and_missing_pairs_are_consumer_checked(self):
        """Contradictory fixed endpoints are rejected by the real semantic validator."""
        cases = ({"first_fault": "cycle-observed"}, {"first_fault": "target-not-root-owned"},
                 {"first_fault": "target-worker-writable"}, {"first_fault": "target-missing"},
                 {"hops_observed": 0}, {"target_kind": None})
        for delta in cases:
            with self.subTest(delta=delta):
                value = observed()
                value.update(delta)
                with self.assertRaises(ValueError):
                    subject.validate(value)

    def test_observed_terminal_predicate_order_cannot_be_forged(self):
        """A successful endpoint cannot bypass preceding owner, mode, kind or containment facts."""
        cases = ({"target_uid_is_root": False}, {"target_mode_022_bits": 2},
                 {"target_in_roots": False}, {"target_kind": "directory"},
                 {"first_fault": "target-outside-roots", "target_in_roots": False, "target_uid_is_root": False},
                 {"first_fault": "target-outside-roots", "target_in_roots": False, "target_mode_022_bits": 16},
                 {"first_fault": "target-kind-unsupported"},
                 {"first_fault": "target-kind-unsupported", "target_kind": "directory", "target_mode_022_bits": 2},
                 {"first_fault": "target-kind-unsupported", "target_kind": "directory", "target_uid_is_root": False})
        for delta in cases:
            with self.subTest(delta=delta):
                value = observed()
                value.update(delta)
                with self.assertRaises(ValueError):
                    subject.validate(value)

    def test_observed_faults_require_matching_prefix_and_reachable_endpoint(self):
        """Unavailable codes and contradictory parent/link prefix facts cannot masquerade as observed."""
        cases = ({"first_fault": "entry-not-selected"}, {"first_fault": "source-unverified"},
                 {"parents_root_owned": False}, {"parents_nonwritable": False},
                 {"links_root_owned": False}, {"cycle_detected": True},
                 {"first_fault": "ancestor-not-root-owned", "target_kind": None,
                  "target_id": None, "target_uid_is_root": None, "target_mode_022_bits": None,
                  "target_in_roots": None},
                 {"hops_observed": 0, "chain_id": None, "first_link_form": None, "cycle_detected": None})
        for delta in cases:
            with self.subTest(delta=delta):
                value = observed()
                value.update(delta)
                with self.assertRaises(ValueError):
                    subject.validate(value)

    def test_actual_trace_endpoints_satisfy_closed_validator(self):
        """Actual metadata-only trace output retains native predicate order for every constructed endpoint."""
        for endpoint in ("regular", "owner", "writable", "directory", "other", "missing", "cycle", "link-owner", "ancestor-owner", "ancestor-writable", "intermediate"):
            with self.subTest(endpoint=endpoint):
                api = MemoryOS()
                if endpoint == "owner":
                    api.nodes[TARGET].st_uid = 42
                elif endpoint == "writable":
                    api.nodes[TARGET].st_mode = stat.S_IFREG | 0o666
                elif endpoint == "directory":
                    api.nodes[TARGET].st_mode = stat.S_IFDIR | 0o755
                elif endpoint == "other":
                    api.nodes[TARGET].st_mode = stat.S_IFIFO | 0o644
                elif endpoint == "missing":
                    del api.nodes[TARGET]
                elif endpoint == "cycle":
                    api.link(ALIAS, b"alias.py", 101)
                elif endpoint == "link-owner":
                    api.link(TARGET, b"not-observed.py", 100)
                    api.nodes[TARGET].st_uid = 42
                elif endpoint.startswith("ancestor-"):
                    api.nodes["/outside"] = info(stat.S_IFDIR, 203,
                        uid=42 if endpoint == "ancestor-owner" else 0,
                        mode=0o755 if endpoint == "ancestor-owner" else 0o777)
                    api.link(ALIAS, b"/outside/not-opened.py", 101)
                elif endpoint == "intermediate":
                    api.link(ALIAS, b"through/target.py", 101)
                    api.link(ROOT + "/through", b"lib-dynload", 202)
                reason, facts, _inspector = run_trace(api)
                value = {"schema_version": subject.SCHEMA, "outcome": "observed",
                         "first_fault": reason, "identity": identity(), **facts,
                         "object_stat_stable": True}
                self.assertEqual(subject.validate(value), value)

    def test_wrong_protected_source_and_changed_context_cannot_bind_facts(self):
        """A shape-valid pin or claimed observations cannot replace exact protected origins."""
        for changed in (False, True):
            with self.subTest(changed=changed):
                value = observed()
                if changed:
                    value["identity"]["before_after"] = "changed"
                else:
                    value["identity"]["inputs"]["scripts/test_workspace_os_denial.py"]["sha256"] = "0" * 64
                with self.assertRaises(ValueError):
                    subject.validate(value)

    def test_entire_encoded_sidecar_cap_is_applied(self):
        """Complete identity plus facts must fit the same bounded sidecar envelope."""
        value = observed()
        with mock.patch.object(subject, "MAX_SIDECAR", len(subject.canonical(value)) - 1), self.assertRaises(ValueError):
            subject.validate(value)


class PublicationControls(unittest.TestCase):
    """Use only generated private sidecars and mocked main ports, never native execution."""

    def test_new_sidecar_is_exact_bounded_private_and_no_replacement(self):
        """An actual ordinary-file publication contains only the validated generated JSON."""
        with tempfile.TemporaryDirectory() as private:
            path = Path(private) / "new"
            value = subject.empty("unsupported-primitive")
            subject.publish(path, value)
            self.assertEqual((path / subject.OUTPUT).read_bytes(), subject.canonical(value))
            self.assertEqual(stat.S_IMODE(path.stat().st_mode), 0o700)
            with self.assertRaises(FileExistsError):
                subject.publish(path, value)

    def test_preexisting_sentinel_and_oversized_file_never_emit_fresh_flag(self):
        """Main's publication refusal preserves old bytes and never calls the positive flag port."""
        for raw in (b"sentinel", b"x" * (subject.MAX_SIDECAR + 1)):
            with self.subTest(size=len(raw)), tempfile.TemporaryDirectory() as private:
                path = Path(private) / "existing"
                path.mkdir()
                saved = path / subject.OUTPUT
                saved.write_bytes(raw)
                argv = ["observer", "--output-dir", str(path), "--requested-commit", "a" * 40, "--tested-commit", "b" * 40]
                with mock.patch.object(subject.sys, "argv", argv), mock.patch.object(subject, "observe", return_value=subject.empty()), mock.patch.object(subject, "publication_flag") as flag, mock.patch.dict(subject.os.environ, {"GITHUB_ACTIONS": "true"}):
                    self.assertEqual(subject.main(), 1)
                flag.assert_not_called()
                self.assertEqual(saved.read_bytes(), raw)

    def test_publication_fault_never_calls_flag(self):
        """A real main control-flow fault before successful publication does not authorize upload."""
        argv = ["observer", "--output-dir", "unused", "--requested-commit", "a" * 40, "--tested-commit", "b" * 40]
        with mock.patch.object(subject.sys, "argv", argv), mock.patch.object(subject, "observe", return_value=subject.empty()), mock.patch.object(subject, "publish", side_effect=OSError()), mock.patch.object(subject, "publication_flag") as flag, mock.patch.dict(subject.os.environ, {"GITHUB_ACTIONS": "true"}):
            self.assertEqual(subject.main(), 1)
        flag.assert_not_called()

    def test_entropy_fault_closes_directory_and_never_calls_flag(self):
        """Failure after directory open retains raw ownership and still attempts its close."""
        with tempfile.TemporaryDirectory() as private:
            destination = Path(private) / "new"
            argv = ["observer", "--output-dir", str(destination), "--requested-commit", "a" * 40, "--tested-commit", "b" * 40]
            actual_close = subject.os.close
            with mock.patch.object(subject.sys, "argv", argv), mock.patch.object(subject, "observe", return_value=subject.empty()), mock.patch.object(subject.os, "urandom", side_effect=OSError()), mock.patch.object(subject.os, "close", wraps=actual_close) as closed, mock.patch.object(subject, "publication_flag") as flag, mock.patch.dict(subject.os.environ, {"GITHUB_ACTIONS": "true"}):
                self.assertEqual(subject.main(), 1)
            flag.assert_not_called()
            self.assertEqual(closed.call_count, 1)
            self.assertFalse((destination / subject.OUTPUT).exists())

    def test_raw_write_fault_and_short_write_keep_explicit_ownership(self):
        """Reached write faults close both descriptors; progressing short writes publish exact bytes."""
        for fault in ("exception", "zero", "oversize"):
            with self.subTest(fault=fault), tempfile.TemporaryDirectory() as private:
                destination = Path(private) / "new"
                argv = ["observer", "--output-dir", str(destination), "--requested-commit", "a" * 40, "--tested-commit", "b" * 40]
                actual_close = subject.os.close
                value = subject.empty()
                with mock.patch.object(subject.sys, "argv", argv), mock.patch.object(subject, "observe", return_value=value), mock.patch.object(subject.os, "write", side_effect=OSError() if fault == "exception" else None, return_value=0 if fault == "zero" else subject.MAX_SIDECAR + 1) as written, mock.patch.object(subject.os, "close", wraps=actual_close) as closed, mock.patch.object(subject, "publication_flag") as flag, mock.patch.dict(subject.os.environ, {"GITHUB_ACTIONS": "true"}):
                    self.assertEqual(subject.main(), 1)
                written.assert_called_once()
                flag.assert_not_called()
                self.assertEqual(closed.call_count, 2)
                self.assertEqual(len({call.args[0] for call in closed.call_args_list}), 2)
                self.assertFalse((destination / subject.OUTPUT).exists())
                self.assertEqual(list(destination.iterdir()), [])
        with tempfile.TemporaryDirectory() as private:
            destination = Path(private) / "new"
            value = subject.empty("unsupported-primitive")
            actual_write = subject.os.write
            with mock.patch.object(subject.os, "write", side_effect=lambda fd, raw: actual_write(fd, raw[:max(1, len(raw) // 2)])) as written:
                subject.publish(destination, value)
            self.assertGreater(written.call_count, 1)
            self.assertEqual((destination / subject.OUTPUT).read_bytes(), subject.canonical(value))

    def test_actual_publication_precedes_fixed_flag_and_postwrite_fault_is_honest(self):
        """A flag port fault can follow fresh publication; no stale-file retraction claim is made."""
        events = []
        argv = ["observer", "--output-dir", "unused", "--requested-commit", "a" * 40, "--tested-commit", "b" * 40]
        def publish(_path, _value):
            """Record the mocked actual-publication boundary before flag emission."""
            events.append("new-publication")
        def flag(_environment):
            """Retain a simulated persisted fixed line then expose a later close fault."""
            events.append(subject.FLAG)
            raise OSError()
        with mock.patch.object(subject.sys, "argv", argv), mock.patch.object(subject, "observe", return_value=subject.empty()), mock.patch.object(subject, "publish", side_effect=publish), mock.patch.object(subject, "publication_flag", side_effect=flag), mock.patch.dict(subject.os.environ, {"GITHUB_ACTIONS": "true"}):
            self.assertEqual(subject.main(), 1)
        self.assertEqual(events, ["new-publication", subject.FLAG])

    def test_unqualified_runner_output_does_not_authorize_upload(self):
        """Absent or nonrunner output context cannot emit a trusted positive step value."""
        self.assertFalse(subject.publication_flag({}))
        self.assertFalse(subject.publication_flag({"GITHUB_ACTIONS": "true"}))
        with mock.patch.object(subject.os, "open") as opened:
            self.assertFalse(subject.publication_flag({"GITHUB_ACTIONS": "true", "RUNNER_TEMP": "/runner", "GITHUB_OUTPUT": "/outside/output"}))
        opened.assert_not_called()

    def test_fixed_flag_complete_and_short_write_are_distinct(self):
        """The actual flag function emits only its fixed line and checks the complete write count."""
        for short in (False, True):
            with self.subTest(short=short):
                directory = info(stat.S_IFDIR, 1, uid=1000)
                regular = info(stat.S_IFREG, 2, uid=1000, mode=0o600)
                fd_stats = {10: directory, 20: directory, 30: directory, 40: regular}
                context = {"GITHUB_ACTIONS": "true", "RUNNER_TEMP": "/runner/_temp", "GITHUB_OUTPUT": "/runner/_temp/output"}
                with mock.patch.object(subject.os, "O_PATH", 1, create=True), mock.patch.object(subject.os, "open", side_effect=[10, 20, 30, 40]), mock.patch.object(subject.os, "fstat", side_effect=lambda fd: fd_stats[fd]), mock.patch.object(subject.os, "stat", return_value=regular), mock.patch.object(subject.os, "close") as closed, mock.patch.object(subject.os, "getuid", return_value=1000), mock.patch.object(subject.os, "write", return_value=len(subject.FLAG) - int(short)) as written:
                    self.assertEqual(subject.publication_flag(context), not short)
                written.assert_called_once_with(40, subject.FLAG)
                self.assertEqual({call.args[0] for call in closed.call_args_list}, {10, 20, 30, 40})

    def test_flag_file_mode_refusal_prevents_any_write(self):
        """A runner-owned but other-writable output file is not a qualified flag channel."""
        directory = info(stat.S_IFDIR, 1, uid=1000)
        regular = info(stat.S_IFREG, 2, uid=1000, mode=0o666)
        context = {"GITHUB_ACTIONS": "true", "RUNNER_TEMP": "/runner", "GITHUB_OUTPUT": "/runner/output"}
        with mock.patch.object(subject.os, "O_PATH", 1, create=True), mock.patch.object(subject.os, "open", side_effect=[10, 20]), mock.patch.object(subject.os, "fstat", return_value=directory), mock.patch.object(subject.os, "stat", return_value=regular), mock.patch.object(subject.os, "close"), mock.patch.object(subject.os, "getuid", return_value=1000), mock.patch.object(subject.os, "write") as written:
            self.assertFalse(subject.publication_flag(context))
        written.assert_not_called()

    def test_flag_postwrite_close_fault_attempts_other_owned_handles(self):
        """A persisted positive line remains tied to fresh publication while closes are independent."""
        directory = info(stat.S_IFDIR, 1, uid=1000)
        regular = info(stat.S_IFREG, 2, uid=1000, mode=0o600)
        fd_stats = {10: directory, 20: directory, 30: regular}
        context = {"GITHUB_ACTIONS": "true", "RUNNER_TEMP": "/runner", "GITHUB_OUTPUT": "/runner/output"}
        attempted = []
        def close(fd):
            """Inject a late file close fault without preventing the directory close observation."""
            attempted.append(fd)
            if fd == 30:
                raise OSError()
        with mock.patch.object(subject.os, "O_PATH", 1, create=True), mock.patch.object(subject.os, "open", side_effect=[10, 20, 30]), mock.patch.object(subject.os, "fstat", side_effect=lambda fd: fd_stats[fd]), mock.patch.object(subject.os, "stat", return_value=regular), mock.patch.object(subject.os, "close", side_effect=close), mock.patch.object(subject.os, "getuid", return_value=1000), mock.patch.object(subject.os, "write", return_value=len(subject.FLAG)) as written:
            with self.assertRaises(OSError):
                subject.publication_flag(context)
        written.assert_called_once_with(30, subject.FLAG)
        self.assertEqual(set(attempted), {10, 20, 30})


if __name__ == "__main__":
    unittest.main()
