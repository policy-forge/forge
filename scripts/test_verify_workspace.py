#!/usr/bin/env python3
"""Synthetic stdlib tests for F04 receipt boundaries; never run Cargo/Forge."""
import copy
import json
import io
from pathlib import Path
import sys
import subprocess
import tempfile
import unittest.mock

sys.dont_write_bytecode = True
import verify_workspace as verifier
import test_workspace_client as client_test

ROOT = Path(__file__).resolve().parents[1]
SUMMARY = b"test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n"
CHECKS = [
    "upload_requires_confirmation", "idempotent_commit_replays_exact_bytes", "conversion_succeeds",
    "explicit_no_relationship_is_preserved", "omitted_scope_decisions_remain_under_review",
    "explicit_scope_decision_validates", "committed_scope_analysis_succeeds",
    "export_matches_committed_redacted_bytes", "read_only_summary_reconciles_resources",
    "read_only_mutation_is_rejected", "clean_shutdown_writable", "clean_shutdown_read_only",
    "metadata_bundle_preview_preserves_complete_denominator",
    "metadata_bundle_fingerprints_match_exact_bytes",
    "registered_bundle_comparison_reconciles_expected_current",
    "metadata_bundle_queries_preserve_workspace_files",
]


def client_receipt(declared):
    """Build a synthetic current-format unit fixture without claiming live operation outcomes."""
    observed = sorted(declared)[:2]
    return {
        "schema_version": "forge.workspace-client-verification/2", "status": "passed",
        "checks": CHECKS, "check_count": len(CHECKS), "declared_operation_count": len(declared),
        "observed_operations": observed, "unobserved_operations": sorted(set(declared) - set(observed)),
        "operation_outcomes": {name: {"succeeded": 1, "rejected": 0, "transport_failed": 0}
                               for name in observed},
    }


