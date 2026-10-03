#!/usr/bin/env python3
"""Mocked maintained-client major negotiation and transport controls; no product evidence.

Root runs this new module with the reviewed candidate scripts directory exposed
externally. Ordinary import resolution is intentional; no alternate library or
TEMP-before fallback is embedded here. Every child and HTTP response is fake.
"""
import io
import json
import unittest
from collections import defaultdict, deque
from contextlib import ExitStack
from unittest import mock

import workspace_client as library

FORGE = "/private/tmp/client-version-fixture/forge"
PROJECT = "/private/tmp/client-version-fixture/project"
SESSION_ID = "sess_0123456789abcdef"
CAPABILITY = "c" * 64


def descriptor(major=2, read_only=True):
    """Produce fixed mock descriptor metadata without launching a listener or storing a real secret."""
    value = {"base_url": "http://127.0.0.1:34891", "api_version": f"{major}.0.0",
             "session_id": SESSION_ID, "capability": CAPABILITY,
             "mode": "machine", "read_only": read_only, "pid": 321}
    if major == 2:
        value["api_major"] = 2
    return value


def session(value):
    """Bind mock bootstrap fields to one descriptor without asserting server correctness."""
    return {"session_id": value["session_id"], "api_major": 2,
            "contract_version": value["api_version"], "mode": "machine",
            "read_only": value["read_only"], "project_label": "Mock fixture"}


class FakeResponse:
    """Bounded byte reader for an explicit synthetic HTTP response."""

    def __init__(self, value, status=200, *, raw=False):
        """Retain exact fixture bytes and the observed read ceiling."""
        self.status = status
        self.data = value if raw else json.dumps(value).encode()
        self.read_limits = []

    def read(self, limit):
        """Return at most the requested bytes, matching the client's response-bound seam."""
        self.read_limits.append(limit)
        return self.data[:limit]


class DescriptorPipe(io.BytesIO):
    """Record the real descriptor read bound while retaining only synthetic private bytes."""

    def __init__(self, raw):
        """Initialize one finite fixture stream and its read-limit observations."""
        super().__init__(raw)
        self.readline_limits = []

    def readline(self, limit=-1):
        """Honor and record the actual library descriptor ceiling without reading a product stream."""
        self.readline_limits.append(limit)
        return super().readline(limit)


class FakeProcess:
    """Original mocked Popen owner with explicit stream and wait/kill records."""

    def __init__(self, raw):
        """Provide only a private bounded descriptor stream; never create an OS child."""
        self.stdout = DescriptorPipe(raw)
        self.returncode = None
        self.wait_calls = []
        self.kill_calls = 0
        self.timeout_once = False

    def poll(self):
        """Return the mock owner's current terminal state."""
        return self.returncode

    def wait(self, timeout=None):
        """Record natural wait or one explicit mocked timeout on the original owner."""
        self.wait_calls.append(timeout)
        if self.timeout_once:
            self.timeout_once = False
            raise library.subprocess.TimeoutExpired("mock-owned-child", timeout)
        if self.returncode is None:
            self.returncode = 0
        return self.returncode

    def kill(self):
        """Record a fallback on this exact fake child without signalling a process."""
        self.kill_calls += 1
        self.returncode = -9


class FakeConnection:
    """HTTP adapter that records actual maintained-client method/path/body/header arguments."""

    def __init__(self, harness, host, port, timeout):
        """Bind each connection to the fake loopback endpoint and its response owner."""
        self.harness = harness
        self.address = (host, port, timeout)
        self.response = None
        self.closed = False

    def request(self, method, path, *, body=None, headers=None):
        """Observe the real library request arguments and select an explicit fixture response."""
        self.harness.requests.append({"method": method, "path": path,
                                      "body": body, "headers": dict(headers or {})})
        self.response = self.harness.response_for(method, path)

    def getresponse(self):
        """Return the response selected by this connection's actual requested route."""
        if self.response is None:
            raise AssertionError("A response requires an actual recorded request")
        return self.response

    def close(self):
        """Retain independent connection-close observation without an OS socket."""
        self.closed = True


