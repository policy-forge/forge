#!/usr/bin/env python3
"""Ordinary worker-only controls for dispatcher accounting and redaction; no sudo or physical campaign runs."""
import copy
import hashlib
import json
from pathlib import Path
import sys
import signal
import tempfile
import types
import unittest
from unittest import mock

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import run_workspace_stdlib_closure as dispatch


class DispatcherControls(unittest.TestCase):
    """Check the narrow receipt/copy contract with mocked dispatch, never count these as Linux physical evidence."""

    def setUp(self):
        """Give each control independent exact source pins, counters and a normal closed transport fixture."""
        self.bodies = {name: b"synthetic-source" for name in dispatch.FILES}
        self.pins = {name: dispatch.byte_pin(body) for name, body in self.bodies.items()}
        self.runner = dispatch.byte_pin(dispatch.RUNNER.encode("utf-8"))
        self.value = {
            "schema_version": "forge.stdlib-closure-root-run/1", "environment": True,
            "source_pins": self.pins, "runner_pin": self.runner, "protected_copy": True,
            "source_unchanged": True, "protected_directory_removed": True, "fault": None,
            "process": {"exit_code": 0, "reaped": True, "started": True, "ownership_verified": True, "forced": False, "deadline": False,
                        "output_closed": True, "output_bound": False, "close_fault": False},
            "execution": {"schema_version": "forge.stdlib-closure-case-run/1", "discovered": 24,
                          "ran": 24, "passed": 24, "failures": 0, "errors": 0, "skips": 0,
                          "unexpected_successes": 0, "expected_failures": 0, "exact_ids": True,
                          "fixture_checks": 24, "fixture_absent": 24, "protocol_valid": True},
        }

    def transport(self, value=None, **changes):
        """Build only redacted protocol bytes so negative admission controls do not invoke a child."""
        value = self.value if value is None else value
        transport = {"exit_code": 0, "reaped": True, "output_closed": True, "fault": None,
                     "output": json.dumps(value).encode("utf-8")}
        transport.update(changes)
        return transport

    def admitted(self, value=None, **changes):
        """Exercise the real outer/result validator, keeping physical dispatch entirely mocked."""
        return dispatch.outcome(self.transport(value, **changes), self.pins, self.runner)

    def test_exact_campaign_is_a_reconciled_positive_control(self):
        """Exactly twenty-two real-looking outcomes and both cleanup records satisfy only the receipt predicate."""
        retained, good = self.admitted()
        self.assertTrue(good)
        self.assertEqual(retained, self.value)

    def test_zero_discovery_never_earns_credit(self):
        """An empty unittest run is retained as negative data instead of being promoted by exit zero."""
        self.value["execution"].update(discovered=0, ran=0, passed=0, fixture_checks=0, fixture_absent=0, exact_ids=False, protocol_valid=False)
        retained, good = self.admitted()
        self.assertFalse(good)
        self.assertEqual(retained["execution"]["ran"], 0)

    def test_skipped_class_never_earns_credit(self):
        """Actual Linux/UID0 admission cannot be replaced with discovery or class-skip success."""
        self.value["execution"].update(ran=0, passed=0, skips=1, fixture_checks=0, fixture_absent=0, protocol_valid=False)
        retained, good = self.admitted()
        self.assertFalse(good)
        self.assertEqual(retained["execution"]["skips"], 1)

    def test_failures_errors_and_unexpected_successes_stay_negative(self):
        """Each nonzero failure category refuses the whole campaign without exception prose."""
        for category in ("failures", "errors", "unexpected_successes", "expected_failures"):
            with self.subTest(category=category):
                value = copy.deepcopy(self.value)
                value["execution"][category] = 1
                self.assertFalse(self.admitted(value)[1])

    def test_float_and_boolean_denominators_are_invalid(self):
        """JSON numeric equality cannot let 24.0 or booleans masquerade as exact count observations."""
        for number in (24.0, True):
            with self.subTest(number=number):
                value = copy.deepcopy(self.value)
                value["execution"]["ran"] = number
                self.assertEqual(self.admitted(value), (None, False))

    def test_discovered_registered_ids_must_match(self):
        """The protected runner's exact-ID comparison is required independently of an apparent count of twenty-two."""
        self.value["execution"]["exact_ids"] = False
        self.assertFalse(self.admitted()[1])

    def test_ran_registered_ids_must_match(self):
        """Unknown or repeated executed IDs cannot receive credit from otherwise complete counters."""
        self.value["execution"]["protocol_valid"] = False
        self.assertFalse(self.admitted()[1])

    def test_fixture_cleanup_shortfall_revokes_credit(self):
        """A passed test body with one remaining synthetic directory is not a cleaned campaign."""
        self.value["execution"]["fixture_absent"] = 21
        self.assertFalse(self.admitted()[1])

    def test_protected_cleanup_fault_is_retained_without_credit(self):
        """An incomplete protected-copy cleanup remains an explicit negative receipt, never a safe deletion claim."""
        self.value.update(protected_directory_removed=False, fault="protected-cleanup-unverified")
        retained, good = self.admitted()
        self.assertFalse(good)
        self.assertEqual(retained["fault"], "protected-cleanup-unverified")

    def test_inner_deadline_output_and_close_faults_revoke_credit(self):
        """Every privileged resource fault invalidates otherwise passed-looking case totals."""
        for field in ("deadline", "output_bound", "close_fault", "forced"):
            with self.subTest(field=field):
                value = copy.deepcopy(self.value)
                value["process"][field] = True
                self.assertFalse(self.admitted(value)[1])

    def test_outer_deadline_output_and_close_faults_revoke_credit(self):
        """A worker transport fault cannot borrow nested success or assert the root child's cleanup."""
        for fault in ("outer-deadline", "outer-output-bound", "outer-close-unverified", "outer-cleanup-unverified"):
            with self.subTest(fault=fault):
                self.assertEqual(self.admitted(fault=fault), (None, False))

    def test_nonzero_outer_exit_rejects_passed_looking_bytes(self):
        """An emitted passed-looking result cannot override the actual sudo/bootstrap exit status."""
        retained, good = self.admitted(exit_code=1)
        self.assertEqual(retained, self.value)
        self.assertFalse(good)

    def test_source_pin_drift_is_rejected(self):
        """The root response must match the selected complete copy inventory, not just the case counts."""
        self.value["source_pins"] = copy.deepcopy(self.pins)
        self.value["source_pins"][dispatch.FILES[0]]["sha256"] = "0" * 64
        self.assertEqual(self.admitted(), (None, False))

    def test_missing_copy_pin_is_retained_without_credit(self):
        """An incomplete source copy can be described as negative evidence without claiming fixture import safety."""
        self.value["source_pins"] = {}
        self.assertFalse(self.admitted()[1])

    def test_extra_root_or_case_fields_are_not_retained(self):
        """Raw paths, payloads or exception fields cannot escape the closed result shape."""
        for nested in (False, True):
            with self.subTest(nested=nested):
                value = copy.deepcopy(self.value)
                (value["execution"] if nested else value)["secret"] = "DO-NOT-RETAIN"
                self.assertEqual(self.admitted(value), (None, False))

    def test_duplicate_members_are_refused(self):
        """Duplicate receipt keys are rejected before any last-value interpretation can grant credit."""
        transport = self.transport()
        transport["output"] = b'{"schema_version":"x","schema_version":"y"}'
        self.assertEqual(dispatch.outcome(transport, self.pins, self.runner), (None, False))

    def test_payload_limits_precede_privileged_dispatch(self):
        """Oversized raw source bodies fail before encoding or a sudo invocation can occur."""
        bodies = dict(self.bodies)
        bodies[dispatch.FILES[0]] = b"x" * (dispatch.SOURCE_LIMIT + 1)
        with mock.patch.object(dispatch.subprocess, "Popen") as process:
            with self.assertRaises(ValueError):
                dispatch.copy_payload(bodies)
            process.assert_not_called()

    def test_source_names_cannot_select_other_paths(self):
        """The source envelope has an exact four-basename domain and no caller-selected path or command."""
        bodies = dict(self.bodies)
        bodies["../arbitrary.py"] = b"source"
        with self.assertRaises(ValueError):
            dispatch.copy_payload(bodies)

    def test_fixed_tools_pin_actual_descriptor_bytes_at_selected_paths(self):
        """Both absolute tool selections hash real fixture bytes; only root-trusted admission is mocked."""
        with tempfile.TemporaryDirectory(prefix="forge-fixed-tool-") as directory:
            for role, selected in (("python", "/usr/bin/python3"), ("sudo", "/usr/bin/sudo")):
                with self.subTest(role=role):
                    path = Path(directory) / role
                    body = (b"\x00fixed-tool-fixture\xff\n" + role.encode("ascii")) * 1000
                    path.write_bytes(body)
                    with mock.patch.object(dispatch.qualifier, "administration_tool", return_value=path) as fixed:
                        value = dispatch.fixed_tool(role)
                    fixed.assert_called_once_with(selected)
                    self.assertEqual(value, {"selected": selected, "resolved": str(path),
                                             "pin": {"bytes": len(body), "sha256": hashlib.sha256(body).hexdigest()}})

    def test_unknown_tool_role_refuses_before_admission(self):
        """An undeclared role or a caller-supplied executable path cannot reach trusted-tool admission."""
        for role in ("ip", "/usr/bin/python3", ""):
            with self.subTest(role=role), mock.patch.object(dispatch.qualifier, "administration_tool") as fixed:
                with self.assertRaises(KeyError):
                    dispatch.fixed_tool(role)
                fixed.assert_not_called()

    def test_fixed_tool_absence_has_no_fallback(self):
        """Missing selected distro tools refuse after one absolute-path admission attempt without a fallback."""
        for role, selected in (("python", "/usr/bin/python3"), ("sudo", "/usr/bin/sudo")):
            missing = dispatch.qualifier.GateError("tool-unavailable", True, tool_reason="missing")
            with self.subTest(role=role), mock.patch.object(dispatch.qualifier, "administration_tool", side_effect=missing) as fixed:
                with self.assertRaises(dispatch.qualifier.GateError):
                    dispatch.fixed_tool(role)
                fixed.assert_called_once_with(selected)

    def test_failed_process_creation_is_unqualified_and_redacted(self):
        """Startup errors produce fixed fields without their raw exception arguments or a fictitious reap."""
        with mock.patch.object(dispatch.subprocess, "Popen", side_effect=OSError("SECRET")):
            value = dispatch.run_root(dispatch.copy_payload(self.bodies))
        self.assertIsNotNone(value["fault"])
        self.assertFalse(value["reaped"])
        self.assertEqual(value["output"], b"")
        self.assertNotIn("SECRET", str(value))


