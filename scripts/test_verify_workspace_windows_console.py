#!/usr/bin/env python3
"""Exercise wrapper evidence failure boundaries with synthetic receipts; never launch Forge or ConPTY."""
import copy
import contextlib
import io
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

import verify_workspace_windows_console as wrapper


class WindowsWrapperTests(unittest.TestCase):
    """Use real bounded files and mocked tool/producer observations, without native acceptance credit."""

    def setUp(self):
        """Create private fixture inputs and release bytes so no project state or real tools are used."""
        self.private = tempfile.TemporaryDirectory(prefix="forge-console-wrapper-control-")
        self.addCleanup(self.private.cleanup)
        self.root = Path(self.private.name)
        self.forge = self.root / "fixture-forge.exe"
        self.forge.write_bytes(b"synthetic-release-bytes\x00\r\n")
        for name in wrapper.EXTRA_INPUTS:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(("synthetic-source:" + name).encode())
        self.output = self.root / "outer"
        self.base_identity = {"source_commit": "a" * 40, "ordered_parents": [], "tracked_source_clean": True,
                              "inputs": {}, "provided_release_binary": wrapper.shared.hash_file(self.forge)}
        self.tools = {"python": "3.11.14", "os": "Windows", "os_release": "10", "machine": "AMD64",
                      "rust_host": "x86_64-pc-windows-msvc", "rustc": {"version": "1.98.1"}, "cargo": {"version": "1.98.1"}}
        self.identity = copy.deepcopy(self.base_identity)
        self.identity["inputs"] = {name: wrapper.shared.hash_file(self.root / name) for name in wrapper.EXTRA_INPUTS}

    def native_fixture(self, status="passed"):
        """Author explicitly synthetic closed native observations for reader and producer-exit controls."""
        pin = {"bytes": 42, "sha256": "b" * 64}
        value = {"schema_version": wrapper.NATIVE_SCHEMA, "scope": "browser-session-console-smoke", "truth_state": "synthetic-development",
                 "acceptance_eligible": False, "status": status, "scoped_complete": status == "passed",
                 "platform": {"system": "Windows", "build": 26100, "architecture": "x64", "python": self.tools["python"]},
                 "identity": {"harness": copy.deepcopy(self.identity["inputs"][wrapper.EXTRA_INPUTS[0]]), "provided_release_binary": {"before": copy.deepcopy(self.identity["provided_release_binary"]), "after": copy.deepcopy(self.identity["provided_release_binary"])}},
                 "fixture": {"before": pin, "after": pin, "unchanged": True},
                 "checks": {**{name: name != "forced_cleanup" for name in wrapper.BOOL_CHECKS}, "prompts": 2, "no_echo_modes": 2, "input_writes": 2, "terminal_bytes": 1024,
                            "responses": [{"method": method, "path": path, "status": code} for method, path, code in wrapper.RESPONSES]},
                 "failure": {"stage": None, "code": None}}
        if status != "passed":
            value["failure"] = {"stage": "terminal", "code": "terminal-output-limit"}
            value["checks"]["terminal_bounded"] = False
        if status == "incomplete":
            value.update(platform={"system": "unsupported", "build": None, "architecture": "unsupported", "python": self.tools["python"]}, failure={"stage": "platform", "code": "unsupported-platform"})
            value["checks"] = {**{name: False for name in wrapper.BOOL_CHECKS}, **{name: 0 for name in wrapper.COUNT_LIMITS}, "responses": []}
            value["fixture"] = {"before": None, "after": None, "unchanged": False}
        return value

    def read_fixture(self, value, producer_exit=0):
        """Serialize fixture bytes through the real reader with bound harness and release identities."""
        path = self.root / "native-reader.json"
        path.write_bytes(wrapper.shared.canonical_bytes(value))
        return wrapper.read_native_receipt(path, producer_exit, self.identity, self.tools)

    def run_wrapper(self, *, native=None, producer_exit=0, command_failure=None, build="success", after=None, tools_after=None, source_dirty=False, mutate=None, context=None):
        """Mock external executions while retaining actual input hashing, receipt reading and publication."""
        before = copy.deepcopy(self.base_identity)
        before["tracked_source_clean"] = not source_dirty
        after = copy.deepcopy(before if after is None else after)
        executions = []

        def producer(command, root, timeout):
            """Supply only a synthetic producer artifact; retain no raw output in the wrapper receipt."""
            executions.append((command, root, timeout))
            directory = Path(command[command.index("--output-dir") + 1])
            if native is not False:
                value = self.native_fixture() if native is None else copy.deepcopy(native)
                (directory / wrapper.NATIVE_FILENAME).write_bytes(wrapper.shared.canonical_bytes(value))
            if mutate is not None:
                mutate()
            return {"exit_code": producer_exit, "failure": command_failure, "output": b"PRIVATE-CONSOLE-TOKEN /private/location HTTP-body"}

        with mock.patch.object(wrapper.shared, "capture_identity", side_effect=[before, after]), \
             mock.patch.object(wrapper.shared, "tool_versions", side_effect=[self.tools, self.tools if tools_after is None else tools_after]), \
             mock.patch.object(wrapper.shared, "run_command", side_effect=producer):
            value = wrapper.verify(self.root, self.forge, self.output, build_outcome=build, **(context or {}))
        return value, executions

    def test_complete_synthetic_fixture_preserves_scope_and_pins(self):
        """Exercise the pass path while retaining development truth, open gates and exact release/source hashes."""
        value, executions = self.run_wrapper()
        self.assertEqual(value["status"], "passed")
        self.assertFalse(value["acceptance_eligible"])
        self.assertEqual(value["input_stability"], "unchanged")
        self.assertEqual(value["tool_stability"], "unchanged")
        self.assertEqual(value["identity"]["provided_release_binary"], self.identity["provided_release_binary"])
        self.assertEqual(set(value["identity"]["inputs"]), set(wrapper.EXTRA_INPUTS))
        self.assertEqual(len(executions), 1)
        self.assertIn("final-roadmap-documentation-review", value["pending_gates"])
        self.assertNotIn("PRIVATE-CONSOLE", (self.output / wrapper.OUTPUT_FILENAME).read_text())
        self.assertFalse(value["checkout"]["hosted_context_asserted"])

    def test_missing_failed_or_unverified_producer_never_passes(self):
        """Reject missing receipts, passed-looking nonzero exits and command cleanup/output failures."""
        for index, arguments in enumerate(({"native": False}, {"producer_exit": 1}, {"producer_exit": None}, {"command_failure": "subprocess-output-not-closed"}, {"command_failure": "output-bound-exceeded"})):
            with self.subTest(arguments=arguments):
                self.output = self.root / f"producer-{index}"
                value, _ = self.run_wrapper(**arguments)
                self.assertEqual(value["status"], "failed")
                self.assertIsNone(value["native"]["receipt"])
                self.assertNotIn("PRIVATE-CONSOLE", json.dumps(value))

    def test_valid_failed_and_unsupported_producers_are_distinct(self):
        """Retain bounded failed evidence and an unsupported incomplete result without skip-as-pass credit."""
        for status, exit_code, expected in (("failed", 1, "failed"), ("incomplete", 2, "incomplete")):
            self.output = self.root / status
            value, _ = self.run_wrapper(native=self.native_fixture(status), producer_exit=exit_code)
            self.assertEqual(value["status"], expected)
            self.assertEqual(value["native"]["receipt"]["status"], status)
            self.assertFalse(value["native"]["receipt"]["scoped_complete"])

    def test_build_and_dirty_inputs_prevent_native_execution(self):
        """Refuse to execute native work from dirty sources or an absent/failed/skipped/cancelled build assertion."""
        for index, arguments in enumerate(({"source_dirty": True}, *({"build": outcome} for outcome in ("unrecorded", "failure", "skipped", "cancelled")))):
            self.output = self.root / f"precondition-{index}"
            value, executions = self.run_wrapper(**arguments)
            self.assertNotEqual(value["status"], "passed")
            self.assertEqual(executions, [])
            self.assertEqual(value["native"]["status"], "not-run")

    def test_source_and_release_drift_override_native_pass(self):
        """Fail outer verification when source commit or supplied release hashes drift after a synthetic native pass."""
        for name in ("source_commit", "provided_release_binary"):
            self.output = self.root / name
            after = copy.deepcopy(self.base_identity)
            after[name] = "c" * 40 if name == "source_commit" else {"bytes": 99, "sha256": "c" * 64}
            value, _ = self.run_wrapper(after=after)
            self.assertEqual(value["status"], "failed")
            self.assertEqual(value["input_stability"], "changed")
            self.assertEqual(value["native"]["status"], "passed")

    def test_each_supplemented_input_drift_fails(self):
        """Bind all four new source/control files through actual byte hashing before and after execution."""
        for index, name in enumerate(wrapper.EXTRA_INPUTS):
            self.output = self.root / f"input-{index}"
            path = self.root / name
            original = path.read_bytes()
            value, _ = self.run_wrapper(mutate=lambda: path.write_bytes(original + b"changed"))
            self.assertEqual(value["status"], "failed")
            self.assertEqual(value["input_stability"], "changed")
            path.write_bytes(original)

    def test_tool_missing_or_changed_fails_closed(self):
        """Missing Rust identity blocks execution, and a changed post-run tool version revokes synthetic pass."""
        self.tools["rustc"] = None
        value, executions = self.run_wrapper()
        self.assertEqual(value["status"], "failed")
        self.assertEqual(executions, [])
        self.tools["rustc"] = {"version": "1.98.1"}
        self.output = self.root / "tool-changed"
        changed = copy.deepcopy(self.tools)
        changed["python"] = "3.12.0"
        value, _ = self.run_wrapper(tools_after=changed)
        self.assertEqual(value["status"], "failed")
        self.assertEqual(value["tool_stability"], "changed")

    def test_hosted_parent_order_and_context_are_not_local_credit(self):
        """Reuse exact merge-parent context guards; wrong order prevents execution rather than degrading to local."""
        base, head = "b" * 40, "c" * 40
        self.base_identity["ordered_parents"] = [base, head]
        context = {"event": "pull_request", "checkout_kind": "pull-request-merge", "expected_commit": "a" * 40, "requested_base": base, "requested_head": head}
        value, _ = self.run_wrapper(context=context)
        self.assertEqual(value["status"], "passed")
        self.assertTrue(value["checkout"]["hosted_context_asserted"])
        self.output = self.root / "wrong-parents"
        self.base_identity["ordered_parents"] = [head, base]
        value, executions = self.run_wrapper(context=context)
        self.assertEqual(value["status"], "failed")
        self.assertEqual(executions, [])

    def test_closed_shape_and_integral_types(self):
        """Reject private extra fields, booleans/floats as counts, and missing native observations."""
        paths = (("private",), ("checks", "private"), ("platform", "private"), ("identity", "harness", "private"), ("fixture", "private"), ("failure", "private"))
        for path in paths:
            value = self.native_fixture()
            target = value
            for part in path[:-1]:
                target = target[part]
            target[path[-1]] = "PRIVATE"
            with self.subTest(path=path), self.assertRaises(ValueError):
                self.read_fixture(value)
        for bad in (True, 2.0, -1, 3):
            value = self.native_fixture()
            value["checks"]["prompts"] = bad
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                self.read_fixture(value)
        for bad in (True, 42.0):
            value = self.native_fixture()
            value["identity"]["harness"]["bytes"] = bad
            with self.assertRaises(ValueError):
                self.read_fixture(value)

    def test_native_pass_denominators_and_receipt_binding(self):
        """Reject partial cleanup/response/fixture/binary/harness observations even with producer exit zero."""
        mutations = (lambda value: value["checks"].update(forced_cleanup=True),
                     lambda value: value["checks"].update(terminal_eof=False),
                     lambda value: value["checks"].update(terminal_bytes=65537),
                     lambda value: value["checks"]["responses"].pop(),
                     lambda value: value["fixture"].update(unchanged=False),
                     lambda value: value["identity"]["provided_release_binary"].update(after={"bytes": 10, "sha256": "c" * 64}),
                     lambda value: value["identity"]["harness"].update(sha256="c" * 64),
                     lambda value: value["platform"].update(build=17762),
                     lambda value: value.update(acceptance_eligible=True))
        for index, mutate in enumerate(mutations):
            value = self.native_fixture()
            mutate(value)
            with self.subTest(index=index), self.assertRaises(ValueError):
                self.read_fixture(value)

    def test_response_method_order_and_outcome(self):
        """Bind the six actual response tuple slots and retain a failed known-route status without pass credit."""
        for field, bad in (("method", "POST"), ("path", "/api/v1/effects/commits"), ("status", True), ("status", 500)):
            value = self.native_fixture()
            value["checks"]["responses"][0][field] = bad
            with self.subTest(field=field, bad=bad), self.assertRaises(ValueError):
                self.read_fixture(value)
        value = self.native_fixture("failed")
        value["checks"]["responses"] = [{"method": "GET", "path": "/api/v1/project/summary", "status": 500}]
        actual, _ = self.read_fixture(value, 1)
        self.assertEqual(actual["checks"]["responses"][0]["status"], 500)

    def test_duplicate_nonfinite_and_oversized_receipts(self):
        """Reject raw nested duplicate keys/nonfinite values and oversized bytes without dropping controls."""
        path = self.root / "malformed.json"
        raw = wrapper.shared.canonical_bytes(self.native_fixture())
        cases = (raw.replace(b'"prompts":2', b'"prompts":2,"pro\\u006dpts":2'),
                 raw.replace(b'"terminal_bytes":1024', b'"terminal_bytes":NaN'), b" " * (wrapper.MAX_NATIVE_RECEIPT + 1), b"{broken")
        for index, data in enumerate(cases):
            path.write_bytes(data)
            with self.subTest(index=index), self.assertRaises((ValueError, UnicodeError)):
                wrapper.read_native_receipt(path, 0, self.identity, self.tools)
    def test_link_receipts_are_rejected_or_explicitly_unavailable(self):
        """Reject links with a distinct explicit skip when the host cannot create this fixture."""
        target = self.root / "target.json"
        target.write_bytes(wrapper.shared.canonical_bytes(self.native_fixture()))
        link = self.root / "link.json"
        try:
            link.symlink_to(target)
        except OSError:
            self.skipTest("Symbolic-link fixture unavailable; no platform acceptance credit")
        with self.assertRaises(ValueError):
            wrapper.native_bytes(link)

    def test_nonregular_artifacts_fail_before_open(self):
        """Reject a directory and a mocked FIFO mode before os.open, without a platform-specific pipe fixture."""
        directory = self.root / "receipt-directory"
        directory.mkdir()
        with mock.patch.object(wrapper.os, "open") as opened, self.assertRaises(ValueError):
            wrapper.native_bytes(directory)
        opened.assert_not_called()
        pipe_stat = os.stat_result((0o010600, 1, 1, 1, 1, 1, 0, 0, 0, 0))
        with mock.patch.object(Path, "lstat", return_value=pipe_stat), mock.patch.object(wrapper.os, "open") as opened, self.assertRaises(ValueError):
            wrapper.native_bytes(self.root / "receipt-fifo")
        opened.assert_not_called()

    def test_unsupported_cannot_invent_native_flags(self):
        """A valid unsupported fixture has no observed native counts, flags or responses."""
        value = self.native_fixture("incomplete")
        value["checks"]["console_job"] = True
        with self.assertRaises(ValueError):
            self.read_fixture(value, 2)
        value = self.native_fixture("incomplete")
        value["failure"] = {"stage": "cleanup", "code": "cleanup-unverified"}
        with self.assertRaises(ValueError):
            self.read_fixture(value, 2)

    def test_fixture_mutations_do_not_taint_expected_bindings(self):
        """Keep malformed receipt controls isolated from expected identities and later valid fixtures."""
        original = copy.deepcopy(self.identity)
        bad = self.native_fixture()
        bad["identity"]["harness"]["sha256"] = "c" * 64
        bad["identity"]["provided_release_binary"]["before"]["bytes"] = 1
        self.assertEqual(self.identity, original)
        with self.assertRaises(ValueError):
            self.read_fixture(bad)
        valid, _ = self.read_fixture(self.native_fixture())
        self.assertEqual(valid["identity"]["harness"], original["inputs"][wrapper.EXTRA_INPUTS[0]])

    def test_existing_output_is_never_replaced(self):
        """Refuse nonempty evidence storage before running a helper or replacing a sentinel receipt."""
        self.output.mkdir()
        sentinel = self.output / wrapper.OUTPUT_FILENAME
        sentinel.write_bytes(b"sentinel")
        with mock.patch.object(wrapper, "capture_identity") as capture, self.assertRaises(ValueError):
            wrapper.verify(self.root, self.forge, self.output)
        capture.assert_not_called()
        self.assertEqual(sentinel.read_bytes(), b"sentinel")

    def test_main_publication_error_is_fixed_nonzero(self):
        """Treat a publisher exception as failed process evidence without exposing arbitrary exception text."""
        arguments = ["verify", "--forge", str(self.forge), "--output-dir", str(self.output)]
        captured = io.StringIO()
        with mock.patch.object(sys, "argv", arguments), mock.patch.object(wrapper, "verify", side_effect=OSError("PRIVATE /absolute/path")), contextlib.redirect_stderr(captured):
            self.assertEqual(wrapper.main(), 2)
        self.assertNotIn("PRIVATE", captured.getvalue())
        self.assertNotIn(str(self.root), captured.getvalue())

    def test_unexpected_input_exception_is_redacted_and_failed(self):
        """Convert an unexpected input/tool exception into fixed failure evidence without leaking its text."""
        with mock.patch.object(wrapper, "capture_identity", side_effect=RuntimeError("PRIVATE /absolute/path")):
            value = wrapper.verify(self.root, self.forge, self.output, build_outcome="success")
        self.assertEqual(value["status"], "failed")
        self.assertEqual(value["failure"], "verification-input-invalid")
        self.assertIsNone(value["native"]["receipt"])
        self.assertNotIn("PRIVATE", (self.output / wrapper.OUTPUT_FILENAME).read_text())

    def test_main_forwards_hosted_identifiers_and_build(self):
        """Forward all parsed checkout fields into the source-bound wrapper rather than silently granting local context."""
        arguments = ["verify", "--forge", str(self.forge), "--output-dir", str(self.output), "--expected-commit", "a" * 40,
                     "--event", "pull_request", "--checkout-kind", "pull-request-merge", "--requested-base", "b" * 40, "--requested-head", "c" * 40, "--build-outcome", "success"]
        with mock.patch.object(sys, "argv", arguments), mock.patch.object(wrapper, "verify", return_value={"status": "passed"}) as verify, contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(wrapper.main(), 0)
        self.assertEqual(verify.call_args.kwargs, {"event": "pull_request", "checkout_kind": "pull-request-merge", "requested_head": "c" * 40, "requested_base": "b" * 40})
        self.assertEqual(verify.call_args.args[4:6], ("a" * 40, "success"))


if __name__ == "__main__":
    unittest.main()