class Harness:
    """Stateful fake process and network seams; the real Workspace methods run only under root tests."""

    def __init__(self, value=None, *, raw_descriptor=None, session_value=None):
        """Prepare explicit bootstrap data and empty transport records without importing an alternate client."""
        self.descriptor = descriptor() if value is None else value
        raw = (json.dumps(self.descriptor).encode() + b"\n") if raw_descriptor is None else raw_descriptor
        self.process = FakeProcess(raw)
        self.session_value = session(self.descriptor) if session_value is None else session_value
        self.spawn_arguments = []
        self.requests = []
        self.connections = []
        self.responses = defaultdict(deque)
        self.client = None
        self.stack = ExitStack()

    def spawn(self, args, **kwargs):
        """Capture the actual launch argv/options while returning the original fake child."""
        self.spawn_arguments.append((list(args), dict(kwargs)))
        return self.process

    def connection(self, host, port, timeout):
        """Install one recorded network adapter instead of opening a socket."""
        connection = FakeConnection(self, host, port, timeout)
        self.connections.append(connection)
        return connection

    def response_for(self, method, path):
        """Require explicit nonbootstrap fixtures and support only selected-session cleanup defaults."""
        queued = self.responses[(method, path)]
        if queued:
            return queued.popleft()
        if (method, path) == ("GET", "/api/v2/session"):
            return FakeResponse(self.session_value)
        if method == "POST" and path in ("/api/v1/session/shutdown", "/api/v2/session/shutdown"):
            return FakeResponse({"state": "shutting-down"})
        raise AssertionError("Unexpected mock route: " + method + " " + path)

    def enqueue(self, method, path, value, status=200, *, raw=False):
        """Associate supplied mock bytes with the exact real method/path consumer."""
        response = FakeResponse(value, status, raw=raw)
        self.responses[(method, path)].append(response)
        return response

    def __enter__(self):
        """Patch only process/network/clock seams; preserve the actual client implementation."""
        self.popen = self.stack.enter_context(mock.patch.object(library.subprocess, "Popen", side_effect=self.spawn))
        self.http = self.stack.enter_context(mock.patch.object(library.http.client, "HTTPConnection", side_effect=self.connection))
        self.clock = self.stack.enter_context(mock.patch.object(library.time, "monotonic", return_value=1.0))
        self.sleep = self.stack.enter_context(mock.patch.object(library.time, "sleep"))
        return self

    def launch(self, *, major=2, read_only=True, omit_major=False):
        """Construct the actual library client, retaining its returned object only on success."""
        if omit_major:
            self.client = library.Workspace(FORGE, PROJECT, read_only)
        else:
            self.client = library.Workspace(FORGE, PROJECT, read_only, api_major=major)
        return self.client

    def __exit__(self, exc_type, exc, traceback):
        """Complete mocked owner/stream cleanup before independently restoring every patch."""
        try:
            if self.client is not None:
                self.client.close()
        finally:
            self.process.stdout.close()
            self.stack.close()
        return False


