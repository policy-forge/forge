#!/usr/bin/env python3
"""Standard-library wrapper/negotiation controls; no live server or runtime acceptance."""
import io
import json
import unittest
from unittest.mock import patch
from urllib.parse import parse_qs, urlsplit

from workspace_client import Workspace


class RecordedWorkspace(Workspace):
    """Exercise public wrappers through a controlled recorder instead of an HTTP connection."""
    def __init__(self, major=2, version="2.1.0"):
        """Declare the fixture's negotiated selection without launching a process."""
        self._api_major = major
        self._api_prefix = "/api/v" + str(major)
        self._contract_version = version
        self.calls = []

    def request(self, method, path, body=None, **keywords):
        """Record exact wrapper transport arguments without generating operation accounting."""
        self.calls.append((method, path, body, keywords))
        return {"synthetic": True}


class DescriptorProcess:
    """Provide only an owned bootstrap line for constructor negotiation controls."""
    def __init__(self, descriptor):
        """Retain synthetic descriptor bytes in memory, never a real capability or file."""
        self.stdout = io.BytesIO((json.dumps(descriptor) + "\n").encode())
        self.returncode = None

    def poll(self):
        """Expose the fake process lifecycle expected by unchanged close cleanup."""
        return self.returncode

    def wait(self, timeout=None):
        """Mark this in-memory process closed, without waiting on a subprocess."""
        self.returncode = 0
        return 0

    def kill(self):
        """Mark forced cleanup for this fake process only."""
        self.returncode = -9


