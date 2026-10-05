#!/usr/bin/env python3
"""Proposed fixed-site/secondary-publication controls with synthetic IO; no runner or native qualification."""
import copy
import importlib.util
import json
import os
from pathlib import Path
import stat
import sys
import tempfile
import types
import unittest
from unittest import mock

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))


def load_wrapper():
    """Load the maintained wrapper in controls only without invoking main or any tool/process."""
    spec = importlib.util.spec_from_file_location("stdlib_link_detail_controls", HERE / "verify_workspace_os_denial.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


wrapper = load_wrapper()


class LinkDiagnosticControls(unittest.TestCase):
    """Exercise actual raise sites and bounded publication with explicitly synthetic metadata and owned test files."""

    def info(self, size=1, uid=0, mode=None):
        """Supply complete independent synthetic symlink metadata; this is not a held runner object."""
        return types.SimpleNamespace(st_dev=1, st_ino=2, st_mode=stat.S_IFLNK | 0o777 if mode is None else mode,
                                     st_uid=uid, st_gid=0, st_size=size, st_mtime_ns=3, st_ctime_ns=4)

    def capture_error(self, callback, *args):
        """Retain a genuine raised exception traceback in the except block instead of a fabricated traceback."""
        try:
            callback(*args)
        except wrapper.LeafClosureError as error:
            return wrapper.link_rejection_site(error), error.reason, error.phase
        self.fail("expected qualification refusal")

    def fixture(self, directory):
        """Copy actual control sources and write a synthetic primary receipt for publication-body controls only."""
        root = directory / "project"
        destination = directory / "output"
        destination.mkdir()
        inputs = {}
        for relative in wrapper.LINK_DETAIL_INPUTS:
            original = HERE.parent / relative
            target = root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(original.read_bytes())
            inputs[relative] = wrapper.pin_bytes(target.read_bytes())
        receipt = {"status": "incomplete", "failure": "tool-untrusted", "input_stability": "unchanged",
                   "diagnostic": {"phase": "stdlib-entry", "reason": "unsupported-link", "exit_code": None},
                   "producer": {"status": "not-run", "exit_code": None, "failure": None, "receipt": None, "receipt_pin": None},
                   "cleanup": {"state": "verified-not-created", "forced": False}, "identity": {"inputs": inputs},
                   "checkout": {"requested_head": "a" * 40, "tested_commit": "b" * 40}}
        wrapper.shared.atomic_receipt(destination / wrapper.OUTPUT, receipt)
        return root, destination, receipt

    def test_required_kind_link_preserves_uid_priority(self):
        """A real trusted-method refusal identifies only its fixed kind site; owner failure still wins first."""
        instance = wrapper.LeafClosure.__new__(wrapper.LeafClosure)
        self.assertEqual(self.capture_error(instance.trusted, self.info(), "directory"),
                         ("required-kind-symlink", "unsupported-link", "stdlib-entry"))
        self.assertEqual(self.capture_error(instance.trusted, self.info(uid=1001), "directory"),
                         (None, "not-root-owned", "stdlib-entry"))
        instance.trusted(self.info(), "link")

    def test_inventoried_link_size_refuses_before_io(self):
        """Both out-of-range original sizes execute the real size refusal before open/readlink."""
        instance = wrapper.LeafClosure.__new__(wrapper.LeafClosure)
        for size in (0, wrapper.LEAF_PATH_LIMIT + 1):
            with self.subTest(size=size), mock.patch.object(wrapper.os, "open") as opened, \
                 mock.patch.object(wrapper.os, "readlink") as read:
                self.assertEqual(self.capture_error(instance.read_link, 9, b"PRIVATE", wrapper.leaf_identity(self.info(size))),
                                 ("inventoried-link-size", "unsupported-link", "stdlib-entry"))
                opened.assert_not_called()
                read.assert_not_called()

    def test_held_size_and_text_sites_close_actual_budget_owner(self):
        """Synthetic held-size/text failures retain real budget-close behavior and exact fixed categories."""
        for observed_size, text, expected in ((0, b"x", "held-link-size"),
                                            (1, b"", "link-text-shape"), (2, b"//", "link-text-shape"),
                                            (2, b"x/", "link-text-shape"), (1, b"\0", "link-text-shape"),
                                            (1, "x", "link-text-shape"), (1, b"\xff", "link-text-utf8")):
            instance = wrapper.LeafClosure.__new__(wrapper.LeafClosure)
            original = wrapper.leaf_identity(self.info(observed_size if observed_size else 1))
            with self.subTest(expected=expected, text=repr(text)), \
                 mock.patch.object(wrapper.time, "monotonic", return_value=0), \
                 mock.patch.object(wrapper.os, "O_PATH", 0x200000, create=True), \
                 mock.patch.object(wrapper.os, "open", return_value=10), \
                 mock.patch.object(wrapper.os, "fstat", return_value=self.info(observed_size)), \
                 mock.patch.object(wrapper.os, "readlink", return_value=text) as read, \
                 mock.patch.object(wrapper.os, "close") as closed:
                instance.budget = wrapper.LeafBudget(100)
                self.assertEqual(self.capture_error(instance.read_link, 9, b"PRIVATE", original),
                                 (expected, "unsupported-link", "stdlib-entry"))
                self.assertFalse(instance.budget.held)
                closed.assert_called_once_with(10)
                if observed_size == 0:
                    read.assert_not_called()

    def test_origin_and_hop_sites_are_real_resolve_refusals(self):
        """The original-inventory and zero-hop refusals use real resolve code, without target inspection."""
        instance = wrapper.LeafClosure.__new__(wrapper.LeafClosure)
        self.assertEqual(self.capture_error(instance.resolve, b"PRIVATE", {}, {}),
                         ("origin-not-in-link-inventory", "unsupported-link", "stdlib-entry"))
        with mock.patch.object(wrapper, "LEAF_HOP_LIMIT", 0):
            self.assertEqual(self.capture_error(instance.resolve, b"PRIVATE", {}, {}),
                             ("link-hop-limit", "unsupported-link", "stdlib-entry"))

    def test_every_site_offset_is_the_frozen_raise_statement(self):
        """Validate all thirteen lexical site anchors; this does not execute the remaining path-walk branches."""
        lines = (HERE / "verify_workspace_os_denial.py").read_text().splitlines()
        count = 0
        for name, offsets in wrapper.LINK_DETAIL_SITES.items():
            first = getattr(wrapper.LeafClosure, name).__code__.co_firstlineno
            for offset in offsets:
                self.assertIn('raise LeafClosureError("unsupported-link"', lines[first + offset - 1])
                count += 1
        self.assertEqual(count, 13)

    def test_caller_traceback_unknown_phase_and_unraised_errors_abstain(self):
        """Neither a caller-created traceback nor a different phase/error type becomes an engine-site observation."""
        def foreign_raise():
            """Raise the same fixed reason outside the maintained engine to test code-identity refusal."""
            raise wrapper.LeafClosureError("unsupported-link")
        self.assertEqual(self.capture_error(foreign_raise), (None, "unsupported-link", "stdlib-entry"))
        self.assertIsNone(wrapper.link_rejection_site(wrapper.LeafClosureError("unsupported-link")))
        self.assertIsNone(wrapper.link_rejection_site(wrapper.LeafClosureError("unsupported-link", "stdlib-roots")))
        self.assertIsNone(wrapper.link_rejection_site(ValueError("PRIVATE")))

    def test_actual_inventory_keeps_primary_error_and_private_site(self):
        """Actual stdlib_inventory attaches a fixed site while preserving incomplete/tool-untrusted and closed /2 fields."""
        instance = wrapper.LeafClosure.__new__(wrapper.LeafClosure)
        def refuse(_paths, _deadline):
            """Execute a real size rejection under the actual inventory adapter without native IO."""
            instance.read_link(9, b"PRIVATE", wrapper.leaf_identity(self.info(0)))
        with mock.patch.object(wrapper, "leaf_closure", side_effect=refuse):
            try:
                wrapper.stdlib_inventory(["/mock/a", "/mock/lib-dynload"], 100)
            except wrapper.GateError as error:
                self.assertEqual((error.code, error.incomplete, error.link_subcondition),
                                 ("tool-untrusted", True, "inventoried-link-size"))
                self.assertEqual(error.diagnostic, {"phase": "stdlib-entry", "reason": "unsupported-link", "exit_code": None})
            else:
                self.fail("expected unchanged primary failure")

    def test_closed_record_pins_real_engine_outer_file_and_no_private_payload(self):
        """Real owned-file reads bind the minimal fixed record; synthetic receipt facts confer no native authority."""
        with tempfile.TemporaryDirectory() as name:
            root, destination, receipt = self.fixture(Path(name))
            record = wrapper.link_diagnostic_record(root, destination, receipt, "target-not-in-inventory")
            self.assertEqual(set(record), {"schema_version", "scope", "truth_state", "acceptance_eligible", "status",
                             "phase", "reason", "subcondition", "requested_commit", "tested_commit", "source_inputs",
                             "engine_pin", "outer_receipt_pin", "membership"})
            self.assertFalse(record["acceptance_eligible"])
            self.assertEqual(record["outer_receipt_pin"], wrapper.pin_bytes((destination / wrapper.OUTPUT).read_bytes()))
            raw = wrapper.shared.canonical_bytes(record)
            self.assertLessEqual(len(raw), 2048)
            self.assertNotIn(str(root).encode(), raw)
            self.assertNotIn(b"PRIVATE", raw)

    def test_failed_or_changed_or_dispatched_receipt_cannot_publish_detail(self):
        """A later source/primary override, producer dispatch or unknown category never gains an observed sidecar."""
        with tempfile.TemporaryDirectory() as name:
            root, destination, original = self.fixture(Path(name))
            for field, value in (("status", "failed"), ("failure", "source-changed"), ("input_stability", "changed"),
                                 ("producer", dict(original["producer"], status="failed")),
                                 ("diagnostic", {"phase": "stdlib-entry", "reason": "worker-writable", "exit_code": None})):
                changed = copy.deepcopy(original)
                changed[field] = value
                self.assertIsNone(wrapper.link_diagnostic_record(root, destination, changed, "link-text-shape"))
            self.assertIsNone(wrapper.link_diagnostic_record(root, destination, original, "PRIVATE"))

    def test_engine_source_drift_and_outer_file_drift_abstain(self):
        """Even a repinned fixture cannot assign a site to a changed engine or detached primary-file value."""
        with tempfile.TemporaryDirectory() as name:
            root, destination, receipt = self.fixture(Path(name))
            source = root / "scripts/verify_workspace_os_denial.py"
            source.write_bytes(source.read_bytes().replace(b'LEAF_ENTRY_LIMIT = 20000', b'LEAF_ENTRY_LIMIT = 20001'))
            receipt["identity"]["inputs"]["scripts/verify_workspace_os_denial.py"] = wrapper.pin_bytes(source.read_bytes())
            self.assertIsNone(wrapper.link_diagnostic_record(root, destination, receipt, "link-text-shape"))
        with tempfile.TemporaryDirectory() as name:
            root, destination, receipt = self.fixture(Path(name))
            (destination / wrapper.OUTPUT).write_bytes(b"{}\n")
            self.assertIsNone(wrapper.link_diagnostic_record(root, destination, receipt, "link-text-shape"))

    def test_fresh_publication_and_collision_never_replace_or_reflag(self):
        """The actual maintained no-replace publisher invokes the callback only after a new bounded sidecar exists."""
        with tempfile.TemporaryDirectory() as name, mock.patch.object(wrapper.time, "monotonic", return_value=0):
            root, destination, receipt = self.fixture(Path(name))
            callback = mock.Mock()
            self.assertTrue(wrapper.publish_link_diagnostic(root, destination, receipt, "link-text-shape", 100, callback))
            first = (destination / wrapper.LINK_DETAIL_OUTPUT).read_bytes()
            callback.assert_called_once_with()
            callback.reset_mock()
            self.assertFalse(wrapper.publish_link_diagnostic(root, destination, receipt, "link-text-shape", 100, callback))
            self.assertEqual((destination / wrapper.LINK_DETAIL_OUTPUT).read_bytes(), first)
            callback.assert_not_called()

    def test_secondary_publisher_callback_and_expiry_faults_preserve_primary(self):
        """Faults cannot mutate the original failure; expiry starts no publication, and postwrite callbacks are secondary."""
        with tempfile.TemporaryDirectory() as name:
            root, destination, receipt = self.fixture(Path(name))
            original = copy.deepcopy(receipt)
            callback = mock.Mock(side_effect=OSError("PRIVATE"))
            with mock.patch.object(wrapper.time, "monotonic", return_value=100), mock.patch.object(wrapper.shared, "atomic_receipt") as publish:
                self.assertFalse(wrapper.publish_link_diagnostic(root, destination, receipt, "link-text-shape", 100, callback))
                publish.assert_not_called()
                callback.assert_not_called()
            with mock.patch.object(wrapper.time, "monotonic", return_value=0), \
                 mock.patch.object(wrapper.shared, "atomic_receipt", side_effect=OSError("PRIVATE")):
                self.assertFalse(wrapper.publish_link_diagnostic(root, destination, receipt, "link-text-shape", 100, callback))
                callback.assert_not_called()
            with mock.patch.object(wrapper.time, "monotonic", return_value=0):
                self.assertFalse(wrapper.publish_link_diagnostic(root, destination, receipt, "link-text-shape", 100, callback))
            callback.assert_called_once_with()
            self.assertEqual(receipt, original)
            self.assertEqual((destination / wrapper.OUTPUT).read_bytes(), wrapper.shared.canonical_bytes(original))

    def test_flag_uses_only_fixed_bytes_and_missing_environment_abstains(self):
        """Real append output carries no paths/reasons; a missing runner channel creates no alternate output."""
        with mock.patch.dict(wrapper.os.environ, {}, clear=True), mock.patch.object(wrapper.os, "open") as opened:
            self.assertFalse(wrapper.emit_link_diagnostic_publication_flag())
            opened.assert_not_called()
        with tempfile.TemporaryDirectory() as name:
            path = Path(name) / "output"
            path.write_bytes(b"prior=value\n")
            with mock.patch.dict(wrapper.os.environ, {"GITHUB_OUTPUT": str(path)}):
                self.assertTrue(wrapper.emit_link_diagnostic_publication_flag())
            self.assertEqual(path.read_bytes(), b"prior=value\nstdlib_link_detail_published=true\n")


    def test_actual_verify_preserves_failure_and_emits_only_after_stability(self):
        """Drive the actual wrapper catch/recheck/publish seam with genuine size refusal and synthetic tool/checkout adapters."""
        with tempfile.TemporaryDirectory() as name:
            root, _old_output, seed = self.fixture(Path(name))
            identity = dict(seed["identity"], tracked_source_clean=True)
            instance = wrapper.LeafClosure.__new__(wrapper.LeafClosure)
            def refuse(_paths, _deadline):
                """Raise from the actual maintained engine instead of constructing detached diagnostic facts."""
                instance.read_link(9, b"PRIVATE", wrapper.leaf_identity(self.info(0)))
            def tools(_root, deadline):
                """Consume the real inventory adapter while replacing all executable/tool observations."""
                return wrapper.stdlib_inventory(["/mock/a", "/mock/lib-dynload"], deadline)
            with mock.patch.object(wrapper.time, "monotonic", return_value=0), \
                 mock.patch.object(wrapper.sys, "platform", "linux"), \
                 mock.patch.object(wrapper, "capture_identity", side_effect=[identity, copy.deepcopy(identity)]), \
                 mock.patch.object(wrapper.shared, "checkout_binding", return_value=seed["checkout"]), \
                 mock.patch.object(wrapper, "leaf_closure", side_effect=refuse), \
                 mock.patch.object(wrapper, "capture_tools", side_effect=tools), \
                 mock.patch.object(wrapper, "native_run") as native_run:
                callback = mock.Mock()
                output = Path(name) / "actual-output"
                receipt = wrapper.verify(root, root / "forge", output, build_outcome="success",
                                         link_diagnostic_published=callback)
            self.assertEqual((receipt["status"], receipt["failure"], receipt["input_stability"]),
                             ("incomplete", "tool-untrusted", "unchanged"))
            self.assertEqual(receipt["diagnostic"], seed["diagnostic"])
            self.assertEqual(receipt["producer"], seed["producer"])
            self.assertEqual((output / wrapper.OUTPUT).read_bytes(), wrapper.shared.canonical_bytes(receipt))
            record = json.loads((output / wrapper.LINK_DETAIL_OUTPUT).read_bytes())
            self.assertEqual(record["subcondition"], "inventoried-link-size")
            self.assertFalse(record["acceptance_eligible"])
            callback.assert_called_once_with()
            native_run.assert_not_called()

    def membership_engines(self):
        """Load both maintained literal engines in controls only; no producer, tools or qualification entrypoint runs."""
        raw = (HERE / "test_workspace_os_denial.py").read_bytes()
        first = raw.index(b"\nLEAF_PROOF_FORMAT =") + 1
        last = raw.index(b"\n\ndef trusted_stdlib", first)
        namespace = {name: getattr(wrapper, name) for name in ("os", "stat", "time", "json", "hashlib", "sys", "re")}
        exec(compile(raw[first:last], "synthetic-native-leaf-engine", "exec"), namespace)
        return (wrapper, types.SimpleNamespace(**namespace))

    def membership_instance(self, engine):
        """Construct a synthetic existing-inventory owner without asserting a native root or detached success proof."""
        instance = engine.LeafClosure.__new__(engine.LeafClosure)
        instance.budget = engine.LeafBudget(100)
        generation = engine.leaf_identity(self.info(mode=stat.S_IFDIR | 0o755))
        instance.roots = [(b"/trusted/stdlib", 21, generation, []),
                          (b"/trusted/stdlib/lib-dynload", 22, generation, [])]
        instance.paths = [b"/trusted/stdlib", b"/trusted/stdlib/lib-dynload", b"/trusted/optional.zip"]
        instance.slash = 9
        instance.scan_entries = 0
        return instance

    def membership_rows(self, engine):
        """Supply the existing complete scan-row shape with one known descendant directory and one regular member."""
        directory = engine.leaf_identity(self.info(mode=stat.S_IFDIR | 0o755))
        regular = engine.leaf_identity(self.info(mode=stat.S_IFREG | 0o644))
        return [[0, b"pkg".hex(), "directory", directory],
                [0, b"os.py".hex(), "file", regular, {"bytes": 1, "sha256": "a" * 64}]]

    def membership_refusal(self, engine, target, rows, *, no_rows=False, final_drift=False, detail_fault=False):
        """Run real lexical resolve/refusal/finally code with explicit synthetic origin IO and forbidden target IO."""
        instance = self.membership_instance(engine)
        origin = b"/trusted/stdlib/link"
        identity = engine.leaf_identity(self.info(size=len(target)))
        ancestry = [[b"/".hex(), engine.leaf_identity(self.info(mode=stat.S_IFDIR | 0o755))]]
        instance.absolute_directory = mock.Mock(return_value=(11, ancestry))
        instance.verify = mock.Mock(side_effect=[None, engine.LeafClosureError("entry-observation-unverified")]
                                    if final_drift else None)
        instance.read_link = mock.Mock(return_value=target)
        instance.directory = mock.Mock(side_effect=AssertionError("target directory IO forbidden"))
        instance.hash_leaf = mock.Mock(side_effect=AssertionError("target hashing forbidden"))
        if detail_fault:
            instance.membership_detail = mock.Mock(return_value=None)
        with mock.patch.object(engine.time, "monotonic", return_value=0), \
             mock.patch.object(engine.os, "O_PATH", 0x200000, create=True), \
             mock.patch.object(engine.os, "open", return_value=100) as opened, \
             mock.patch.object(engine.os, "fstat", side_effect=lambda fd: self.info(size=len(target))
                               if fd == 100 else self.info(mode=stat.S_IFDIR | 0o755)), \
             mock.patch.object(engine.os, "stat", side_effect=AssertionError("target stat forbidden")) as stated, \
             mock.patch.object(engine.os, "readlink", side_effect=AssertionError("target readlink forbidden")) as read, \
             mock.patch.object(engine.os, "close") as closed:
            try:
                if no_rows:
                    instance.resolve(origin, {}, {origin: (identity, target)})
                else:
                    instance.resolve(origin, {}, {origin: (identity, target)}, rows)
            except engine.LeafClosureError as error:
                failure = error
            else:
                self.fail("expected unchanged leaf-target refusal")
            opened.assert_called_once_with(b"link", engine.os.O_PATH | engine.os.O_NOFOLLOW | engine.os.O_CLOEXEC,
                                          dir_fd=11)
            stated.assert_not_called()
            read.assert_not_called()
            instance.directory.assert_not_called()
            instance.hash_leaf.assert_not_called()
            closed.assert_called_once_with(100)
        self.assertFalse(instance.budget.held)
        self.assertFalse(instance.budget.streams)
        return failure

    def test_membership_complete_rows_directory_root_and_component_boundaries(self):
        """Both literal engines distinguish root equality, inventoried descendants and similarly prefixed siblings without IO."""
        vectors = [([b"trusted", b"stdlib"], b"/trusted/stdlib", "present-root-exact", "inventoried-directory"),
                   ([b"trusted", b"stdlib", b"pkg"], b"pkg", "beneath-present-root", "inventoried-directory"),
                   ([b"trusted", b"stdlib", b"missing"], b"missing", "beneath-present-root", "no-file-link-or-directory-row"),
                   ([b"trusted", b"stdlib-extra", b"pkg"], b"/trusted/stdlib-extra/pkg", "outside-present-roots", "no-file-link-or-directory-row"),
                   ([b"trusted", b"optional.zip"], b"/trusted/optional.zip", "outside-present-roots", "no-file-link-or-directory-row")]
        for engine in self.membership_engines():
            for candidate, target, scope, kind in vectors:
                with self.subTest(candidate=candidate, scope=scope), mock.patch.object(engine.time, "monotonic", return_value=0), \
                     mock.patch.object(engine.os, "stat") as stated, mock.patch.object(engine.os, "open") as opened, \
                     mock.patch.object(engine.os, "readlink") as read, mock.patch.object(engine.os, "scandir") as scanned:
                    instance = self.membership_instance(engine)
                    before = (instance.budget.work, instance.budget.state)
                    expected = ("absolute" if target.startswith(b"/") else "relative", scope, kind)
                    self.assertEqual(instance.membership_detail(candidate, target, self.membership_rows(engine)), expected)
                    self.assertGreater(instance.budget.work, before[0])
                    self.assertGreater(instance.budget.state, before[1])
                    self.assertLessEqual(instance.budget.state, engine.LEAF_STATE_LIMIT)
                    carrier = wrapper.link_membership_record("target-not-in-inventory", expected)
                    self.assertLessEqual(len(wrapper.shared.canonical_bytes(carrier)), 512)
                    stated.assert_not_called()
                    opened.assert_not_called()
                    read.assert_not_called()
                    scanned.assert_not_called()

    def test_membership_real_resolve_preserves_absolute_relative_and_dot_reduction(self):
        """Real lexical rejection yields coarse facts after existing dot reduction, never directory-target admission."""
        for engine in self.membership_engines():
            for target, expected in [(b"pkg", ("relative", "beneath-present-root", "inventoried-directory")),
                                     (b"./pkg/../pkg", ("relative", "beneath-present-root", "inventoried-directory")),
                                     (b"/trusted/stdlib/pkg", ("absolute", "beneath-present-root", "inventoried-directory")),
                                     (b"/trusted/stdlib", ("absolute", "present-root-exact", "inventoried-directory")),
                                     (b"/trusted/stdlib-extra/pkg", ("absolute", "outside-present-roots", "no-file-link-or-directory-row"))]:
                with self.subTest(target=target):
                    error = self.membership_refusal(engine, target, self.membership_rows(engine))
                    self.assertEqual((error.reason, error.phase), ("unsupported-link", "stdlib-entry"))
                    self.assertEqual(error.membership, expected)
                    if engine is wrapper:
                        self.assertEqual(wrapper.link_rejection_site(error), "target-not-in-inventory")
                    self.assertNotIn(target.decode(), str(error))

    def test_membership_inventory_absence_is_not_target_filesystem_absence(self):
        """Owned existing and missing external targets yield identical outside-inventory facts without target observation."""
        with tempfile.TemporaryDirectory() as name:
            existing = Path(name) / "existing.py"
            existing.write_bytes(b"synthetic private target; never read by membership\n")
            missing = Path(name) / "missing.py"
            for engine in self.membership_engines():
                for path in (existing, missing):
                    error = self.membership_refusal(engine, str(path).encode(), self.membership_rows(engine))
                    self.assertEqual(error.membership, ("absolute", "outside-present-roots", "no-file-link-or-directory-row"))
                    self.assertEqual(error.reason, "unsupported-link")

    def test_membership_optional_direct_resolve_rows_abstain_without_new_io(self):
        """The old three-argument resolve call preserves the primary failure and carries no invented complete inventory facts."""
        for engine in self.membership_engines():
            error = self.membership_refusal(engine, b"pkg", None, no_rows=True)
            self.assertEqual((error.reason, error.phase, error.membership), ("unsupported-link", "stdlib-entry", None))

    def test_membership_shared_work_state_and_deadline_limits_abstain(self):
        """The original monotonic ledger and deadline suppress only diagnostic facts without renewal or cap increase."""
        for engine in self.membership_engines():
            for exhausted in ("work", "state", "deadline"):
                with self.subTest(exhausted=exhausted), mock.patch.object(engine.time, "monotonic", return_value=0):
                    instance = self.membership_instance(engine)
                    if exhausted == "deadline":
                        instance.budget.deadline = 0
                    else:
                        setattr(instance.budget, exhausted, getattr(engine, "LEAF_" + exhausted.upper() + "_LIMIT"))
                    old_deadline = instance.budget.deadline
                    self.assertIsNone(instance.membership_detail([b"trusted", b"stdlib", b"pkg"], b"pkg", self.membership_rows(engine)))
                    self.assertEqual(instance.budget.deadline, old_deadline)
                    self.assertLessEqual(instance.budget.work, engine.LEAF_WORK_LIMIT)
                    self.assertLessEqual(instance.budget.state, engine.LEAF_STATE_LIMIT)
                    self.assertFalse(instance.budget.held)
                    self.assertFalse(instance.budget.streams)

    def test_membership_diagnostic_allocation_fault_abstains(self):
        """A synthetic diagnostic accounting/allocation fault cannot turn inventory-relative metadata into a new primary failure."""
        for engine in self.membership_engines():
            with mock.patch.object(engine.time, "monotonic", return_value=0):
                instance = self.membership_instance(engine)
                with mock.patch.object(instance.budget, "charge", side_effect=MemoryError("PRIVATE")):
                    self.assertIsNone(instance.membership_detail([b"trusted", b"stdlib", b"pkg"], b"pkg", self.membership_rows(engine)))
                error = self.membership_refusal(engine, b"pkg", self.membership_rows(engine), detail_fault=True)
                self.assertEqual((error.reason, error.membership), ("unsupported-link", None))

    def test_membership_origin_final_generation_failure_overrides_observed_detail(self):
        """An actual resolve finally-fence failure keeps observation failure priority and cannot carry a usable membership profile."""
        for engine in self.membership_engines():
            error = self.membership_refusal(engine, b"pkg", self.membership_rows(engine), final_drift=True)
            self.assertEqual((error.reason, error.phase, error.membership), ("entry-observation-unverified", "stdlib-entry", None))
            if engine is wrapper:
                self.assertIsNone(wrapper.link_rejection_site(error))

    def test_membership_successful_scan_keeps_historical_exact_private_proof_bytes(self):
        """The real complete scan serializer preserves a fixed historical regular-member proof and allocates no rejection carrier."""
        expected = (b'["forge.stdlib-leaf-closure/1",[],[[0,"6f732e7079","file",[1,2,33188,0,0,1,3,4],'
                    b'{"bytes":1,"sha256":"' + b'a' * 64 + b'"}],[1,"6e61746976652e736f","file",'
                    b'[1,2,33188,0,0,1,3,4],{"bytes":1,"sha256":"' + b'a' * 64 + b'"}]]]\n')
        for engine in self.membership_engines():
            instance = self.membership_instance(engine)
            instance.membership_detail = mock.Mock(side_effect=AssertionError("success diagnostic allocation forbidden"))
            regular = engine.leaf_identity(self.info(mode=stat.S_IFREG | 0o644))
            digest = {"bytes": 1, "sha256": "a" * 64}
            def visit(index, raw, _fd, _relative, _depth, rows, files, _links):
                """Supply complete synthetic existing scan facts; retain actual row charging and serialization logic."""
                name = b"os.py" if index == 0 else b"native.so"
                instance.budget.charge("entries", 1, engine.LEAF_ENTRY_LIMIT * 2, "entry-bound")
                instance.row(rows, [index, name.hex(), "file", regular, digest])
                files[raw + b"/" + name] = (regular, digest)
            instance.visit = visit
            with mock.patch.object(engine.time, "monotonic", return_value=0), \
                 mock.patch.object(engine.os, "fstat", return_value=self.info(mode=stat.S_IFDIR | 0o755)):
                raw, count, files = instance.scan([])
            self.assertEqual(raw, expected)
            self.assertEqual(count, 2)
            self.assertEqual(len(files), 2)
            instance.membership_detail.assert_not_called()

    def test_membership_carrier_closed_tuple_and_subcondition_coupling(self):
        """Only the exact three closed private facts at the actual target-not-in-inventory category become observed membership."""
        observed = wrapper.link_membership_record("target-not-in-inventory", ("relative", "beneath-present-root", "inventoried-directory"))
        self.assertEqual(observed, {"availability": "observed-inventory-facts", "target_form": "relative",
                         "lexical_scope": "beneath-present-root", "inventory_kind": "inventoried-directory",
                         "basis": "existing-complete-inventory-only", "target_stat": "not-observed"})
        unavailable = wrapper.link_membership_record("link-text-shape", None)
        self.assertEqual(unavailable, dict(observed, availability="unavailable", target_form=None,
                                         lexical_scope=None, inventory_kind=None))
        bad = [(), ("relative", "beneath-present-root"), ("relative", "beneath-present-root", "inventoried-directory", "PRIVATE"),
               ["relative", "beneath-present-root", "inventoried-directory"],
               ("PRIVATE", "beneath-present-root", "inventoried-directory"),
               ("relative", "PRIVATE", "inventoried-directory"),
               ("relative", "beneath-present-root", "PRIVATE"),
               ("absolute", "outside-present-roots", "inventoried-directory"),
               ("absolute", "present-root-exact", "no-file-link-or-directory-row")]
        for carrier in bad:
            with self.subTest(carrier=carrier), self.assertRaises(ValueError):
                wrapper.link_membership_record("target-not-in-inventory", carrier)
        with self.assertRaises(ValueError):
            wrapper.link_membership_record("link-text-shape", ("relative", "beneath-present-root", "inventoried-directory"))

    def test_membership_current_record_closed_six_fields_and_original_primary_binding(self):
        """Real owned-file publication inputs produce one bounded /2 record without origin, target or private receipt mutation."""
        with tempfile.TemporaryDirectory() as name:
            root, destination, receipt = self.fixture(Path(name))
            original = copy.deepcopy(receipt)
            record = wrapper.link_diagnostic_record(root, destination, receipt, "target-not-in-inventory",
                                                     membership=("relative", "beneath-present-root", "inventoried-directory"))
            self.assertTrue(wrapper.validate_link_diagnostic_record(record))
            self.assertEqual(record["schema_version"], "forge.stdlib-unsupported-link-diagnostic/2")
            self.assertEqual(set(record["membership"]), {"availability", "target_form", "lexical_scope", "inventory_kind", "basis", "target_stat"})
            self.assertEqual(record["outer_receipt_pin"], wrapper.pin_bytes((destination / wrapper.OUTPUT).read_bytes()))
            raw = wrapper.shared.canonical_bytes(record)
            self.assertLessEqual(len(raw), 2048)
            self.assertNotIn(str(root).encode(), raw)
            self.assertNotIn(b"PRIVATE", raw)
            self.assertEqual(receipt, original)
            self.assertFalse(record["acceptance_eligible"])

    def test_membership_current_validator_rejects_forged_correlations_and_old_shape(self):
        """The actual closed /2 validator rejects historical /1, unknowns, missing facts, false target stats and category contradictions."""
        with tempfile.TemporaryDirectory() as name:
            root, destination, receipt = self.fixture(Path(name))
            good = wrapper.link_diagnostic_record(root, destination, receipt, "target-not-in-inventory",
                                                   membership=("relative", "beneath-present-root", "inventoried-directory"))
            historical = copy.deepcopy(good)
            historical["schema_version"] = "forge.stdlib-unsupported-link-diagnostic/1"
            historical.pop("membership")
            self.assertFalse(wrapper.validate_link_diagnostic_record(historical))
            mutations = [("availability", "unavailable"), ("target_form", None), ("target_form", "PRIVATE"),
                         ("lexical_scope", "outside-present-roots"), ("inventory_kind", None),
                         ("basis", "filesystem-observation"), ("target_stat", "observed"), ("PRIVATE", "PRIVATE")]
            for field, value in mutations:
                with self.subTest(field=field, value=value):
                    changed = copy.deepcopy(good)
                    changed["membership"][field] = value
                    self.assertFalse(wrapper.validate_link_diagnostic_record(changed))
            changed = copy.deepcopy(good)
            changed["membership"].update(lexical_scope="present-root-exact", inventory_kind="no-file-link-or-directory-row")
            self.assertFalse(wrapper.validate_link_diagnostic_record(changed))
            changed = copy.deepcopy(good)
            changed["subcondition"] = "link-text-shape"
            self.assertFalse(wrapper.validate_link_diagnostic_record(changed))
            changed = copy.deepcopy(good)
            changed.pop("membership")
            self.assertFalse(wrapper.validate_link_diagnostic_record(changed))
            changed = copy.deepcopy(good)
            changed["PRIVATE"] = "PRIVATE"
            self.assertFalse(wrapper.validate_link_diagnostic_record(changed))

    def test_membership_unavailable_null_facts_still_requires_current_shape(self):
        """No private carrier means all three facts are null while fixed inventory-only and not-observed qualifiers remain mandatory."""
        with tempfile.TemporaryDirectory() as name:
            root, destination, receipt = self.fixture(Path(name))
            record = wrapper.link_diagnostic_record(root, destination, receipt, "target-not-in-inventory")
            self.assertTrue(wrapper.validate_link_diagnostic_record(record))
            self.assertEqual(record["membership"], {"availability": "unavailable", "target_form": None,
                             "lexical_scope": None, "inventory_kind": None,
                             "basis": "existing-complete-inventory-only", "target_stat": "not-observed"})
            for field, value in [("target_form", "relative"), ("lexical_scope", "beneath-present-root"),
                                 ("inventory_kind", "no-file-link-or-directory-row")]:
                changed = copy.deepcopy(record)
                changed["membership"][field] = value
                self.assertFalse(wrapper.validate_link_diagnostic_record(changed))

    def test_membership_fresh_v2_publication_collision_and_invalid_carrier_never_reflag(self):
        """Only actual NEW bounded /2 publication grants the existing fixed callback; a sentinel or forged carrier cannot replace it."""
        with tempfile.TemporaryDirectory() as name, mock.patch.object(wrapper.time, "monotonic", return_value=0):
            root, destination, receipt = self.fixture(Path(name))
            callback = mock.Mock()
            carrier = ("relative", "beneath-present-root", "inventoried-directory")
            self.assertEqual(wrapper.LINK_DETAIL_OUTPUT, "stdlib-unsupported-link-diagnostic-v2.json")
            self.assertTrue(wrapper.publish_link_diagnostic(root, destination, receipt, "target-not-in-inventory", 100, callback,
                                                            membership=carrier))
            first = (destination / wrapper.LINK_DETAIL_OUTPUT).read_bytes()
            callback.assert_called_once_with()
            callback.reset_mock()
            self.assertFalse(wrapper.publish_link_diagnostic(root, destination, receipt, "target-not-in-inventory", 100, callback,
                                                             membership=carrier))
            self.assertEqual((destination / wrapper.LINK_DETAIL_OUTPUT).read_bytes(), first)
            callback.assert_not_called()
        with tempfile.TemporaryDirectory() as name, mock.patch.object(wrapper.time, "monotonic", return_value=0):
            root, destination, receipt = self.fixture(Path(name))
            callback = mock.Mock()
            self.assertFalse(wrapper.publish_link_diagnostic(root, destination, receipt, "link-text-shape", 100, callback,
                                                             membership=carrier))
            self.assertFalse((destination / wrapper.LINK_DETAIL_OUTPUT).exists())
            callback.assert_not_called()

    def test_membership_secondary_publish_fault_retains_original_primary_and_no_authority(self):
        """A /2 publisher fault retains the exact primary bytes and emits neither a sidecar nor positive callback."""
        with tempfile.TemporaryDirectory() as name, mock.patch.object(wrapper.time, "monotonic", return_value=0):
            root, destination, receipt = self.fixture(Path(name))
            before = (destination / wrapper.OUTPUT).read_bytes()
            callback = mock.Mock()
            with mock.patch.object(wrapper.shared, "atomic_receipt", side_effect=OSError("PRIVATE")):
                self.assertFalse(wrapper.publish_link_diagnostic(root, destination, receipt, "target-not-in-inventory", 100, callback,
                                                                 membership=("relative", "beneath-present-root", "inventoried-directory")))
            self.assertEqual((destination / wrapper.OUTPUT).read_bytes(), before)
            self.assertFalse((destination / wrapper.LINK_DETAIL_OUTPUT).exists())
            callback.assert_not_called()

    def test_membership_actual_verify_carries_only_actual_refusal_tuple(self):
        """Consume a real resolve traceback through actual inventory/verify/publication with synthetic origins and no native dispatch."""
        with tempfile.TemporaryDirectory() as name:
            root, _old_output, seed = self.fixture(Path(name))
            identity = dict(seed["identity"], tracked_source_clean=True)
            genuine = self.membership_refusal(wrapper, b"pkg", self.membership_rows(wrapper))
            self.assertEqual(wrapper.link_rejection_site(genuine), "target-not-in-inventory")
            def refuse(_paths, _deadline):
                """Retain the actual engine traceback/private tuple; no detached site or target identity is constructed."""
                raise genuine
            def tools(_root, deadline):
                """Drive the actual consuming inventory adapter with controlled non-native tool observation."""
                return wrapper.stdlib_inventory(["/mock/a", "/mock/lib-dynload"], deadline)
            with mock.patch.object(wrapper.time, "monotonic", return_value=0), \
                 mock.patch.object(wrapper.sys, "platform", "linux"), \
                 mock.patch.object(wrapper, "capture_identity", side_effect=[identity, copy.deepcopy(identity)]), \
                 mock.patch.object(wrapper.shared, "checkout_binding", return_value=seed["checkout"]), \
                 mock.patch.object(wrapper, "leaf_closure", side_effect=refuse), \
                 mock.patch.object(wrapper, "capture_tools", side_effect=tools), \
                 mock.patch.object(wrapper, "native_run") as native_run:
                callback = mock.Mock()
                output = Path(name) / "membership-output"
                receipt = wrapper.verify(root, root / "forge", output, build_outcome="success",
                                         link_diagnostic_published=callback)
            self.assertEqual((receipt["status"], receipt["failure"], receipt["input_stability"]),
                             ("incomplete", "tool-untrusted", "unchanged"))
            self.assertEqual(receipt["diagnostic"], seed["diagnostic"])
            self.assertEqual(receipt["producer"], seed["producer"])
            self.assertEqual((output / wrapper.OUTPUT).read_bytes(), wrapper.shared.canonical_bytes(receipt))
            record = json.loads((output / wrapper.LINK_DETAIL_OUTPUT).read_bytes())
            self.assertTrue(wrapper.validate_link_diagnostic_record(record))
            self.assertEqual(record["membership"], wrapper.link_membership_record("target-not-in-inventory", genuine.membership))
            self.assertFalse(record["acceptance_eligible"])
            self.assertEqual(record["outer_receipt_pin"], wrapper.pin_bytes((output / wrapper.OUTPUT).read_bytes()))
            self.assertNotIn(b"/trusted", wrapper.shared.canonical_bytes(record))
            callback.assert_called_once_with()
            native_run.assert_not_called()


if __name__ == "__main__":
    unittest.main()