class ClientVersionControls(unittest.TestCase):
    """Negotiation and transport controls with explicit mocked evidence boundaries."""

    def test_invalid_major_abstains_before_popen(self):
        """Reject booleans, unsupported majors and nonintegers before any child creation."""
        for major in (True, False, 0, 3, -1, 1.0, "2", None):
            with self.subTest(major=major), Harness() as h:
                with self.assertRaises(ValueError):
                    h.launch(major=major)
                h.popen.assert_not_called()
                h.http.assert_not_called()

    def test_default_v1_original_argv_and_no_session_read(self):
        """Preserve the legacy default launch argv and absence of a new bootstrap request."""
        value = descriptor(1)
        value["api_version"] = "1.2.0"
        with Harness(value) as h:
            client = h.launch(omit_major=True)
            args, options = h.spawn_arguments[0]
            self.assertEqual(args, [FORGE, "workspace", "--project", PROJECT, "--machine-session", "--read-only"])
            self.assertEqual(options, {"stdin": library.subprocess.DEVNULL, "stdout": library.subprocess.PIPE,
                                       "stderr": library.subprocess.DEVNULL})
            self.assertEqual(h.requests, [])
            self.assertEqual(client.api_path("/session"), "/api/v1/session")
            self.assertEqual(client._capability, CAPABILITY)

    def test_explicit_v1_writable_does_not_add_version_flag(self):
        """Keep explicit major1 and writable scope on the original argument shape."""
        with Harness(descriptor(1, False)) as h:
            h.launch(major=1, read_only=False)
            self.assertEqual(h.spawn_arguments[0][0], [FORGE, "workspace", "--project", PROJECT, "--machine-session"])
            self.assertEqual(h.requests, [])

    def test_matching_v2_bootstrap_and_explicit_argv(self):
        """Read the actual selected Session before the caller's first mock project request."""
        with Harness() as h:
            client = h.launch()
            self.assertEqual(h.spawn_arguments[0][0], [FORGE, "workspace", "--project", PROJECT,
                                                     "--machine-session", "--api-major", "2", "--read-only"])
            self.assertEqual([(r["method"], r["path"]) for r in h.requests], [("GET", "/api/v2/session")])
            h.enqueue("GET", "/api/v2/project/summary", {"marker": "mock-query"})
            self.assertEqual(client.request("GET", client.api_path("/project/summary")), {"marker": "mock-query"})
            self.assertEqual([(r["method"], r["path"]) for r in h.requests],
                             [("GET", "/api/v2/session"), ("GET", "/api/v2/project/summary")])
            self.assertTrue(all(c.address == ("127.0.0.1", 34891, 30) and c.closed for c in h.connections))

    def test_v2_writable_scope_bootstrap(self):
        """Bind an explicit writable v2 descriptor to the same Session scope."""
        with Harness(descriptor(2, False)) as h:
            h.launch(read_only=False)
            self.assertEqual(h.spawn_arguments[0][0][-2:], ["--api-major", "2"])
            self.assertEqual(h.requests[0]["path"], "/api/v2/session")
            self.assertEqual(h.requests[0]["headers"]["Authorization"], "Bearer " + CAPABILITY)

    def test_unknown_optional_descriptor_and_session_fields_tolerated(self):
        """Honor additive read compatibility while retaining all required negotiation fields."""
        value = descriptor()
        value["future_optional"] = {"marker": "mock"}
        selected = session(value)
        selected["future_optional"] = True
        with Harness(value, session_value=selected) as h:
            client = h.launch()
            self.assertEqual(client.api_path("/project/summary"), "/api/v2/project/summary")
            self.assertEqual(h.requests[0]["path"], "/api/v2/session")

    def test_v2_missing_required_descriptor_fields_fail_before_http(self):
        """Require each known v2 field and clean the original child on incomplete descriptor input."""
        for field in descriptor():
            value = descriptor()
            del value[field]
            with self.subTest(field=field), Harness(value, session_value={}) as h:
                with self.assertRaisesRegex(RuntimeError, "^The local workspace could not be started\\.$"):
                    h.launch()
                h.http.assert_not_called()
                self.assertEqual(h.process.wait_calls, [3])
                self.assertTrue(h.process.stdout.closed)

    def test_v2_descriptor_major_version_pid_and_session_id_rejected(self):
        """Reject malformed major/version/identity values before any Session or project transport."""
        changes = [("api_major", True), ("api_major", 1), ("api_major", "2"),
                   ("api_version", "1.2.0"), ("api_version", "2.0"), ("api_version", "2.x.0"),
                   ("api_version", "2." + "1" * 62 + ".0"),
                   ("pid", True), ("pid", 0), ("pid", -1),
                   ("session_id", "sess_short"), ("session_id", "foreign_0123456789"),
                   ("session_id", "sess_" + "a" * 65)]
        for field, invalid in changes:
            value = descriptor()
            value[field] = invalid
            with self.subTest(field=field, value=invalid), Harness(value, session_value={}) as h:
                with self.assertRaises(RuntimeError):
                    h.launch()
                h.http.assert_not_called()
                self.assertTrue(h.process.stdout.closed)

    def test_v2_descriptor_url_capability_mode_and_scope_rejected(self):
        """Retain original loopback, mode, scope and secret-format descriptor predicates."""
        changes = [("base_url", "https://127.0.0.1:34891"), ("base_url", "http://localhost:34891"),
                   ("base_url", "http://127.0.0.1:34891/private"),
                   ("base_url", "http://user@127.0.0.1:34891"), ("mode", "browser"),
                   ("read_only", False), ("read_only", 1), ("capability", "C" * 64),
                   ("capability", "c" * 63)]
        for field, invalid in changes:
            value = descriptor()
            value[field] = invalid
            with self.subTest(field=field), Harness(value, session_value={}) as h:
                with self.assertRaises(RuntimeError):
                    h.launch()
                h.http.assert_not_called()

    def test_descriptor_size_bound_precedes_bootstrap_http(self):
        """Reject an oversized descriptor through the real bounded pipe read, with owner cleanup."""
        raw = json.dumps({**descriptor(), "future_optional": "x" * 8192}).encode() + b"\n"
        with Harness(raw_descriptor=raw) as h:
            with self.assertRaises(RuntimeError):
                h.launch()
            h.http.assert_not_called()
            self.assertEqual(h.process.wait_calls, [3])
            self.assertEqual(h.process.stdout.readline_limits, [8193])
            self.assertTrue(h.process.stdout.closed)

    def test_malformed_descriptor_is_safe_and_precedes_http(self):
        """Reject malformed UTF-8/JSON and nonobject bootstrap data with fixed error and owner cleanup."""
        for raw in (b"{\n", b"\xff\n", b"[]\n", b"null\n", b"true\n", b"42\n"):
            with self.subTest(shape=raw[:2]), Harness(raw_descriptor=raw) as h:
                with self.assertRaisesRegex(RuntimeError, "could not be started"):
                    h.launch()
                h.http.assert_not_called()
                self.assertEqual(h.process.wait_calls, [3])
                self.assertEqual(h.process.stdout.readline_limits, [8193])
                self.assertTrue(h.process.stdout.closed)

    def test_v2_session_mismatch_never_reaches_project_route(self):
        """Check major, exact published version, ID, mode and scope on the actual bootstrap response."""
        changes = [("api_major", 1), ("api_major", True), ("contract_version", "2.1.0"),
                   ("session_id", "sess_aaaaaaaa"), ("mode", "browser"), ("read_only", False),
                   ("read_only", 1)]
        for field, invalid in changes:
            selected = session(descriptor())
            selected[field] = invalid
            with self.subTest(field=field), Harness(session_value=selected) as h:
                with self.assertRaises(RuntimeError):
                    h.launch()
                self.assertEqual(h.requests[0]["path"], "/api/v2/session")
                self.assertTrue(all(r["path"] in ("/api/v2/session", "/api/v2/session/shutdown") for r in h.requests))
                self.assertTrue(h.process.stdout.closed)

    def test_v2_session_shape_or_missing_fields_fail_closed(self):
        """Reject nonobject or missing bootstrap fields while retaining selected cleanup only."""
        values = [[], True, None]
        for field in ("api_major", "contract_version", "session_id", "mode", "read_only"):
            value = session(descriptor())
            del value[field]
            values.append(value)
        for value in values:
            with self.subTest(value=value), Harness(session_value=value) as h:
                if value is None:
                    h.session_value = None
                with self.assertRaises(RuntimeError):
                    h.launch()
                self.assertFalse(any("/project/" in r["path"] for r in h.requests))
                self.assertEqual(h.requests[0]["path"], "/api/v2/session")

    def test_minor_version_requires_matching_session_not_silent_rewrite(self):
        """Accept a bounded major2 numeric revision only when both actual bootstrap documents agree."""
        value = descriptor()
        value["api_version"] = "2.12.3"
        with Harness(value) as h:
            self.assertIsNotNone(h.launch())
            self.assertEqual(h.session_value["contract_version"], "2.12.3")
            self.assertEqual(h.requests[0]["path"], "/api/v2/session")

    def test_foreign_prefix_rejected_before_pacing_or_transport(self):
        """Reject both major directions, aliases and nonversioned paths before clock or HTTP use."""
        for major in (1, 2):
            with self.subTest(major=major), Harness(descriptor(major)) as h:
                client = h.launch(major=major)
                other = 2 if major == 1 else 1
                for path in (f"/api/v{other}/project/summary", f"/api/v{major}x/project/summary",
                             "/api/project/summary", "//api/v2/project/summary", f"/api/v{major}/session#secret",
                             f"/api/v{major}/session\n"):
                    previous = list(h.requests)
                    with self.subTest(path=path), mock.patch.object(library.time, "monotonic", side_effect=AssertionError("Pacing must not run")), mock.patch.object(library.http.client, "HTTPConnection", side_effect=AssertionError("Transport must not run")):
                        with self.assertRaises(ValueError):
                            client.request("GET", path)
                    self.assertEqual(h.requests, previous)

    def test_relative_api_path_guard_and_selected_prefix(self):
        """Build fixed relative routes while refusing another public API path or protocol-relative input."""
        with Harness() as h:
            client = h.launch()
            self.assertEqual(client.api_path("/operations/op_mock"), "/api/v2/operations/op_mock")
            for path in ("session", "/api/v1/session", "/api/v2/session", "//session", "/session#secret", "/session\n", None, True):
                with self.subTest(path=path), self.assertRaises(ValueError):
                    client.api_path(path)

    def test_bundle_helpers_use_selected_routes_and_preserve_supplied_value(self):
        """Observe real helper requests without treating a mock bundle payload as accepted domain evidence."""
        for major in (1, 2):
            with self.subTest(major=major), Harness(descriptor(major)) as h:
                client = h.launch(major=major)
                prefix = f"/api/v{major}"
                supplied = {"mock_input_marker": "preserved"}
                h.enqueue("GET", prefix + "/project/bundle-preview", {"mock_preview_marker": True})
                h.enqueue("POST", prefix + "/project/bundle-verifications", {"mock_comparison_marker": True})
                self.assertEqual(client.bundle_preview(), {"mock_preview_marker": True})
                self.assertEqual(client.verify_bundle(supplied), {"mock_comparison_marker": True})
                self.assertEqual([(r["method"], r["path"]) for r in h.requests[-2:]],
                                 [("GET", prefix + "/project/bundle-preview"), ("POST", prefix + "/project/bundle-verifications")])
                self.assertEqual(json.loads(h.requests[-1]["body"]), {"bundle": supplied})

    def test_wait_polls_same_selected_operation_id(self):
        """Read retained IDs on either selected major without submitting a replacement producer."""
        for major in (1, 2):
            with self.subTest(major=major), Harness(descriptor(major)) as h:
                client = h.launch(major=major)
                prefix = f"/api/v{major}"
                completed = {"operation_id": "op_mock", "state": "succeeded"}
                h.enqueue("GET", prefix + "/operations/op_mock", completed)
                self.assertEqual(client.wait({"operation_id": "op_mock", "state": "running"}), completed)
                self.assertEqual(h.requests[-1]["method"], "GET")
                self.assertEqual(h.requests[-1]["path"], prefix + "/operations/op_mock")
                self.assertFalse(any(r["method"] == "POST" and r["path"] != prefix + "/session/shutdown" for r in h.requests))

    def test_wait_timeout_abstains_from_operation_poll(self):
        """Preserve the existing inclusive timeout before a further operation request."""
        with Harness() as h:
            client = h.launch()
            before = list(h.requests)
            with self.assertRaises(TimeoutError):
                client.wait({"operation_id": "op_mock", "state": "pending"}, timeout=0)
            self.assertEqual(h.requests, before)

    def test_commit_routes_exact_confirmation_preview_and_key(self):
        """Carry each selected session's exact preview token/version/key through actual transport."""
        for major in (1, 2):
            with self.subTest(major=major), Harness(descriptor(major)) as h:
                client = h.launch(major=major)
                prefix = f"/api/v{major}"
                preview = {"receipt": {"token": "mock-private-preview"}, "target_version": "mock-version"}
                h.enqueue("POST", prefix + "/effects/commits", {"mock_commit_marker": True})
                self.assertEqual(client.commit(preview, confirmed=True, idempotency_key="mock-key"), {"mock_commit_marker": True})
                request = h.requests[-1]
                self.assertEqual(request["path"], prefix + "/effects/commits")
                self.assertEqual(request["headers"]["Idempotency-Key"], "mock-key")
                self.assertEqual(json.loads(request["body"]), {"receipt": "mock-private-preview", "observed_version": "mock-version", "confirmed": True})
                before = list(h.requests)
                for confirmed in (False, None, 1, "true"):
                    with self.subTest(confirmed=confirmed), self.assertRaises(ValueError):
                        client.commit(preview, confirmed=confirmed, idempotency_key="mock-key")
                    self.assertEqual(h.requests, before)

    def test_close_selected_shutdown_and_original_owner_cleanup(self):
        """Use each selected shutdown, clear capability and wait/close the exact fake owner."""
        for major in (1, 2):
            with self.subTest(major=major), Harness(descriptor(major)) as h:
                client = h.launch(major=major)
                prefix = f"/api/v{major}"
                client.close()
                self.assertEqual(h.requests[-1]["path"], prefix + "/session/shutdown")
                self.assertEqual(h.requests[-1]["method"], "POST")
                self.assertEqual(client._capability, "")
                self.assertEqual(h.process.wait_calls, [3])
                self.assertEqual(h.process.kill_calls, 0)
                self.assertTrue(h.process.stdout.closed)
                before = list(h.requests)
                client.close()
                self.assertEqual(h.requests, before)

    def test_close_retains_original_wait_timeout_fallback(self):
        """Exercise the old wait/kill/wait fallback on a mocked original owner without native cleanup credit."""
        with Harness() as h:
            client = h.launch()
            h.process.timeout_once = True
            client.close()
            self.assertEqual(h.process.wait_calls, [3, None])
            self.assertEqual(h.process.kill_calls, 1)
            self.assertTrue(h.process.stdout.closed)
            self.assertEqual(client._capability, "")

    def test_raw_response_ceiling_and_connection_cleanup(self):
        """Preserve actual raw bytes, the MAX_RESPONSE+1 read fence and independent HTTP close."""
        with Harness() as h:
            client = h.launch()
            body = b"fixed mock bytes\x00\xff"
            response = h.enqueue("GET", "/api/v2/exports/export_mock/download", body, raw=True)
            self.assertEqual(client.request("GET", "/api/v2/exports/export_mock/download", raw=True), body)
            self.assertEqual(response.read_limits, [library.MAX_RESPONSE + 1])
            self.assertTrue(h.connections[-1].closed)
            oversized = h.enqueue("GET", "/api/v2/project/summary", b"x" * (library.MAX_RESPONSE + 1), raw=True)
            with self.assertRaisesRegex(RuntimeError, "Response exceeds"):
                client.request("GET", "/api/v2/project/summary", raw=True)
            self.assertEqual(oversized.read_limits, [library.MAX_RESPONSE + 1])
            self.assertTrue(h.connections[-1].closed)

    def test_typed_error_payload_and_http_close_preserved(self):
        """Retain the server's exact typed error payload without converting failure into success."""
        with Harness() as h:
            client = h.launch()
            value = {"code": "not-found", "message": "Fixed mock missing resource", "retryable": False}
            h.enqueue("GET", "/api/v2/resources/res_mock", value, status=404)
            with self.assertRaises(library.WorkspaceError) as caught:
                client.request("GET", "/api/v2/resources/res_mock")
            self.assertEqual(caught.exception.payload, value)
            self.assertEqual(h.requests[-1]["path"], "/api/v2/resources/res_mock")
            self.assertTrue(h.connections[-1].closed)

    def test_json_request_bound_and_nonfinite_values_precede_http(self):
        """Retain original encoded-body and finite-JSON admission before network construction."""
        with Harness() as h:
            client = h.launch()
            before = list(h.requests)
            for body in ({"content": "x" * (14 * 1024 * 1024)}, {"value": float("nan")}):
                with self.subTest(kind="large" if "content" in body else "nonfinite"), mock.patch.object(library.http.client, "HTTPConnection", side_effect=AssertionError("No transport for rejected body")):
                    with self.assertRaises(ValueError):
                        client.request("POST", "/api/v2/resources/upload", body)
                self.assertEqual(h.requests, before)


if __name__ == "__main__":
    unittest.main()
