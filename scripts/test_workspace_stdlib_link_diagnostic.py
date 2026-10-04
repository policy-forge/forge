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
                             "engine_pin", "outer_receipt_pin"})
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


if __name__ == "__main__":
    unittest.main()