def bootstrap_namespace():
    """Load reviewed literal declarations at Root's deterministic trace filename; never call the privileged main guard."""
    filename = str(Path(dispatch.__file__).resolve()) + "::root_bootstrap.py"
    namespace = {"__name__": "protected_bootstrap_controls", "__file__": filename}
    exec(compile(dispatch.BOOTSTRAP, filename, "exec"), namespace)
    return namespace


class DirectChildControls(unittest.TestCase):
    """Mock every wait/signal primitive to check direct-child ownership without spawning or signalling any process."""

    def engines(self):
        """Exercise both actual ordinary declarations and the separately attributed bootstrap declarations."""
        root = bootstrap_namespace()
        return (("ordinary", dispatch.DirectChild, dispatch),
                ("protected-bootstrap", root["DirectChild"], types.SimpleNamespace(**root)))

    def wait_owner(self, engine):
        """Mock only the declared single-thread/default-SIGCHLD assumptions, with no real wait or signal."""
        stack = __import__("contextlib").ExitStack()
        thread = object()
        stack.enter_context(mock.patch.object(engine.signal, "getsignal", return_value=signal.SIG_DFL))
        stack.enter_context(mock.patch.object(engine.threading, "current_thread", return_value=thread))
        stack.enter_context(mock.patch.object(engine.threading, "main_thread", return_value=thread))
        stack.enter_context(mock.patch.object(engine.threading, "enumerate", return_value=[thread]))
        return stack

    def test_already_reaped_direct_child_is_never_signalled(self):
        """An actual wait status latches exit permanently; later settlement cannot signal a recycled numeric PID."""
        for name, owner_type, engine in self.engines():
            with self.subTest(engine=name), self.wait_owner(engine), mock.patch.object(engine.os, "waitpid", return_value=(312, 0)) as wait, mock.patch.object(engine.os, "kill") as kill:
                child = owner_type(types.SimpleNamespace(pid=312, returncode=None))
                self.assertTrue(child.observe())
                self.assertTrue(child.stop())
                self.assertTrue(child.observe())
                self.assertEqual(child.exit_code, 0)
                wait.assert_called_once_with(312, engine.os.WNOHANG)
                kill.assert_not_called()

    def test_fault_settlement_signals_only_original_unreaped_child_once(self):
        """A pending child is killed directly once and receives an exit only from the subsequent mocked wait status."""
        for name, owner_type, engine in self.engines():
            with self.subTest(engine=name), self.wait_owner(engine), mock.patch.object(engine.os, "waitpid", side_effect=[(0, 0), (312, signal.SIGKILL)]), mock.patch.object(engine.os, "kill") as kill:
                process = types.SimpleNamespace(pid=312, returncode=None)
                child = owner_type(process)
                self.assertTrue(child.stop())
                self.assertEqual(child.exit_code, -signal.SIGKILL)
                self.assertEqual(process.returncode, -signal.SIGKILL)
                self.assertTrue(child.stop())
                kill.assert_called_once_with(312, signal.SIGKILL)

    def test_external_reap_uncertainty_never_signals_or_fabricates_exit(self):
        """ECHILD revokes sole ownership permanently rather than synthesizing zero or signalling the old numeric PID."""
        for name, owner_type, engine in self.engines():
            with self.subTest(engine=name), self.wait_owner(engine), mock.patch.object(engine.os, "waitpid", side_effect=ChildProcessError) as wait, mock.patch.object(engine.os, "kill") as kill:
                child = owner_type(types.SimpleNamespace(pid=312, returncode=None))
                self.assertFalse(child.stop())
                self.assertFalse(child.stop())
                self.assertFalse(child.reaped)
                self.assertFalse(child.certain)
                self.assertIsNone(child.exit_code)
                wait.assert_called_once()
                kill.assert_not_called()

    def test_settlement_timeout_keeps_unverified_exit(self):
        """A timed-out settlement stays unverified and cannot renew its wait ceiling or signal the child again."""
        for name, owner_type, engine in self.engines():
            with self.subTest(engine=name), self.wait_owner(engine), mock.patch.object(engine.os, "waitpid", return_value=(0, 0)) as wait, mock.patch.object(engine.os, "kill") as kill, mock.patch.object(engine.time, "monotonic", side_effect=[0, 6]):
                child = owner_type(types.SimpleNamespace(pid=312, returncode=None))
                self.assertFalse(child.stop())
                self.assertFalse(child.stop())
                self.assertFalse(child.reaped)
                self.assertIsNone(child.exit_code)
                wait.assert_called_once_with(312, engine.os.WNOHANG)
                kill.assert_called_once_with(312, signal.SIGKILL)

    def test_inherited_ignored_sigchld_refuses_before_outer_spawn(self):
        """Auto-reap policy is unsupported; the dispatcher declines before creating a sudo child."""
        with mock.patch.object(dispatch.signal, "getsignal", return_value=signal.SIG_IGN), mock.patch.object(dispatch.subprocess, "Popen") as spawn:
            value = dispatch.run_root(dispatch.copy_payload({name: b"source" for name in dispatch.FILES}))
        spawn.assert_not_called()
        self.assertFalse(value["reaped"])
        self.assertIsNone(value["exit_code"])
        self.assertIsNotNone(value["fault"])

    def test_changed_sigchld_or_other_python_thread_never_signals(self):
        """Unqualified handler/thread ownership after construction revokes signal authority and leaves exit unknown."""
        for name, owner_type, engine in self.engines():
            for changed in ("handler", "threads"):
                with self.subTest(engine=name, changed=changed), self.wait_owner(engine), mock.patch.object(engine.os, "waitpid") as wait, mock.patch.object(engine.os, "kill") as kill:
                    patch = (mock.patch.object(engine.signal, "getsignal", return_value=signal.SIG_IGN)
                             if changed == "handler" else mock.patch.object(engine.threading, "enumerate", return_value=[object(), object()]))
                    with patch:
                        child = owner_type(types.SimpleNamespace(pid=312, returncode=None))
                        self.assertFalse(child.stop())
                    wait.assert_not_called()
                    kill.assert_not_called()
                    self.assertFalse(child.certain)
                    self.assertIsNone(child.exit_code)


if __name__ == "__main__":
    unittest.main()