class LifecycleClientTests(unittest.TestCase):
    """Check exact query/namespace controls; mock outcomes are not domain/HTTP evidence."""
    def calls(self, client):
        """Return nine distinct public wrapper invocations with meaningful explicit query context."""
        return [
            lambda: client.lifecycle_records(as_of="2026-10-02", owner="owner & key", state="approved", cursor="opaque+page"),
            lambda: client.lifecycle_record("res_record000001", as_of="2026-10-02"),
            lambda: client.lifecycle_history("res_record000001", page_size=200),
            lambda: client.lifecycle_queue(as_of="2026-10-02", owner="owner & key"),
            lambda: client.framework_impact_comparisons(),
            lambda: client.framework_impact_comparison("res_compare000001"),
            lambda: client.framework_impact_changes("res_compare000001", change_class="content-changed"),
            lambda: client.framework_impact_findings("res_compare000001", group="g & 1", decision_state="under-review", policy_source="source?1", priority="blocking", owner="owner & key"),
            lambda: client.framework_impact_prior_dispositions("res_compare000001"),
        ]

    def descriptor(self, version):
        """Build the eight-field API2 bootstrap fixture with a deliberately synthetic capability."""
        return {"base_url":"http://127.0.0.1:32123", "api_version":version, "api_major":2,
                "session_id":"sess_client001", "capability":"a" * 64, "mode":"machine", "read_only":True, "pid":1}

    def test_nine_wrappers_preserve_exact_get_paths_and_all_five_and_filters(self):
        """Each wrapper delegates one GET with encoded filters and no body or idempotency key."""
        client = RecordedWorkspace()
        for call in self.calls(client):
            self.assertEqual(call(), {"synthetic": True})
        expected = ["/lifecycle/records", "/lifecycle/records/res_record000001", "/lifecycle/records/res_record000001/history",
                    "/lifecycle/queue", "/framework-impact/comparisons", "/framework-impact/comparisons/res_compare000001",
                    "/framework-impact/comparisons/res_compare000001/changes", "/framework-impact/comparisons/res_compare000001/findings",
                    "/framework-impact/comparisons/res_compare000001/prior-dispositions"]
        self.assertEqual(len(client.calls), 9)
        for (method, path, body, keywords), relative in zip(client.calls, expected):
            self.assertEqual(method, "GET")
            self.assertEqual(urlsplit(path).path, "/api/v2" + relative)
            self.assertIsNone(body)
            self.assertEqual(keywords, {})
        self.assertEqual(parse_qs(urlsplit(client.calls[0][1]).query),
                         {"as_of":["2026-10-02"], "owner":["owner & key"], "state":["approved"], "page_size":["50"], "cursor":["opaque+page"]})
        self.assertEqual(parse_qs(urlsplit(client.calls[7][1]).query),
                         {"group":["g & 1"], "decision_state":["under-review"], "policy_source":["source?1"], "priority":["blocking"], "owner":["owner & key"], "page_size":["50"]})

    def test_api1_and_prior_api2_contract_fail_before_every_s3_transport(self):
        """Both old negotiated selections retain their original methods without orphan S3 calls."""
        for major, version in [(1, "1.2.0"), (2, "2.0.0")]:
            client = RecordedWorkspace(major, version)
            for call in self.calls(client):
                with self.subTest(major=major, version=version), self.assertRaises(ValueError):
                    call()
            self.assertEqual(client.calls, [])
            client.bundle_preview()
            self.assertEqual(client.calls[0][1], f"/api/v{major}/project/bundle-preview")

    def test_required_date_and_malformed_date_fail_without_clock_or_transport(self):
        """Detail and owner queue require explicit date inputs; malformed dates cannot be sent."""
        client = RecordedWorkspace()
        for date_value in [None, "", "2026-2-02", "2026-02-30", "2026-10-02T00:00:00Z"]:
            for call in [lambda: client.lifecycle_record("res_record000001", as_of=date_value),
                         lambda: client.lifecycle_queue(as_of=date_value)]:
                with self.subTest(date=date_value), self.assertRaises(ValueError):
                    call()
        self.assertEqual(client.calls, [])
        client.lifecycle_records()
        self.assertNotIn("as_of", parse_qs(urlsplit(client.calls[-1][1]).query))

    def test_query_page_and_cursor_bounds_reject_bool_and_empty_strings(self):
        """Zero, booleans, over-limit pages and empty/over-limit filters cannot become encoded reads."""
        client = RecordedWorkspace()
        for invalid in [0, 201, True, 50.0]:
            with self.subTest(page=invalid), self.assertRaises(ValueError):
                client.framework_impact_comparisons(page_size=invalid)
        for cursor in ["", "x" * 257]:
            with self.subTest(cursor=len(cursor)), self.assertRaises(ValueError):
                client.lifecycle_history("res_record000001", cursor=cursor)
        for owner in ["", "x" * 65537]:
            with self.subTest(owner=len(owner)), self.assertRaises(ValueError):
                client.lifecycle_records(owner=owner)
        self.assertEqual(client.calls, [])
        client.lifecycle_history("res_record000001", cursor="x" * 256, page_size=200)
        self.assertEqual(len(client.calls), 1)

    def test_selected_ids_cannot_rewrite_the_route_or_join_finding_tokens(self):
        """Only exact registered IDs enter path segments; finding IDs cannot select a comparison."""
        client = RecordedWorkspace()
        for resource_id in ["", "../res_record000001", "res_record000001/changes", "finding-shared", "res_UPPERCASE"]:
            with self.subTest(resource=resource_id), self.assertRaises(ValueError):
                client.framework_impact_comparison(resource_id)
        self.assertEqual(client.calls, [])

    def test_numeric_api2_versions_keep_bootstrap_compatibility_and_gate_new_methods(self):
        """Retain numeric API2 bootstrap; inspection admits exactly 2.1.0, 2.2.0 and 2.3.0."""
        for version in ["2.0.0", "2.1.0", "2.0.1", "2.2.0", "2.3.0", "2.12.3"]:
            process = DescriptorProcess(self.descriptor(version))
            session = {"api_major":2, "contract_version":version, "session_id":"sess_client001", "mode":"machine", "read_only":True}
            with self.subTest(version=version), patch("workspace_client.subprocess.Popen", return_value=process), patch.object(Workspace, "request", return_value=session):
                client = Workspace("forge", ".", api_major=2)
                self.assertEqual(client._contract_version, version)
                if version in ("2.1.0", "2.2.0", "2.3.0"):
                    self.assertEqual(client.lifecycle_records(), session)
                else:
                    for call in self.calls(client):
                        with self.assertRaises(ValueError):
                            call()
                client.close()
                self.assertIsNotNone(process.returncode)

    def test_session_version_mismatch_cannot_enable_the_new_methods(self):
        """A 2.1 descriptor paired with 2.0 Session is rejected and the fake process is closed."""
        process = DescriptorProcess(self.descriptor("2.1.0"))
        foreign = {"api_major":2, "contract_version":"2.0.0", "session_id":"sess_client001", "mode":"machine", "read_only":True}
        with patch("workspace_client.subprocess.Popen", return_value=process), patch.object(Workspace, "request", return_value=foreign):
            with self.assertRaisesRegex(RuntimeError, "could not be started"):
                Workspace("forge", ".", api_major=2)
        self.assertIsNotNone(process.returncode)


if __name__ == "__main__":
    unittest.main()
