#!/usr/bin/env python3
"""Synthetic stdlib tests for F04 receipt boundaries; never run Cargo/Forge."""
import copy
import json
import io
from pathlib import Path
import sys
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
]


def client_receipt(declared):
    observed = sorted(declared)[:2]
    return {
        "schema_version": "forge.workspace-client-verification/1", "status": "passed",
        "checks": CHECKS, "check_count": len(CHECKS), "declared_operation_count": len(declared),
        "observed_operations": observed, "unobserved_operations": sorted(set(declared) - set(observed)),
        "operation_outcomes": {name: {"succeeded": 1, "rejected": 0, "transport_failed": 0}
                               for name in observed},
    }


class ReceiptTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="forge-receipt-test-")
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.declared = [name for _, _, name in verifier.contract_routes(ROOT)]

    def write_client(self, value):
        path = self.directory / "client.json"
        path.write_bytes(verifier.canonical_bytes(value))
        return path

    def fixture_run(self, *, selected=verifier.SUITES, result=None, dirty=False,
                    drift=False, tool_missing=False, build="success", expected_commit=None):
        identity = {"source_commit": "a" * 40, "tracked_source_clean": not dirty,
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
            calls.append(arguments)
            if "--receipt" in arguments:
                if result is None or result.get("exit_code") == 0:
                    path = Path(arguments[arguments.index("--receipt") + 1])
                    path.write_bytes(verifier.canonical_bytes(client_receipt(self.declared)))
            return result or {"exit_code": 0, "failure": None, "output": SUMMARY}
        output = self.directory / "evidence"
        with unittest.mock.patch.object(verifier, "capture_identity", side_effect=[identity, after]), \
             unittest.mock.patch.object(verifier, "tool_versions", return_value=versions), \
             unittest.mock.patch.object(verifier, "run_command", side_effect=command):
            receipt = verifier.verify(ROOT, self.directory / "synthetic-binary", output,
                                      selected, expected_commit=expected_commit, build_outcome=build)
        return receipt, calls, output

    def test_canonical_publication_refuses_overwrite(self):
        path = self.directory / "receipt.json"
        verifier.atomic_receipt(path, {"z": 2, "a": "opaque"})
        self.assertEqual(path.read_bytes(), b'{"a":"opaque","z":2}\n')
        with self.assertRaises(FileExistsError):
            verifier.atomic_receipt(path, {"different": True})
        self.assertEqual(path.read_bytes(), b'{"a":"opaque","z":2}\n')
        self.assertEqual(sorted(item.name for item in self.directory.iterdir()), ["receipt.json"])

    def test_hash_preserves_binary_crlf_and_rejects_symlink(self):
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
        routes = verifier.contract_routes(ROOT)
        self.assertEqual(len(routes), 37)
        self.assertEqual([(verb, name) for verb, pattern, name in routes
                          if pattern.fullmatch("/api/v1/operations/op_opaque")], [("GET", "getOperation")])
        self.assertFalse(any(pattern.fullmatch("/api/v1/operations/op_opaque/extra")
                             for _, pattern, _ in routes))

    def test_duplicate_contract_operation_rejected(self):
        root = self.directory / "repo"
        path = root / "docs/api/forge-workspace-v1.openapi.yaml"
        path.parent.mkdir(parents=True)
        path.write_text("paths:\n  /api/v1/first:\n    get:\n      operationId: same\n  /api/v1/second:\n    post:\n      operationId: same\n")
        with self.assertRaises(ValueError):
            verifier.contract_routes(root)

    def test_operation_names_with_underscore_and_hyphen_preserve_full_inventory(self):
        root = self.directory / "repo"
        path = root / "docs/api/forge-workspace-v1.openapi.yaml"
        path.parent.mkdir(parents=True)
        path.write_text("paths:\n  /api/v1/first:\n    get:\n      operationId: new_operation\n  /api/v1/second:\n    head:\n      operationId: new-operation\ncomponents:\n")
        self.assertEqual([name for _, _, name in verifier.contract_routes(root)], ["new_operation", "new-operation"])

    def test_unsupported_contract_lines_fail_instead_of_reducing_denominator(self):
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
        path = self.directory / "client.json"
        for raw in (b'{"status":"passed","status":"failed"}', b'{"status":NaN}', b'[]'):
            path.write_bytes(raw)
            with self.subTest(raw=raw), self.assertRaises(ValueError):
                verifier.read_client_receipt(path, self.declared)

    def test_all_suites_pass_only_with_clean_stable_inputs(self):
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
        receipt, _, _ = self.fixture_run(selected=("api-contract",), result={
            "exit_code": 0, "failure": None, "output": SUMMARY.replace(b"0 measured", b"1 measured")})
        self.assertEqual(receipt["status"], "incomplete")
        self.assertEqual(receipt["suites"]["api-contract"]["status"], "incomplete")
        self.assertEqual(receipt["suites"]["api-contract"]["counts"]["measured"], 1)

    def test_unselected_suite_is_incomplete(self):
        receipt, calls, _ = self.fixture_run(selected=("api-contract",))
        self.assertEqual(receipt["status"], "incomplete")
        self.assertEqual(len(calls), 1)
        self.assertEqual(receipt["suites"]["maintained-client"]["failure"], "not-selected")

    def test_failed_suite_does_not_suppress_remaining_suites_or_leak_output(self):
        receipt, calls, output = self.fixture_run(result={"exit_code": 9, "failure": None,
                                                       "output": b"/private/absolute/path SECRET CAPABILITY PRIVATE PROSE"})
        self.assertEqual(receipt["status"], "failed")
        self.assertEqual(len(calls), 3)
        self.assertTrue(all(row["status"] == "failed" for row in receipt["suites"].values()))
        raw = (output / "workspace-verification.json").read_bytes()
        for forbidden in (b"/private", b"SECRET", b"CAPABILITY", b"PRIVATE PROSE"):
            self.assertNotIn(forbidden, raw)

    def test_zero_exit_without_summary_is_not_success(self):
        receipt, _, _ = self.fixture_run(selected=("api-contract",),
                                        result={"exit_code": 0, "failure": None, "output": b"looks green"})
        self.assertEqual(receipt["suites"]["api-contract"]["failure"], "invalid-suite-receipt")
        self.assertEqual(receipt["status"], "failed")

    def test_dirty_source_missing_tool_or_unqualified_build_is_incomplete(self):
        for options in ({"dirty": True}, {"tool_missing": True}, {"build": "failure"}, {"build": "unrecorded"}):
            with self.subTest(options=options):
                self.temporary.cleanup()
                self.setUp()
                receipt, _, _ = self.fixture_run(**options)
                self.assertEqual(receipt["status"], "incomplete")
                self.assertTrue(all(row["status"] == "passed" for row in receipt["suites"].values()))

    def test_source_drift_fails_after_suites(self):
        receipt, calls, _ = self.fixture_run(drift=True)
        self.assertEqual(receipt["status"], "failed")
        self.assertEqual(receipt["input_stability"], "changed")
        self.assertEqual(len(calls), 3)

    def test_wrong_commit_fails_before_executing_suites(self):
        receipt, calls, _ = self.fixture_run(expected_commit="f" * 40)
        self.assertEqual(receipt["status"], "failed")
        self.assertEqual(calls, [])
        self.assertTrue(all(row["status"] == "incomplete" for row in receipt["suites"].values()))

    def test_existing_output_directory_is_preserved(self):
        output = self.directory / "evidence"
        output.mkdir()
        previous = output / "historical.json"
        previous.write_bytes(b"immutable historical evidence\n")
        with self.assertRaises(ValueError):
            verifier.verify(ROOT, self.directory / "binary", output, verifier.SUITES)
        self.assertEqual(previous.read_bytes(), b"immutable historical evidence\n")

    def test_process_capture_is_bounded_and_timeout_is_explicit(self):
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
        with unittest.mock.patch.object(verifier, "run_command", return_value={"exit_code": 0, "failure": None,
                                                                     "output": b"/private/path SECRET CAPABILITY"}):
            versions = verifier.tool_versions(ROOT, 1)
        self.assertIsNone(versions["cargo"])
        self.assertIsNone(versions["rustc"])
        self.assertIsNone(versions["rust_host"])
        self.assertNotIn("SECRET", json.dumps(versions))

    def test_recording_client_retains_only_documented_identifiers(self):
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
        path = self.directory / "failure.json"
        with unittest.mock.patch.object(sys, "argv", ["test_workspace_client.py", "--forge", "/private/binary", "--receipt", str(path)]), \
             unittest.mock.patch.object(client_test, "run", side_effect=AssertionError("SECRET CAPABILITY PRIVATE PROSE")):
            self.assertEqual(client_test.main(), 1)
        value = json.loads(path.read_bytes())
        self.assertEqual(value["status"], "failed")
        self.assertEqual(value["check_count"], 0)
        self.assertNotIn(b"SECRET", path.read_bytes())
        self.assertNotIn(b"/private", path.read_bytes())


if __name__ == "__main__":
    if not __debug__:
        raise SystemExit("Receipt tests require Python without optimization")
    unittest.main(verbosity=2)