class ReceiptTests(unittest.TestCase):
    def setUp(self):
        """Create an isolated receipt directory, derive operations and reset shared accounting for every case."""
        self.temporary = tempfile.TemporaryDirectory(prefix="forge-receipt-test-")
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.declared = [name for _, _, name in verifier.contract_routes(ROOT)]
        self.reset_recording()
        self.addCleanup(self.reset_recording)

    def reset_recording(self):
        """Restore shared recorder state before and after each isolated unit case."""
        client_test.RecordingWorkspace.routes = verifier.contract_routes(ROOT)
        client_test.RecordingWorkspace.outcomes = {}
        client_test.RecordingWorkspace.request_count = 0
        client_test.RecordingWorkspace.accounting_failed = False

    def write_client(self, value):
        """Write one canonical synthetic receipt to the isolated fixture directory."""
        path = self.directory / "client.json"
        path.write_bytes(verifier.canonical_bytes(value))
        return path

    def fixture_run(self, *, selected=verifier.SUITES, result=None, dirty=False,
                    drift=False, tool_missing=False, build="success", expected_commit=None,
                    client_published=False):
        """Replay synthetic bindings and optionally leave a passed-looking client file despite a failed producer exit."""
        identity = {"source_commit": "a" * 40, "ordered_parents": [], "tracked_source_clean": not dirty,
                    "inputs": {name: {"sha256": "b" * 64, "bytes": 10} for name in verifier.INPUTS},
                    "provided_release_binary": {"sha256": "c" * 64, "bytes": 20}}
        after = copy.deepcopy(identity)
        if drift:
            after["provided_release_binary"]["sha256"] = "d" * 64
        versions = {"python": "3.11.0", "os": "Linux", "os_release": "1", "machine": "x86_64",
                    "cargo": {"version": "1.99.0"}, "rustc": {"version": "1.99.0"},
                    "rust_host": "x86_64-unknown-linux-gnu"}
        if tool_missing:
            versions["rustc"] = None
        calls = []
        def command(arguments, root, timeout):
            """Substitute bounded suite output and conditionally publish a synthetic receipt for exit-status controls."""
            calls.append(arguments)
            if "--receipt" in arguments:
                if client_published or result is None or result.get("exit_code") == 0:
                    path = Path(arguments[arguments.index("--receipt") + 1])
                    path.write_bytes(verifier.canonical_bytes(client_receipt(self.declared)))
                    if client_published:
                        self.assertEqual(json.loads(path.read_bytes())["status"], "passed")
            return result or {"exit_code": 0, "failure": None, "output": SUMMARY}
        output = self.directory / "evidence"
        with unittest.mock.patch.object(verifier, "capture_identity", side_effect=[identity, after]), \
             unittest.mock.patch.object(verifier, "tool_versions", return_value=versions), \
             unittest.mock.patch.object(verifier, "run_command", side_effect=command):
            receipt = verifier.verify(ROOT, self.directory / "synthetic-binary", output,
                                      selected, expected_commit=expected_commit, build_outcome=build)
        return receipt, calls, output

    def test_canonical_publication_refuses_overwrite(self):
        """Check canonical atomic publication preserves the first destination and removes private staging files."""
        path = self.directory / "receipt.json"
        verifier.atomic_receipt(path, {"z": 2, "a": "opaque"})
        self.assertEqual(path.read_bytes(), b'{"a":"opaque","z":2}\n')
        with self.assertRaises(FileExistsError):
            verifier.atomic_receipt(path, {"different": True})
        self.assertEqual(path.read_bytes(), b'{"a":"opaque","z":2}\n')
        self.assertEqual(sorted(item.name for item in self.directory.iterdir()), ["receipt.json"])

    def test_hash_preserves_binary_crlf_and_rejects_symlink(self):
        """Hash exact binary CRLF bytes and reject a symlink input when platform privileges permit the control."""
        binary = self.directory / "binary"
        binary.write_bytes(b"a\r\nb\x00")
        self.assertEqual(verifier.hash_file(binary)["bytes"], 5)
        self.assertEqual(verifier.hash_file(binary)["sha256"], verifier.hashlib.sha256(b"a\r\nb\x00").hexdigest())
        link = self.directory / "link"
        try:
            link.symlink_to(binary)
        except OSError:
            self.skipTest("Symlink creation requires a platform privilege")
        with self.assertRaises(ValueError):
            verifier.hash_file(link)

    def test_contract_inventory_uses_parameterized_documented_routes(self):
        """Derive the current 39-route inventory and match parameterized paths exactly."""
        routes = verifier.contract_routes(ROOT)
        self.assertEqual(len(routes), 39)
        self.assertEqual([(verb, name) for verb, pattern, name in routes
                          if pattern.fullmatch("/api/v1/operations/op_opaque")], [("GET", "getOperation")])
        self.assertFalse(any(pattern.fullmatch("/api/v1/operations/op_opaque/extra")
                             for _, pattern, _ in routes))

    def test_duplicate_contract_operation_rejected(self):
        """Reject duplicate operation identifiers rather than accepting an ambiguous API denominator."""
        root = self.directory / "repo"
        path = root / "docs/api/forge-workspace-v1.openapi.yaml"
        path.parent.mkdir(parents=True)
        path.write_text("paths:\n  /api/v1/first:\n    get:\n      operationId: same\n  /api/v1/second:\n    post:\n      operationId: same\n")
        with self.assertRaises(ValueError):
            verifier.contract_routes(root)

    def test_operation_names_with_underscore_and_hyphen_preserve_full_inventory(self):
        """Keep supported underscore and hyphen operation identifiers in the complete contract inventory."""
        root = self.directory / "repo"
        path = root / "docs/api/forge-workspace-v1.openapi.yaml"
        path.parent.mkdir(parents=True)
        path.write_text("paths:\n  /api/v1/first:\n    get:\n      operationId: new_operation\n  /api/v1/second:\n    head:\n      operationId: new-operation\ncomponents:\n")
        self.assertEqual([name for _, _, name in verifier.contract_routes(root)], ["new_operation", "new-operation"])

    def test_unsupported_contract_lines_fail_instead_of_reducing_denominator(self):
        """Reject unsupported YAML declarations instead of silently losing documented operations."""
        root = self.directory / "repo"
        path = root / "docs/api/forge-workspace-v1.openapi.yaml"
        path.parent.mkdir(parents=True)
        first = "paths:\n  /api/v1/first:\n    get:\n      operationId: first\n"
        second = "  /api/v1/second:\n    get:\n      operationId: second\n"
        invalid = (
            second.replace("operationId: second", 'operationId: "second"'),
            second.replace("operationId: second", "operationId: 42"),
            second.replace("operationId: second", '"operationId": second'),
            second.replace("operationId: second", "operationId : second"),
            second.replace("operationId: second", "operationId: second value"),
            second.replace("      operationId: second", "     operationId: second"),
            second.replace("    get:", '    "get":'),
            second.replace("    get:", "    GET:"),
            second.replace("    get:", "    connect:"),
            second.replace("  /api/v1/second:", '  "/api/v1/second":'),
            second.replace("    get:\n      operationId: second\n", "    get:\n      summary: Missing operation ID\n"),
        )
        for suffix in invalid:
            path.write_text(first + suffix + "components:\n")
            with self.subTest(suffix=suffix), self.assertRaises(ValueError):
                verifier.contract_routes(root)
        for prefix in ('"paths":\n', "paths: {}\n"):
            path.write_text(prefix + second)
            with self.subTest(prefix=prefix), self.assertRaises(ValueError):
                verifier.contract_routes(root)

    def test_contract_read_is_bounded_before_utf8_decode(self):
        """Enforce the raw contract byte bound before decoding and reject invalid UTF-8."""
        stream = io.BytesIO(b"\xff" * 4096)
        with unittest.mock.patch.object(verifier, "MAX_CAPTURE", 64), \
             unittest.mock.patch.object(Path, "open", return_value=stream), \
             unittest.mock.patch.object(Path, "read_text", side_effect=AssertionError("Unbounded text read")):
            with self.assertRaisesRegex(ValueError, "exceeds its bound"):
                verifier.contract_routes(ROOT)
        root = self.directory / "repo"
        path = root / "docs/api/forge-workspace-v1.openapi.yaml"
        path.parent.mkdir(parents=True)
        path.write_bytes(b"paths:\n\xff")
        with self.assertRaises(UnicodeDecodeError):
            verifier.contract_routes(root)

    def test_rust_summary_rejects_empty_ambiguous_and_missing(self):
        """Require one nonempty Rust summary and withhold pass credit for failed or unexercised cases."""
        counts, passed = verifier.rust_summary(SUMMARY)
        self.assertTrue(passed)
        self.assertEqual(counts["passed"], 8)
        for output in (b"", SUMMARY + SUMMARY, SUMMARY.replace(b"8 passed", b"0 passed")):
            with self.subTest(output=output), self.assertRaises(ValueError):
                verifier.rust_summary(output)
        for output in (SUMMARY.replace(b"0 ignored", b"1 ignored"),
                       SUMMARY.replace(b"0 filtered out", b"1 filtered out"),
                       SUMMARY.replace(b"0 measured", b"1 measured"),
                       SUMMARY.replace(b"0 failed", b"1 failed")):
            self.assertFalse(verifier.rust_summary(output)[1])

    def test_client_receipt_closed_and_denominator_consistent(self):
        """Reject extra members, missing checks, inconsistent denominators and invalid outcome cell types."""
        value = client_receipt(self.declared)
        self.assertEqual(verifier.read_client_receipt(self.write_client(value), self.declared), value)
        for alter in (
            lambda v: v.update(extra="SECRET CAPABILITY"),
            lambda v: v.update(check_count=99),
            lambda v: v.update(checks=CHECKS[:-1]),
            lambda v: v.update(declared_operation_count=1),
            lambda v: v.update(unobserved_operations=[]),
            lambda v: v["operation_outcomes"][v["observed_operations"][0]].update(succeeded=True),
        ):
            invalid = copy.deepcopy(value)
            alter(invalid)
            with self.subTest(invalid=invalid), self.assertRaises((ValueError, TypeError)):
                verifier.read_client_receipt(self.write_client(invalid), self.declared)

    def test_client_duplicate_keys_and_nonfinite_numbers_rejected(self):
        """Reject duplicate JSON keys, nonfinite literals and non-object receipt roots."""
        path = self.directory / "client.json"
        for raw in (b'{"status":"passed","status":"failed"}', b'{"status":NaN}', b'[]'):
            path.write_bytes(raw)
            with self.subTest(raw=raw), self.assertRaises(ValueError):
                verifier.read_client_receipt(path, self.declared)

    def test_all_suites_pass_only_with_clean_stable_inputs(self):
        """Accept the synthetic complete suite only with stable clean bindings and retain pending acceptance gates."""
        receipt, calls, output = self.fixture_run()
        self.assertEqual(receipt["status"], "passed")
        self.assertEqual(receipt["input_stability"], "unchanged")
        self.assertEqual(len(calls), 3)
        self.assertTrue(all("--offline" in command and "--locked" in command for command in calls[:2]))
        self.assertEqual(calls[2][0], sys.executable)
        self.assertEqual(json.loads((output / "workspace-verification.json").read_bytes()), receipt)
        self.assertIn("packaged-runtime-os-network-denial", receipt["pending_gates"])
        self.assertIn("final-roadmap-documentation-review", receipt["pending_gates"])

    def test_per_suite_binding_does_not_assign_release_bytes_to_cargo_tests(self):
        """Keep Cargo test-profile binding separate from the provided binary used by maintained-client execution."""
        receipt, _, _ = self.fixture_run()
        release = receipt["identity"]["provided_release_binary"]
        self.assertNotIn("binary", receipt["identity"])
        for suite in ("api-contract", "api-workflow"):
            execution = receipt["suites"][suite]["execution"]
            self.assertEqual(execution["cargo_profile"], "test")
            self.assertEqual(execution["executable_binding"], "source-lock-tool-test-pins")
            self.assertFalse(execution["binary_hash_measured"])
            self.assertIn("test_source", execution)
            self.assertIn("cargo_lock", execution)
            self.assertIn("rustc", execution["tools"])
            self.assertNotIn("provided_release_binary", execution)
            self.assertNotIn(release["sha256"], json.dumps(execution))
        execution = receipt["suites"]["maintained-client"]["execution"]
        self.assertEqual(execution["cargo_profile"], "release")
        self.assertEqual(execution["executable_binding"], "provided-release-bytes")
        self.assertEqual(execution["provided_release_binary"], release)
        self.assertIn("python", execution["tools"])

    def test_measured_rust_suite_is_incomplete_at_slice_level(self):
        """Withhold complete-suite credit when a reported Rust case was measured rather than exercised."""
        receipt, _, _ = self.fixture_run(selected=("api-contract",), result={
            "exit_code": 0, "failure": None, "output": SUMMARY.replace(b"0 measured", b"1 measured")})
        self.assertEqual(receipt["status"], "incomplete")
        self.assertEqual(receipt["suites"]["api-contract"]["status"], "incomplete")
        self.assertEqual(receipt["suites"]["api-contract"]["counts"]["measured"], 1)

    def test_unselected_suite_is_incomplete(self):
        """Retain an unselected suite as incomplete rather than treating omission as success."""
        receipt, calls, _ = self.fixture_run(selected=("api-contract",))
        self.assertEqual(receipt["status"], "incomplete")
        self.assertEqual(len(calls), 1)
        self.assertEqual(receipt["suites"]["maintained-client"]["failure"], "not-selected")

    def test_failed_suite_does_not_suppress_remaining_suites_or_leak_output(self):
        """Continue remaining suites after failure while keeping native paths and raw subprocess output out of receipts."""
        receipt, calls, output = self.fixture_run(result={"exit_code": 9, "failure": None,
                                                       "output": b"/private/absolute/path SECRET CAPABILITY PRIVATE PROSE"})
        self.assertEqual(receipt["status"], "failed")
        self.assertEqual(len(calls), 3)
        self.assertTrue(all(row["status"] == "failed" for row in receipt["suites"].values()))
        raw = (output / "workspace-verification.json").read_bytes()
        for forbidden in (b"/private", b"SECRET", b"CAPABILITY", b"PRIVATE PROSE"):
            self.assertNotIn(forbidden, raw)

    def test_zero_exit_without_summary_is_not_success(self):
        """Reject a zero-exit Rust process whose output lacks the required test denominator."""
        receipt, _, _ = self.fixture_run(selected=("api-contract",),
                                        result={"exit_code": 0, "failure": None, "output": b"looks green"})
        self.assertEqual(receipt["suites"]["api-contract"]["failure"], "invalid-suite-receipt")
        self.assertEqual(receipt["status"], "failed")

    def test_dirty_source_missing_tool_or_unqualified_build_is_incomplete(self):
        """Keep otherwise passing synthetic suites incomplete when source, tool or build binding is unqualified."""
        for options in ({"dirty": True}, {"tool_missing": True}, {"build": "failure"}, {"build": "unrecorded"}):
            with self.subTest(options=options):
                self.temporary.cleanup()
                self.setUp()
                receipt, _, _ = self.fixture_run(**options)
                self.assertEqual(receipt["status"], "incomplete")
                self.assertTrue(all(row["status"] == "passed" for row in receipt["suites"].values()))

    def test_source_drift_fails_after_suites(self):
        """Fail verification if exact provided-input identity changes after the selected suites run."""
        receipt, calls, _ = self.fixture_run(drift=True)
        self.assertEqual(receipt["status"], "failed")
        self.assertEqual(receipt["input_stability"], "changed")
        self.assertEqual(len(calls), 3)

    def test_wrong_commit_fails_before_executing_suites(self):
        """Reject a mismatched requested source commit before invoking any suite."""
        receipt, calls, _ = self.fixture_run(expected_commit="f" * 40)
        self.assertEqual(receipt["status"], "failed")
        self.assertEqual(calls, [])
        self.assertTrue(all(row["status"] == "incomplete" for row in receipt["suites"].values()))

    def test_existing_output_directory_is_preserved(self):
        """Refuse an existing evidence directory and preserve its historical receipt bytes."""
        output = self.directory / "evidence"
        output.mkdir()
        previous = output / "historical.json"
        previous.write_bytes(b"immutable historical evidence\n")
        with self.assertRaises(ValueError):
            verifier.verify(ROOT, self.directory / "binary", output, verifier.SUITES)
        self.assertEqual(previous.read_bytes(), b"immutable historical evidence\n")

    def test_process_capture_is_bounded_and_timeout_is_explicit(self):
        """Exercise bounded subprocess capture and explicit timeout labels using small synthetic Python children."""
        success = verifier.run_command(
            [sys.executable, "-c", "import sys; sys.stdout.buffer.write(b'synthetic tool output\\n')"], ROOT, 5
        )
        self.assertEqual(success["exit_code"], 0)
        self.assertEqual(success["output"], b"synthetic tool output\n")
        with unittest.mock.patch.object(verifier, "MAX_CAPTURE", 1024):
            overflow = verifier.run_command([sys.executable, "-c", "print('x' * 4096)"], ROOT, 5)
        self.assertEqual(overflow["failure"], "output-bound-exceeded")
        self.assertEqual(overflow["output"], b"")
        timed = verifier.run_command([sys.executable, "-c", "import time; time.sleep(5)"], ROOT, 0.1)
        self.assertEqual(timed["failure"], "suite-timeout")
        self.assertEqual(timed["output"], b"")

    def test_tool_versions_do_not_copy_untrusted_banners(self):
        """Retain no validated tool identity when a banner contains only untrusted private text."""
        with unittest.mock.patch.object(verifier, "run_command", return_value={"exit_code": 0, "failure": None,
                                                                     "output": b"/private/path SECRET CAPABILITY"}):
            versions = verifier.tool_versions(ROOT, 1)
        self.assertIsNone(versions["cargo"])
        self.assertIsNone(versions["rustc"])
        self.assertIsNone(versions["rust_host"])
        self.assertNotIn("SECRET", json.dumps(versions))

    def recording_instance(self):
        """Reset shared recorder fixtures and return an unstarted client for accounting controls."""
        self.reset_recording()
        instance = object.__new__(client_test.RecordingWorkspace)
        instance._api_prefix = "/api/v1"
        return instance

    def test_unexpected_shutdown_error_is_sticky_after_unchanged_close(self):
        """Exercise unchanged close swallowing a delegated error while accounting still rejects success."""
        instance = self.recording_instance()
        instance._capability = "synthetic-private-capability"
        instance.process = unittest.mock.Mock()
        instance.process.poll.side_effect = [None, None]
        instance.process.stdout = None
        with unittest.mock.patch.object(client_test.Workspace, "request", side_effect=RuntimeError("SECRET PRIVATE PROSE")):
            instance.close()
        instance.process.wait.assert_called_once_with(timeout=3)
        self.assertEqual(instance._capability, "")
        self.assertEqual(client_test.RecordingWorkspace.request_count, 1)
        self.assertEqual(client_test.RecordingWorkspace.outcomes, {})
        self.assertTrue(client_test.RecordingWorkspace.accounting_failed)
        with self.assertRaises(RuntimeError) as caught:
            client_test.RecordingWorkspace.ensure_accounting()
        self.assertNotIn("SECRET", str(caught.exception))
        self.assertNotIn("PRIVATE", str(caught.exception))

    def test_unknown_route_stays_failed_after_a_reconciled_valid_request(self):
        """Keep unknown-route failure sticky even when later documented calls reconcile exactly."""
        instance = self.recording_instance()
        with unittest.mock.patch.object(client_test.Workspace, "request", return_value={}) as delegate:
            with self.assertRaises(RuntimeError):
                instance.request("GET", "/api/private/SECRET")
            delegate.assert_not_called()
            instance.request("GET", "/api/v1/project/summary")
            self.assertEqual(delegate.call_count, 1)
        self.assertEqual(client_test.RecordingWorkspace.request_count, 1)
        self.assertEqual(client_test.RecordingWorkspace.outcomes, {
            "getProjectSummary": {"succeeded": 1, "rejected": 0, "transport_failed": 0},
        })
        with self.assertRaises(RuntimeError):
            client_test.RecordingWorkspace.ensure_accounting()

    def test_unreconciled_attempts_latch_failure_after_counts_are_repaired(self):
        """Reject a missing outcome and keep the latch after synthetic counts are made equal."""
        self.recording_instance()
        client_test.RecordingWorkspace.request_count = 2
        client_test.RecordingWorkspace.outcomes = {
            "getProjectSummary": {"succeeded": 1, "rejected": 0, "transport_failed": 0},
        }
        with self.assertRaises(RuntimeError):
            client_test.RecordingWorkspace.ensure_accounting()
        client_test.RecordingWorkspace.outcomes["getProjectSummary"]["succeeded"] = 2
        with self.assertRaises(RuntimeError):
            client_test.RecordingWorkspace.ensure_accounting()

    def test_exact_shared_budget_is_a_reconciled_positive_control(self):
        """Accept a closed synthetic exact-budget inventory without pretending to dispatch those requests."""
        self.recording_instance()
        bound = client_test.MAX_RECORDED_REQUESTS
        client_test.RecordingWorkspace.request_count = bound
        client_test.RecordingWorkspace.outcomes = {
            "getProjectSummary": {"succeeded": bound - 1, "rejected": 0, "transport_failed": 0},
            "shutdownSession": {"succeeded": 1, "rejected": 0, "transport_failed": 0},
        }
        client_test.RecordingWorkspace.ensure_accounting()
        self.assertFalse(client_test.RecordingWorkspace.accounting_failed)

    def test_client_reader_enforces_aggregate_budget_across_cells_and_operations(self):
        """Accept exactly ten thousand synthetic attempts and reject one more across either dimension."""
        value = client_receipt(self.declared)
        first, second = value["observed_operations"]
        value["operation_outcomes"][first]["succeeded"] = verifier.MAX_CLIENT_REQUESTS - 1
        self.assertEqual(verifier.read_client_receipt(self.write_client(value), self.declared), value)
        too_many_operations = copy.deepcopy(value)
        too_many_operations["operation_outcomes"][second]["rejected"] = 1
        too_many_cells = copy.deepcopy(value)
        too_many_cells["operation_outcomes"][first]["succeeded"] = verifier.MAX_CLIENT_REQUESTS
        for invalid in (too_many_operations, too_many_cells):
            with self.subTest(outcomes=invalid["operation_outcomes"]), self.assertRaises(ValueError):
                verifier.read_client_receipt(self.write_client(invalid), self.declared)

    def test_client_reader_requires_exact_integer_header_counts(self):
        """Reject float and boolean count headers while accepting the ordinary integer fixture."""
        value = client_receipt(self.declared)
        self.assertEqual(verifier.read_client_receipt(self.write_client(value), self.declared), value)
        for key in ("check_count", "declared_operation_count"):
            for replacement in (float(value[key]), True, False):
                invalid = copy.deepcopy(value)
                invalid[key] = replacement
                with self.subTest(key=key, replacement=replacement), self.assertRaises(ValueError):
                    verifier.read_client_receipt(self.write_client(invalid), self.declared)

    def test_setup_resets_shared_budget_and_failure_for_the_next_case(self):
        """Invoke actual setup after dirty shared state and permit a reconciled next-case request."""
        client_test.RecordingWorkspace.request_count = client_test.MAX_RECORDED_REQUESTS
        client_test.RecordingWorkspace.accounting_failed = True
        client_test.RecordingWorkspace.outcomes = {
            "getProjectSummary": {"succeeded": client_test.MAX_RECORDED_REQUESTS,
                                  "rejected": 0, "transport_failed": 0},
        }
        self.setUp()
        self.assertEqual(client_test.RecordingWorkspace.request_count, 0)
        self.assertFalse(client_test.RecordingWorkspace.accounting_failed)
        self.assertEqual(client_test.RecordingWorkspace.outcomes, {})
        instance = object.__new__(client_test.RecordingWorkspace)
        with unittest.mock.patch.object(client_test.Workspace, "request", return_value={}):
            instance.request("GET", "/api/v1/project/summary")
        self.assertEqual(client_test.RecordingWorkspace.request_count, 1)
        client_test.RecordingWorkspace.ensure_accounting()

    def test_post_link_cleanup_error_leaves_committed_file_with_failed_exit(self):
        """Exercise real hard-link publication with injected cleanup failure and retain nonzero redacted status."""
        path = self.directory / "client.json"
        value = client_receipt(self.declared)
        real_unlink = verifier.os.unlink
        def fail_staging_unlink(target, *arguments, **options):
            """Fail only private staging removal after the destination hard link actually exists."""
            if Path(target).name.startswith(".receipt-"):
                self.assertTrue(path.exists())
                raise OSError("/private/SECRET staging cleanup failure")
            return real_unlink(target, *arguments, **options)
        with unittest.mock.patch.object(sys, "argv", ["test_workspace_client.py", "--forge", "/private/binary",
                                            "--receipt", str(path)]), \
             unittest.mock.patch.object(client_test, "run", return_value=value), \
             unittest.mock.patch.object(verifier.os, "unlink", side_effect=fail_staging_unlink), \
             unittest.mock.patch.object(sys, "stderr", new_callable=io.StringIO) as stderr, \
             unittest.mock.patch.object(sys, "stdout", new_callable=io.StringIO) as stdout:
            self.assertEqual(client_test.main(), 1)
        self.assertEqual(path.read_bytes(), verifier.canonical_bytes(value))
        self.assertEqual(json.loads(path.read_bytes())["status"], "passed")
        self.assertEqual(stdout.getvalue(), "")
        self.assertEqual(stderr.getvalue(), "Maintained headless client receipt publication failed.\n")
        self.assertNotIn("SECRET", stderr.getvalue())
        self.assertNotIn("/private", stderr.getvalue())
        staging = [item for item in self.directory.iterdir() if item.name.startswith(".receipt-")]
        self.assertEqual(len(staging), 1)
        for item in staging:
            real_unlink(item)
        self.assertTrue(path.exists())

    def test_failed_producer_exit_rejects_a_passed_looking_receipt_file(self):
        """Reject a synthetic exit-one producer even when its complete passed-looking bytes exist."""
        with unittest.mock.patch.object(verifier, "read_client_receipt", side_effect=AssertionError("Do not read a failed producer")) as reader:
            receipt, calls, _ = self.fixture_run(selected=("maintained-client",), client_published=True,
                                                result={"exit_code": 1, "failure": None, "output": b"static failure"})
        reader.assert_not_called()
        self.assertEqual(len(calls), 1)
        self.assertEqual(receipt["status"], "failed")
        row = receipt["suites"]["maintained-client"]
        self.assertEqual(row["exit_code"], 1)
        self.assertEqual(row["status"], "failed")
        self.assertIsNone(row["counts"])
        self.assertNotIn("coverage", row)

    def test_historical_v1_client_receipt_is_not_current_success(self):
        """Reject a historical format even when all current group and denominator fields fit."""
        value = client_receipt(self.declared)
        value["schema_version"] = "forge.workspace-client-verification/1"
        with self.assertRaises(ValueError):
            verifier.read_client_receipt(self.write_client(value), self.declared)

    def test_metadata_bundle_routes_are_in_current_closed_inventory(self):
        """Recognize both current metadata queries without opening a live workspace."""
        routes = verifier.contract_routes(ROOT)
        for method, path, expected in (
            ("GET", "/api/v1/project/bundle-preview", "getProjectBundlePreview"),
            ("POST", "/api/v1/project/bundle-verifications", "verifyProjectBundle"),
        ):
            matched = [name for verb, pattern, name in routes
                       if verb == method and pattern.fullmatch(path)]
            self.assertEqual(matched, [expected])

    def test_recording_client_keeps_closed_typed_outcomes(self):
        """Retain only success, typed rejection and transport-failure counts."""
        instance = object.__new__(client_test.RecordingWorkspace)
        instance.routes = verifier.contract_routes(ROOT)
        instance.outcomes = {}
        with unittest.mock.patch.object(client_test.RecordingWorkspace, "request_count", 0), \
             unittest.mock.patch.object(client_test.Workspace, "request", return_value={"private": "SECRET"}):
            self.assertEqual(instance.request("GET", "/api/v1/project/bundle-preview?private=SECRET"),
                             {"private": "SECRET"})
        with unittest.mock.patch.object(client_test.RecordingWorkspace, "request_count", 0), \
             unittest.mock.patch.object(client_test.Workspace, "request", side_effect=client_test.WorkspaceError({
                 "code": "synthetic-rejection", "message": "SECRET PRIVATE PROSE",
             })):
            with self.assertRaises(client_test.WorkspaceError):
                instance.request("GET", "/api/v1/project/bundle-preview")
        with unittest.mock.patch.object(client_test.RecordingWorkspace, "request_count", 0), \
             unittest.mock.patch.object(client_test.Workspace, "request", side_effect=OSError("/private/SECRET")):
            with self.assertRaises(OSError):
                instance.request("GET", "/api/v1/project/bundle-preview")
        self.assertEqual(instance.outcomes, {
            "getProjectBundlePreview": {"succeeded": 1, "rejected": 1, "transport_failed": 1},
        })
        self.assertNotIn("SECRET", json.dumps(instance.outcomes))
        self.assertNotIn("private", json.dumps(instance.outcomes))

    def test_recording_request_budget_fails_before_an_extra_delegate_call(self):
        """Stop an over-budget workflow without classifying an unissued request."""
        instance = object.__new__(client_test.RecordingWorkspace)
        instance.routes = verifier.contract_routes(ROOT)
        instance.outcomes = {}
        with unittest.mock.patch.object(client_test, "MAX_RECORDED_REQUESTS", 2), \
             unittest.mock.patch.object(client_test.RecordingWorkspace, "request_count", 0), \
             unittest.mock.patch.object(client_test.Workspace, "request", return_value={}) as delegate:
            instance.request("GET", "/api/v1/project/summary")
            instance.request("GET", "/api/v1/project/summary")
            with self.assertRaises(RuntimeError):
                instance.request("GET", "/api/v1/project/summary")
            self.assertEqual(delegate.call_count, 2)
            self.assertEqual(client_test.RecordingWorkspace.request_count, 2)
            self.assertTrue(client_test.RecordingWorkspace.accounting_failed)
            with self.assertRaises(RuntimeError):
                client_test.RecordingWorkspace.ensure_accounting()
        self.assertEqual(instance.outcomes, {
            "getProjectSummary": {"succeeded": 2, "rejected": 0, "transport_failed": 0},
        })

    def test_existing_client_receipt_is_preserved_before_workflow(self):
        """Reject an existing destination before starting Forge or replacing its bytes."""
        path = self.directory / "historical.json"
        previous = b"immutable historical evidence\n"
        path.write_bytes(previous)
        with unittest.mock.patch.object(sys, "argv", ["test_workspace_client.py", "--forge", "/private/binary",
                                            "--receipt", str(path)]), \
             unittest.mock.patch.object(client_test, "run") as run, \
             unittest.mock.patch.object(sys, "stderr", new_callable=io.StringIO) as stderr:
            self.assertEqual(client_test.main(), 1)
        run.assert_not_called()
        self.assertEqual(path.read_bytes(), previous)
        self.assertNotIn("/private", stderr.getvalue())

    def test_dangling_client_receipt_symlink_is_preserved(self):
        """Reject a dangling destination link before executing the workflow."""
        path = self.directory / "receipt-link.json"
        try:
            path.symlink_to(self.directory / "absent.json")
        except OSError:
            self.skipTest("Symlink creation requires a platform privilege")
        with unittest.mock.patch.object(sys, "argv", ["test_workspace_client.py", "--forge", "/private/binary",
                                            "--receipt", str(path)]), \
             unittest.mock.patch.object(client_test, "run") as run, \
             unittest.mock.patch.object(sys, "stderr", new_callable=io.StringIO) as stderr:
            self.assertEqual(client_test.main(), 1)
        run.assert_not_called()
        self.assertTrue(path.is_symlink())
        self.assertFalse(path.exists())
        self.assertNotIn("/private", stderr.getvalue())

    def test_client_receipt_publication_failure_is_redacted_and_failed(self):
        """Fail successful-workflow publication without copying native exception text."""
        path = self.directory / "client.json"
        with unittest.mock.patch.object(sys, "argv", ["test_workspace_client.py", "--forge", "/private/binary",
                                            "--receipt", str(path)]), \
             unittest.mock.patch.object(client_test, "run", return_value=client_receipt(self.declared)), \
             unittest.mock.patch.object(client_test, "atomic_receipt", side_effect=OSError("/private/SECRET")), \
             unittest.mock.patch.object(sys, "stderr", new_callable=io.StringIO) as stderr, \
             unittest.mock.patch.object(sys, "stdout", new_callable=io.StringIO) as stdout:
            self.assertEqual(client_test.main(), 1)
        self.assertFalse(path.exists())
        self.assertEqual(stdout.getvalue(), "")
        self.assertNotIn("SECRET", stderr.getvalue())
        self.assertNotIn("/private", stderr.getvalue())

    def test_failed_client_and_failed_publication_expose_no_exception_text(self):
        """Keep a conformance and publication double failure redacted with nonzero status."""
        path = self.directory / "client.json"
        with unittest.mock.patch.object(sys, "argv", ["test_workspace_client.py", "--forge", "/private/binary",
                                            "--receipt", str(path)]), \
             unittest.mock.patch.object(client_test, "run", side_effect=AssertionError("SECRET PRIVATE PROSE")), \
             unittest.mock.patch.object(client_test, "atomic_receipt", side_effect=OSError("/private/SECRET")), \
             unittest.mock.patch.object(sys, "stderr", new_callable=io.StringIO) as stderr:
            self.assertEqual(client_test.main(), 1)
        self.assertFalse(path.exists())
        self.assertNotIn("SECRET", stderr.getvalue())
        self.assertNotIn("/private", stderr.getvalue())

    def test_recording_client_retains_only_documented_identifiers(self):
        """Record only a matched operation identifier and reject undocumented paths without retaining query values."""
        instance = object.__new__(client_test.RecordingWorkspace)
        instance.routes = verifier.contract_routes(ROOT)
        instance.outcomes = {}
        with unittest.mock.patch.object(client_test.Workspace, "request", return_value={"private": "response"}):
            self.assertEqual(instance.request("GET", "/api/v1/operations/op_private?secret=private"), {"private": "response"})
        self.assertEqual(instance.outcomes, {"getOperation": {"succeeded": 1, "rejected": 0, "transport_failed": 0}})
        self.assertNotIn("private", json.dumps(instance.outcomes))
        with self.assertRaises(RuntimeError):
            instance.request("GET", "/api/private-path")

    def test_client_failure_receipt_excludes_exception_payload(self):
        """Publish only a failed zero-group receipt when a mocked workflow raises private exception text."""
        path = self.directory / "failure.json"
        with unittest.mock.patch.object(sys, "argv", ["test_workspace_client.py", "--forge", "/private/binary", "--receipt", str(path)]), \
             unittest.mock.patch.object(client_test, "run", side_effect=AssertionError("SECRET CAPABILITY PRIVATE PROSE")):
            self.assertEqual(client_test.main(), 1)
        value = json.loads(path.read_bytes())
        self.assertEqual(value["status"], "failed")
        self.assertEqual(value["check_count"], 0)
        self.assertNotIn(b"SECRET", path.read_bytes())
        self.assertNotIn(b"/private", path.read_bytes())


