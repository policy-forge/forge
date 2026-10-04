#!/usr/bin/env python3
"""Proposed Linux root-owned filesystem controls for the dual leaf-closure engines.

Ordinary invocation skips these physical trust cases. They create synthetic trees
only under an already qualified /root, never alter real interpreter files, invoke
sudo/ip/network namespaces, or establish runtime/denial/acceptance qualification.
Root must execute reviewed protected source copies and record nonzero discovery.
"""
import importlib.util
import os
from pathlib import Path
import stat
import sys
import tempfile
import time
import types
import unittest
from unittest import mock

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))


def load_closure(name, filename):
    """Load only the reviewed adjacent qualifier source without executing its CLI main guard."""
    specification = importlib.util.spec_from_file_location(name, HERE / filename)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


outer = load_closure("leaf_outer_controls", "verify_workspace_os_denial.py")
privileged = load_closure("leaf_native_controls", "test_workspace_os_denial.py")
ENGINES = (outer, privileged)


class PhysicalLeafClosureControls(unittest.TestCase):
    """Exercise actual no-follow/O_PATH/readlink/EOF primitives in explicitly synthetic owned trees."""

    @classmethod
    def setUpClass(cls):
        """Require actual Linux UID0 and a trusted fixed fixture parent; no provisioning or fallback occurs."""
        if sys.platform != "linux" or os.getuid() != 0 or os.geteuid() != 0:
            raise unittest.SkipTest("Linux UID0 physical controls require separate Root execution")
        outer.root_trusted(Path("/root"), True)

    def setUp(self):
        """Own one fresh synthetic tree and keep every test independent of previous fixtures/counters."""
        self.private = tempfile.TemporaryDirectory(prefix="forge-leaf-closure-", dir="/root")
        self.addCleanup(self.private.cleanup)
        self.base = Path(self.private.name)
        holder = self.base / "holder"
        holder.mkdir(mode=0o700)
        self.root = holder / "stdlib"
        self.root.mkdir(mode=0o700)
        self.dynamic = self.root / "lib-dynload"
        self.dynamic.mkdir(mode=0o700)
        (self.root / "os.py").write_bytes(b"OS\0")
        self.paths = [str(self.root), str(self.dynamic)]

    def proof(self, engine, paths=None):
        """Run one actual standalone engine against the explicit fixture, never process-global stdlib authority."""
        return engine.leaf_closure(self.paths if paths is None else paths, time.monotonic() + 30)

    def both_refuse(self, reason=None):
        """Require both complete engines to refuse the same physical negative fixture, without prefix credit."""
        for engine in ENGINES:
            with self.subTest(engine=engine.__name__), self.assertRaises(engine.LeafClosureError) as caught:
                self.proof(engine)
            if reason is not None:
                self.assertEqual(caught.exception.reason, reason)

    def test_regular_inventory_and_binary_leaf_alias_agree(self):
        """0777 symlink bits cannot reject a fully qualified binary regular member or erase its alias charges."""
        raw = b"BIN\0\xff" * 8193
        (self.root / "target.bin").write_bytes(raw)
        (self.root / "alias.py").symlink_to("target.bin")
        (self.dynamic / "extension.so").write_bytes(b"SO")
        self.assertEqual((self.root / "alias.py").lstat().st_mode & 0o022, 0o022)
        left, files = self.proof(outer)
        right, other = self.proof(privileged)
        self.assertEqual(left, right)
        self.assertEqual(files, other)
        self.assertEqual(left["entries"], 6)
        self.assertEqual(left["charged_bytes"], 2 * (3 + 2 * len(raw) + 2 + 2))
        self.assertEqual(left["link_hops"], 2)
        self.assertEqual(files[os.fsencode(self.root / "target.bin")][1]["bytes"], len(raw))

    def test_distinct_aliases_and_overlapping_roots_keep_complete_counts(self):
        """Two logical leaf aliases and a duplicated dynload member retain all rows and conservative byte reads."""
        (self.dynamic / "part.so").write_bytes(b"ABCD")
        (self.root / "first.so").symlink_to("lib-dynload/part.so")
        (self.root / "second.so").symlink_to("lib-dynload/part.so")
        for engine in ENGINES:
            summary, _ = self.proof(engine)
            self.assertEqual(summary["entries"], 6)
            self.assertEqual(summary["charged_bytes"], 2 * (3 + 4 + 4 + 4 + 4))
            self.assertEqual(summary["link_hops"], 4)

    def test_relative_parent_components_are_actually_qualified(self):
        """A safe in-union parent traversal succeeds, while a writable detour cannot be lexically erased."""
        sub = self.root / "sub"
        sub.mkdir(mode=0o700)
        (self.root / "target").write_bytes(b"T")
        (sub / "safe").symlink_to("../target")
        for engine in ENGINES:
            self.proof(engine)
        detour = self.root.parent / "detour"
        detour.mkdir(mode=0o777)
        detour.chmod(0o777)
        (self.root / "bad").symlink_to("../detour/../stdlib/target")
        self.both_refuse("worker-writable")

    def test_root_and_directory_links_are_never_admitted(self):
        """A leaf pointing at a directory and a root spelling alias remain outside the admitted subset."""
        (self.root / "directory-alias").symlink_to("lib-dynload", target_is_directory=True)
        self.both_refuse("unsupported-link")
        (self.root / "directory-alias").unlink()
        alias = self.base / "root-alias"
        alias.symlink_to(self.root, target_is_directory=True)
        for engine in ENGINES:
            with self.assertRaises((engine.LeafClosureError, OSError)):
                self.proof(engine, [str(alias), str(self.dynamic)])

    def test_external_regular_target_is_refused_without_target_byte_reads(self):
        """A root-owned regular sibling is still outside complete stdlib membership and never enters hash_leaf."""
        outside = self.base / "outside"
        outside.write_bytes(b"PRIVATE OUTSIDE")
        (self.root / "alias").symlink_to(str(outside))
        for engine in ENGINES:
            observed = []
            original = engine.LeafClosure.hash_leaf
            def record(instance, parent, name, expected):
                """Record actual hash-entry names while retaining every real no-follow read."""
                observed.append(name)
                return original(instance, parent, name, expected)
            with mock.patch.object(engine.LeafClosure, "hash_leaf", record):
                with self.assertRaises(engine.LeafClosureError) as caught:
                    self.proof(engine)
            self.assertEqual(caught.exception.reason, "unsupported-link")
            self.assertNotIn(b"outside", observed)

    def test_dangling_cycle_and_special_targets_refuse_whole_proof(self):
        """Dangling links, cycles and FIFOs cannot produce a terminal regular member or a qualified prefix."""
        alias = self.root / "alias"
        alias.symlink_to("absent")
        self.both_refuse("unsupported-link")
        alias.unlink()
        alias.symlink_to("second")
        second = self.root / "second"
        second.symlink_to("alias")
        self.both_refuse("unsupported-link")
        alias.unlink(); second.unlink()
        os.mkfifo(self.root / "pipe", 0o600)
        alias.symlink_to("pipe")
        self.both_refuse("unsupported-kind")

    def test_nonroot_link_and_regular_target_refuse(self):
        """UID0 link ownership cannot authorize a nonroot target; a nonroot link also fails first."""
        target = self.root / "target"
        target.write_bytes(b"T")
        alias = self.root / "alias"
        alias.symlink_to("target")
        os.chown(target, 1001, 1001)
        self.both_refuse("not-root-owned")
        os.chown(target, 0, 0)
        os.chown(alias, 1001, 1001, follow_symlinks=False)
        self.both_refuse("not-root-owned")

    def test_nonlink_writable_target_and_ancestor_refuse(self):
        """Target and ancestor permission predicates remain mandatory regardless of root-owned link metadata."""
        target = self.root / "target"
        target.write_bytes(b"T")
        alias = self.root / "alias"
        alias.symlink_to("target")
        target.chmod(0o666)
        self.both_refuse("worker-writable")
        target.chmod(0o600)
        self.root.chmod(0o777)
        self.both_refuse("worker-writable")
        self.root.chmod(0o700)

    def test_sixteen_hops_positive_seventeen_refuses(self):
        """The exact chain boundary retains every link, and one additional in-union leaf refuses complete proof."""
        (self.root / "target").write_bytes(b"T")
        for ordinal in reversed(range(16)):
            (self.root / ("link%02d" % ordinal)).symlink_to("target" if ordinal == 15 else "link%02d" % (ordinal + 1))
        for engine in ENGINES:
            summary, _ = self.proof(engine)
            self.assertEqual(summary["link_hops"], 2 * sum(range(1, 17)))
        (self.root / "extra").symlink_to("link00")
        self.both_refuse("unsupported-link")

    def test_retarget_after_read_refuses_even_with_same_target_bytes(self):
        """Replacing the original link after its actual held read invalidates identity even when target bytes match."""
        for engine in ENGINES:
            alias = self.root / "alias"
            if alias.is_symlink(): alias.unlink()
            (self.root / "first").write_bytes(b"SAME")
            (self.root / "second").write_bytes(b"SAME")
            alias.symlink_to("first")
            original, changed = engine.LeafClosure.read_link, [False]
            def retarget(instance, parent, name, expected):
                """Change only the explicitly synthetic alias after the real descriptor-bound observation."""
                raw = original(instance, parent, name, expected)
                if name == b"alias" and not changed[0]:
                    changed[0] = True
                    alias.unlink(); alias.symlink_to("second")
                return raw
            with mock.patch.object(engine.LeafClosure, "read_link", retarget):
                with self.assertRaises(engine.LeafClosureError): self.proof(engine)
            self.assertTrue(changed[0])

    def test_target_growth_after_pin_refuses_complete_inventory(self):
        """An actual file mutation after the first full hash cannot be hidden by a still-present alias."""
        for engine in ENGINES:
            target = self.root / "target"
            target.write_bytes(b"OLD")
            alias = self.root / "alias"
            if not alias.is_symlink(): alias.symlink_to("target")
            original, changed = engine.LeafClosure.hash_leaf, [False]
            def mutate(instance, parent, name, expected):
                """Mutate the controlled file only after its original exact-EOF proof returned."""
                pin = original(instance, parent, name, expected)
                if name == b"target" and not changed[0]:
                    changed[0] = True
                    target.write_bytes(b"NEW LONGER")
                return pin
            with mock.patch.object(engine.LeafClosure, "hash_leaf", mutate):
                with self.assertRaises(engine.LeafClosureError): self.proof(engine)
            self.assertTrue(changed[0])

    def test_new_optional_zip_root_refuses_absence_proof(self):
        """A previously absent optional root becoming present is detected by full second-root reconciliation."""
        zip_root = self.base / "python.zip"
        paths = [str(zip_root), *self.paths]
        for engine in ENGINES:
            if zip_root.exists(): zip_root.unlink()
            self.proof(engine, paths)
            original, changed = engine.LeafClosure.scan, [False]
            def materialize(instance, roots):
                """Create the synthetic formerly absent root after one actual complete inventory."""
                result = original(instance, roots)
                if not changed[0]:
                    changed[0] = True
                    zip_root.write_bytes(b"NEW")
                return result
            with mock.patch.object(engine.LeafClosure, "scan", materialize):
                with self.assertRaises(engine.LeafClosureError): self.proof(engine, paths)
            self.assertTrue(changed[0])

    def test_complete_alias_bytes_exact_limit_and_plus_one_refusal(self):
        """The shared cap includes both inventories and every alias; each standalone engine uses one allowance."""
        (self.root / "target").write_bytes(b"ABCDE")
        (self.root / "alias").symlink_to("target")
        charge = 2 * (3 + 5 + 5)
        for engine in ENGINES:
            with mock.patch.object(engine, "LEAF_BYTE_LIMIT", charge):
                summary, _ = self.proof(engine)
                self.assertEqual(summary["charged_bytes"], charge)
            with mock.patch.object(engine, "LEAF_BYTE_LIMIT", charge - 1):
                with self.assertRaises(engine.LeafClosureError) as caught: self.proof(engine)
            self.assertEqual(caught.exception.reason, "byte-bound")

    def test_logical_entry_limit_not_renewed_by_overlapping_root(self):
        """The exact two-entry fixture succeeds; a third member refuses before it can receive a complete proof."""
        for engine in ENGINES:
            with mock.patch.object(engine, "LEAF_ENTRY_LIMIT", 2): self.proof(engine)
        (self.root / "extra").write_bytes(b"X")
        for engine in ENGINES:
            with mock.patch.object(engine, "LEAF_ENTRY_LIMIT", 2):
                with self.assertRaises(engine.LeafClosureError) as caught: self.proof(engine)
            self.assertEqual(caught.exception.reason, "entry-bound")

    def test_descriptor_cleanup_fault_rejects_and_attempts_independent_closes(self):
        """A real closed handle reported as uncertain cannot qualify, and the second owned close is still attempted."""
        for engine in ENGINES:
            budget = engine.LeafBudget(time.monotonic() + 30)
            one = budget.open(os.fsencode(self.root), os.O_RDONLY | os.O_DIRECTORY)
            two = budget.open(os.fsencode(self.dynamic), os.O_RDONLY | os.O_DIRECTORY)
            original, calls = engine.os.close, []
            def uncertain(fd):
                """Close the actual synthetic handle and inject only the first post-close observation fault."""
                calls.append(fd)
                original(fd)
                if fd == one: raise OSError("private close fault")
            with mock.patch.object(engine.os, "close", uncertain):
                with self.assertRaises(engine.LeafClosureError): budget.close_all()
            self.assertIn(one, calls); self.assertIn(two, calls)

    def test_expired_budget_refuses_before_open(self):
        """The exact absolute deadline cannot open even the held slash or dispatch any child."""
        for engine in ENGINES:
            with mock.patch.object(engine.os, "open") as opened:
                with self.assertRaises(engine.LeafClosureError) as caught:
                    engine.leaf_closure(self.paths, time.monotonic() - 1)
            opened.assert_not_called()
            self.assertEqual(caught.exception.reason, "deadline-expired")

    def test_os_marker_requires_root_level_member_but_may_be_a_proved_alias(self):
        """A nested os.py cannot replace the native root requirement; a valid root-level leaf alias can."""
        (self.root / "os.py").unlink()
        nested = self.root / "nested"
        nested.mkdir(mode=0o700)
        (nested / "os.py").write_bytes(b"OS")
        self.both_refuse("entry-observation-unverified")
        (self.root / "os.py").symlink_to("nested/os.py")
        for engine in ENGINES:
            self.proof(engine)

    def fresh_root(self):
        """Rebuild only the synthetic canonical root spelling after an owned directory rename."""
        self.root.mkdir(mode=0o700)
        self.dynamic.mkdir(mode=0o700)
        (self.root / "os.py").write_bytes(b"OS\0")

    def test_root_replacement_during_each_inventory_pass_refuses(self):
        """Real root inode replacement in either full pass invalidates held/fresh root continuity."""
        for engine in ENGINES:
            for phase in (1, 2):
                original, calls, changed = engine.LeafClosure.visit, [0], [False]
                parked = self.root.parent / ("parked-root-" + engine.__name__ + str(phase))
                def replace(instance, index, raw, fd, relative, depth, rows, files, links):
                    """Rename only after the actual root entries were inspected in the selected pass."""
                    result = original(instance, index, raw, fd, relative, depth, rows, files, links)
                    if raw == os.fsencode(self.root) and relative == b"":
                        calls[0] += 1
                        if calls[0] == phase:
                            self.root.rename(parked)
                            self.fresh_root()
                            changed[0] = True
                    return result
                with mock.patch.object(engine.LeafClosure, "visit", replace):
                    with self.assertRaises(engine.LeafClosureError): self.proof(engine)
                self.assertTrue(changed[0])

    def test_superseded_ancestor_replacement_during_each_pass_refuses(self):
        """A renamed root ancestor in either pass is requalified by spelling after its older handle was closed."""
        for engine in ENGINES:
            for phase in (1, 2):
                original, calls, changed = engine.LeafClosure.visit, [0], [False]
                ancestor = self.root.parent
                parked = self.base / ("parked-ancestor-" + engine.__name__ + str(phase))
                def replace(instance, index, raw, fd, relative, depth, rows, files, links):
                    """Replace the controlled ancestor after actual root traversal, retaining old root descriptors."""
                    result = original(instance, index, raw, fd, relative, depth, rows, files, links)
                    if raw == os.fsencode(self.root) and relative == b"":
                        calls[0] += 1
                        if calls[0] == phase:
                            ancestor.rename(parked)
                            ancestor.mkdir(mode=0o700)
                            self.fresh_root()
                            changed[0] = True
                    return result
                with mock.patch.object(engine.LeafClosure, "visit", replace):
                    with self.assertRaises(engine.LeafClosureError): self.proof(engine)
                self.assertTrue(changed[0])

    def test_retained_root_generation_is_checked_after_final_reopen(self):
        """Safe mode drift after the final root rows are captured still invalidates earlier held root generations."""
        for engine in ENGINES:
            self.root.chmod(0o700)
            original, calls, changed = engine.LeafClosure.qualified_roots, [0], [False]
            def change_generation(instance):
                """Change only the already-held synthetic root after the third actual root reconciliation."""
                result = original(instance)
                calls[0] += 1
                if calls[0] == 3:
                    self.root.chmod(0o750)
                    changed[0] = True
                return result
            with mock.patch.object(engine.LeafClosure, "qualified_roots", change_generation):
                with self.assertRaises(engine.LeafClosureError): self.proof(engine)
            self.assertEqual(calls[0], 3)
            self.assertTrue(changed[0])
        self.root.chmod(0o700)


    def test_actual_stream_and_descriptor_share_exact_handle_boundary(self):
        """One actual stream plus its directory FD consumes two slots; further opens refuse before creation."""
        for engine in ENGINES:
            budget = engine.LeafBudget(time.monotonic() + 30)
            fd = budget.open(os.fsencode(self.root), os.O_RDONLY | os.O_DIRECTORY)
            with mock.patch.object(engine, "LEAF_FD_LIMIT", 2):
                with budget.scandir(fd) as entries:
                    self.assertTrue(any(entry.name == "os.py" for entry in entries))
                    self.assertEqual(len(budget.held) + len(budget.streams), 2)
                    with mock.patch.object(engine.os, "scandir") as scanned:
                        with self.assertRaises(engine.LeafClosureError): budget.scandir(fd)
                    scanned.assert_not_called()
                    with mock.patch.object(engine.os, "open") as opened:
                        with self.assertRaises(engine.LeafClosureError): budget.open(b"/never-opened", 0)
                    opened.assert_not_called()
                self.assertEqual(budget.streams, {})
                with budget.scandir(fd) as entries:
                    self.assertTrue(any(entry.name == "os.py" for entry in entries))
            budget.close_all()
            with self.assertRaises(OSError): os.fstat(fd)

    def test_actual_stream_close_fault_keeps_charge_and_attempts_all_owned_cleanup(self):
        """A real closed iterator with an uncertain close report remains charged while independent owners close."""
        for engine in ENGINES:
            budget = engine.LeafBudget(time.monotonic() + 30)
            fd = budget.open(os.fsencode(self.root), os.O_RDONLY | os.O_DIRECTORY)
            original, actual, calls = engine.os.scandir, [], []
            def uncertain_stream(directory):
                """Create a real iterator after reservation and inject only its first owner's post-close fault."""
                iterator = original(directory)
                ordinal = len(actual)
                actual.append(iterator)
                def close():
                    """Close the actual iterator before reporting uncertainty for the first retained stream."""
                    calls.append(ordinal)
                    iterator.close()
                    if ordinal == 0: raise OSError("PRIVATE post-close fault")
                return types.SimpleNamespace(close=close)
            with mock.patch.object(engine.os, "scandir", uncertain_stream):
                first = budget.scandir(fd)
                second = budget.scandir(fd)
                with self.assertRaises(engine.LeafClosureError):
                    with first: raise ValueError("PRIVATE traversal fault")
                self.assertEqual(len(budget.streams), 2)
                with self.assertRaises(engine.LeafClosureError): budget.close_all()
            self.assertIn(0, calls); self.assertIn(1, calls)
            self.assertEqual(len(budget.streams), 1)
            self.assertEqual(budget.held, set())
            with self.assertRaises(OSError): os.fstat(fd)
            # The native iterator really closed; its reservation intentionally
            # remains uncertain after refusal rather than gaining success credit.


if __name__ == "__main__":
    unittest.main()