class CheckoutBindingTests(unittest.TestCase):
    """Exercise raw commit capture and context binding without running Forge or Cargo."""

    def setUp(self):
        """Allocate a private fixture directory, with cleanup owned by this test case."""
        self.temporary = tempfile.TemporaryDirectory(prefix="forge-checkout-test-")
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)

    def object_id(self, raw, algorithm="sha1"):
        """Derive the actual Git object envelope hash from synthetic raw commit bytes."""
        return verifier.hashlib.new(algorithm, b"commit " + str(len(raw)).encode("ascii") + b"\0" + raw,
                                   usedforsecurity=False).hexdigest()

    def commit_fixture(self, parents=(), algorithm="sha1", tree=None):
        """Author a valid commit object with private markers that must never enter the identity."""
        width = 40 if algorithm == "sha1" else 64
        raw = ("tree " + (tree or "0" * width) + "\n" +
               "".join("parent " + parent + "\n" for parent in parents) +
               "author Synthetic PRIVATE AUTHOR <unit@example.invalid> 1 +0000\n" +
               "committer Synthetic PRIVATE COMMITTER <unit@example.invalid> 1 +0000\n\n" +
               "PRIVATE MESSAGE /private/path CAPABILITY\n").encode("ascii")
        return raw, self.object_id(raw, algorithm)

    def identity_fixture(self, commit, parents):
        """Build the same closed captured fields as the production identity for context controls."""
        return {"source_commit": commit, "ordered_parents": list(parents), "tracked_source_clean": True,
                "inputs": {name: {"sha256": "d" * 64, "bytes": 10} for name in verifier.INPUTS},
                "provided_release_binary": {"sha256": "e" * 64, "bytes": 20}}

    def test_commit_object_hash_algorithms_and_ordered_metadata(self):
        """Accept exact SHA-1/SHA-256 objects while retaining neither authors nor message text."""
        for algorithm, width in (("sha1", 40), ("sha256", 64)):
            with self.subTest(algorithm=algorithm):
                parents = ["a" * width, "b" * width]
                raw, commit = self.commit_fixture(parents, algorithm)
                self.assertEqual(verifier.commit_parents(raw, commit), parents)
                captured = self.identity_fixture(commit, verifier.commit_parents(raw, commit))
                self.assertNotIn("PRIVATE", json.dumps(captured))
                self.assertNotIn("CAPABILITY", json.dumps(captured))
                self.assertNotIn("/private/path", json.dumps(captured))

    def test_commit_object_malformed_and_oversized_headers_fail(self):
        """Reject intrinsic malformed objects even when their computed object identifier matches."""
        raw, commit = self.commit_fixture(["a" * 40, "b" * 40])
        malformed = [raw.replace(b"\n\n", b"\n", 1), raw.replace(b"tree " + b"0" * 40, b"tree bad"),
                     raw.replace(b"parent " + b"a" * 40, b"parent " + b"A" * 40),
                     raw.replace(b"parent " + b"b" * 40, b"parent " + b"a" * 40),
                     raw.replace(b"author Synthetic", b"author \0Synthetic"),
                     raw.replace(b"\n\n", b"\nparent " + b"c" * 40 + b"\n\n", 1),
                     raw.replace(b"author Synthetic", b"author " + b"x" * 65536 + b"Synthetic")]
        too_many, _ = self.commit_fixture([format(i + 1, "040x") for i in range(257)])
        malformed.append(too_many)
        at_bound, at_bound_commit = self.commit_fixture([format(i + 1, "040x") for i in range(256)])
        self.assertEqual(len(verifier.commit_parents(at_bound, at_bound_commit)), 256)
        for value in malformed:
            with self.subTest(bytes=len(value)):
                with self.assertRaises(ValueError):
                    verifier.commit_parents(value, self.object_id(value))
        with self.assertRaises(ValueError):
            verifier.commit_parents(raw, "f" * 40)
        with self.assertRaises(ValueError):
            verifier.commit_parents(raw, "INVALID")
        with self.assertRaises(ValueError):
            verifier.commit_parents("not bytes", commit)
        with self.assertRaises(ValueError):
            verifier.commit_parents(b"", commit)
        with unittest.mock.patch.object(verifier, "MAX_CAPTURE", len(raw) - 1):
            with self.assertRaises(ValueError):
                verifier.commit_parents(raw, commit)

    def test_actual_shallow_git_capture_needs_no_parent_objects(self):
        """Capture a real temporary shallow HEAD with unavailable parents and discard raw private headers."""
        repo = self.directory / "repo"
        # Confine both template and configured hooks to this private fixture.
        empty_hooks = self.directory / "empty-hooks"
        empty_hooks.mkdir()
        git_command = ["git", "-c", "core.hooksPath=" + str(empty_hooks)]
        subprocess.run([*git_command, "init", "--quiet", "--object-format=sha1", "--template=", str(repo)], check=True,
                       stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        tree = subprocess.run([*git_command, "-C", str(repo), "hash-object", "-t", "tree", "-w", "--stdin"],
                              input=b"", check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout.strip().decode("ascii")
        parents = ["a" * 40, "b" * 40]
        raw, commit = self.commit_fixture(parents, tree=tree)
        written = subprocess.run([*git_command, "-C", str(repo), "hash-object", "-t", "commit", "-w", "--stdin"],
                                 input=raw, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout.strip().decode("ascii")
        self.assertEqual(written, commit)
        subprocess.run([*git_command, "-C", str(repo), "update-ref", "refs/heads/fixture", commit],
                       check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        subprocess.run([*git_command, "-C", str(repo), "symbolic-ref", "HEAD", "refs/heads/fixture"],
                       check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        (repo / ".git/shallow").write_text(commit + "\n")
        for parent in parents:
            missing = subprocess.run([*git_command, "-C", str(repo), "cat-file", "-e", parent],
                                     stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            self.assertNotEqual(missing.returncode, 0)
        with unittest.mock.patch.object(verifier, "hash_file", return_value={"sha256": "d" * 64, "bytes": 10}):
            captured = verifier.capture_identity(repo, self.directory / "fixture-binary", 5)
        self.assertEqual(captured["source_commit"], commit)
        self.assertEqual(captured["ordered_parents"], parents)
        self.assertTrue(captured["tracked_source_clean"])
        self.assertNotIn("PRIVATE", json.dumps(captured))
        self.assertNotIn("CAPABILITY", json.dumps(captured))

    def test_capture_rejects_object_mismatch_and_observed_head_race(self):
        """Reject raw-object mismatch or a changed HEAD before any dirty-state success can be credited."""
        raw, commit = self.commit_fixture(["a" * 40])
        different, other = self.commit_fixture(["b" * 40])
        ok = {"exit_code": 0, "failure": None}
        cases = [
            [{**ok, "output": commit.encode()}, {**ok, "output": different}],
            [{**ok, "output": commit.encode()}, {**ok, "output": raw}, {**ok, "output": other.encode()}],
            [{**ok, "output": commit.encode()}, {"exit_code": 1, "failure": None, "output": b""}],
        ]
        for replies in cases:
            with self.subTest(reply_count=len(replies)), \
                 unittest.mock.patch.object(verifier, "hash_file", return_value={"sha256": "d" * 64, "bytes": 10}), \
                 unittest.mock.patch.object(verifier, "run_command", side_effect=replies) as commands:
                with self.assertRaises(ValueError):
                    verifier.capture_identity(self.directory, self.directory / "fixture-binary", 5)
                self.assertFalse(any(call.args[0][1] == "status" for call in commands.call_args_list))

    def test_pull_request_merge_exact_order_and_requested_identifiers(self):
        """Require tested merge hash and precisely ordered requested base then head."""
        for algorithm, width in (("sha1", 40), ("sha256", 64)):
            base, head = "a" * width, "b" * width
            _, merge = self.commit_fixture([base, head], algorithm)
            identity = self.identity_fixture(merge, [base, head])
            arguments = {"event": "pull_request", "checkout_kind": "pull-request-merge",
                         "requested_head": head, "requested_base": base}
            context = verifier.checkout_binding(identity, merge, **arguments)
            self.assertEqual(context["ordered_parents"], [base, head])
            self.assertEqual(context["tested_commit"], merge)
            self.assertTrue(context["hosted_context_asserted"])
            for change in ({"requested_head": "c" * width}, {"requested_base": "c" * width}):
                with self.subTest(algorithm=algorithm, change=change), self.assertRaises(ValueError):
                    verifier.checkout_binding(identity, merge, **{**arguments, **change})
            for parents in ([head, base], [base], [base, head, "c" * width]):
                with self.subTest(parents=parents), self.assertRaises(ValueError):
                    verifier.checkout_binding(self.identity_fixture(merge, parents), merge, **arguments)
            with self.assertRaises(ValueError):
                verifier.checkout_binding(identity, "f" * width, **arguments)

    def test_head_events_bind_tested_requested_head(self):
        """Accept push/manual/explicit PR-head context only when tested commit equals the requested head."""
        _, head = self.commit_fixture(["a" * 40])
        identity = self.identity_fixture(head, ["a" * 40])
        for event in ("push", "workflow_dispatch", "pull_request"):
            arguments = {"event": event, "checkout_kind": "head", "requested_head": head,
                         "requested_base": "b" * 40 if event == "pull_request" else None}
            context = verifier.checkout_binding(identity, head, **arguments)
            self.assertEqual(context["kind"], "head")
            self.assertEqual(context["requested_head"], head)
            with self.subTest(event=event), self.assertRaises(ValueError):
                verifier.checkout_binding(identity, head, **{**arguments, "requested_head": "f" * 40})

    def test_local_optional_expected_hash_has_no_hosted_credit(self):
        """Allow local tested-hash pinning while rejecting requested hosted head/base fields."""
        _, commit = self.commit_fixture()
        identity = self.identity_fixture(commit, [])
        for expected in (None, commit):
            context = verifier.checkout_binding(identity, expected)
            self.assertFalse(context["hosted_context_asserted"])
            self.assertEqual(context["binding"], "local-commit-object-only")
            self.assertIsNone(context["requested_head"])
        for fields in ({"requested_head": commit}, {"requested_base": "b" * 40},
                       {"requested_head": ""}, {"requested_base": ""}, {"checkout_kind": "head"}):
            with self.subTest(fields=fields), self.assertRaises(ValueError):
                verifier.checkout_binding(identity, commit, **fields)
        with self.assertRaises(ValueError):
            verifier.checkout_binding(identity, "f" * 40)

    def test_hosted_unknown_event_and_malformed_context_fail_closed(self):
        """Reject missing expected hash, wrong-width IDs and unsupported event/kind combinations."""
        _, commit = self.commit_fixture()
        identity = self.identity_fixture(commit, [])
        defaults = {"event": "push", "checkout_kind": "head", "requested_head": commit}
        invalid = [({**defaults}, None), ({**defaults}, ""), ({**defaults}, "A" * 40),
                   ({**defaults, "event": "workflow_run"}, commit),
                   ({**defaults, "checkout_kind": "pull-request-merge"}, commit),
                   ({**defaults, "checkout_kind": "local"}, commit),
                   ({**defaults, "requested_base": "b" * 40}, commit),
                   ({**defaults, "requested_head": "b" * 64}, commit),
                   ({**defaults, "event": "pull_request", "requested_base": None}, commit),
                   ({**defaults, "event": "pull_request", "requested_base": "BAD"}, commit)]
        for arguments, expected in invalid:
            with self.subTest(arguments=arguments, expected=expected), self.assertRaises(ValueError):
                verifier.checkout_binding(identity, expected, **arguments)

    def test_wrong_hosted_context_fails_before_suite_execution(self):
        """Publish a failed /2 wrapper without issuing a suite after an invalid requested head."""
        _, commit = self.commit_fixture()
        identity = self.identity_fixture(commit, [])
        versions = {"python": "3.11.0", "cargo": {}, "rustc": {}, "rust_host": "fixture"}
        with unittest.mock.patch.object(verifier, "capture_identity", return_value=identity), \
             unittest.mock.patch.object(verifier, "tool_versions", return_value=versions), \
             unittest.mock.patch.object(verifier, "contract_routes", return_value=[]), \
             unittest.mock.patch.object(verifier, "run_command") as command:
            receipt = verifier.verify(self.directory, self.directory / "fixture-binary", self.directory / "evidence",
                                      ("api-contract",), expected_commit=commit, event="push", checkout_kind="head",
                                      requested_head="f" * 40, build_outcome="success")
        command.assert_not_called()
        self.assertEqual(receipt["schema_version"], "forge.workspace-verification/2")
        self.assertEqual(receipt["status"], "failed")
        self.assertIsNone(receipt["checkout"])
        self.assertEqual(receipt["failure"], "verification-input-invalid")
        self.assertEqual(json.loads((self.directory / "evidence/workspace-verification.json").read_bytes()), receipt)


    def test_main_forwards_hosted_context_to_the_verified_capture(self):
        """Pass all parsed workflow fields into verification rather than silently granting local-only context."""
        arguments = ["verify_workspace.py", "--forge", str(self.directory / "forge"),
                     "--output-dir", str(self.directory / "evidence"), "--suite", "api-contract",
                     "--expected-commit", "c" * 40, "--event", "pull_request", "--checkout-kind", "pull-request-merge",
                     "--requested-head", "b" * 40, "--requested-base", "a" * 40, "--build-outcome", "success"]
        with unittest.mock.patch.object(sys, "argv", arguments), \
             unittest.mock.patch.object(verifier, "verify", return_value={"status": "passed"}) as verified, \
             unittest.mock.patch.object(sys, "stdout", new_callable=io.StringIO):
            self.assertEqual(verifier.main(), 0)
        self.assertEqual(verified.call_args.kwargs, {"event": "pull_request", "checkout_kind": "pull-request-merge",
                                                   "requested_head": "b" * 40, "requested_base": "a" * 40})
        self.assertEqual(verified.call_args.args[5:7], ("c" * 40, "success"))


if __name__ == "__main__":
    if not __debug__:
        raise SystemExit("Receipt tests require Python without optimization")
    unittest.main(verbosity=2)
