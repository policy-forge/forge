#!/usr/bin/env python3
"""Mocked functional-extension controls through the real maintained client and request recorder.

These controls replace the library's HTTPConnection transport and clock only;
they launch no Forge process and exercise no browser, privileged tool or network.
"""
import copy
import contextlib
import hashlib
import io
import json
from pathlib import Path
import stat
import tempfile
import time
import types
import unittest
from unittest import mock
from urllib.parse import parse_qs, urlsplit

import test_workspace_client as extension
import workspace_client as client_library


class SyntheticResponse:
    """Expose the exact status and bounded JSON byte interface used by Workspace.request."""

    def __init__(self, status, payload):
        """Serialize a synthetic response without invoking the product or HTTP stack."""
        self.status = status
        self.payload = json.dumps(payload, sort_keys=True, allow_nan=False).encode()

    def read(self, bound):
        """Honor the real client's maximum-read request instead of bypassing its parser."""
        return self.payload[:bound]


class SyntheticConnection:
    """Provide one private synthetic HTTP adapter with independently observed request bytes."""

    def __init__(self, transport, host, port, timeout):
        """Bind the requested loopback connection without opening any socket."""
        self.transport = transport
        self.host, self.port, self.timeout = host, port, timeout
        self.response = None
        self.closed = False

    def request(self, method, path, *, body, headers):
        """Decode real serialized request bytes and invoke the stateful synthetic server."""
        decoded = None if body is None else json.loads(body)
        event = {"method": method, "path": path, "body": decoded, "headers": dict(headers)}
        self.transport.events.append(event)
        status, payload = self.transport.handler(event)
        self.response = SyntheticResponse(status, payload)

    def getresponse(self):
        """Return the response actually prepared by this adapter's observed request."""
        if self.response is None:
            raise RuntimeError("Synthetic response was not prepared")
        return self.response

    def close(self):
        """Retain exact connection closure for library cleanup assertions."""
        self.closed = True


class SyntheticTransport:
    """Retain private synthetic request events while leaving operation classification to the recorder."""

    def __init__(self, handler=None, script=None):
        """Accept a stateful server callback or an exact sequential route/response script."""
        if (handler is None) == (script is None):
            raise ValueError("Choose one synthetic response adapter")
        self.events, self.connections = [], []
        self.script = None if script is None else list(script)
        self.handler = handler if handler is not None else self.scripted

    def scripted(self, event):
        """Refuse unexpected methods, routes or requests after the declared response sequence."""
        if not self.script:
            raise AssertionError("Unexpected synthetic request")
        method, path, status, payload = self.script.pop(0)
        if (event["method"], event["path"]) != (method, path):
            raise AssertionError("Synthetic request did not match the declared route")
        if isinstance(payload, BaseException):
            raise payload
        return status, copy.deepcopy(payload)

    def connection(self, host, port, timeout):
        """Supply the library's only replaced network constructor with an owned fake connection."""
        result = SyntheticConnection(self, host, port, timeout)
        self.connections.append(result)
        return result


class FunctionalFixture:
    """Model independent synthetic server state and private file bytes for helper-semantic controls only."""

    def __init__(self, root, read_only=False):
        """Create seven flat private files and distinct stable resource identities without running domain code."""
        self.root, self.read_only = root, read_only
        self.files = {
            "policy.md": "# Synthetic policy\n\n- Review caf\u00e9 clauses.\n".encode(),
            "catalog.json": b'{"synthetic":"catalog"}\n',
            "framework.json": b'{"synthetic":"two controls"}\n',
            "mapping.json": b'{"mapping":{"maps":[{"key":"reviewed-none"}]}}\n',
            "mapping-collection.json": b'{"synthetic":"built mapping"}\n',
            "scope.json": b'{"synthetic":"scope"}\n',
            "applicability-report.json": b'{"synthetic":"report"}\n',
        }
        for name, raw in self.files.items():
            (root / name).write_bytes(raw)
        definitions = [
            ("built-mapping", "mapping-collection.json", "mapping-collection"),
            ("catalog", "catalog.json", "oscal-catalog-artifact"),
            ("framework", "framework.json", "oscal-catalog-artifact"),
            ("mapping", "mapping.json", "mapping-collection"),
            ("policy", "policy.md", "policy-source"),
            ("scope", "scope.json", "applicability-manifest"),
            ("scope-report", "applicability-report.json", "applicability-report"),
        ]
        self.resources = []
        for number, (key, path, role) in enumerate(definitions, 1):
            raw = self.files[path]
            digest = hashlib.sha256(raw).hexdigest()
            self.resources.append({"resource_id": "res_" + format(number, "032x"), "key": key,
                                   "path": path, "role": role, "sha256": digest, "version": digest,
                                   "size_bytes": len(raw), "validation_state": "valid", "stale": False})
        self.by_id = {row["resource_id"]: row for row in self.resources}
        index = {"schema_version": "forge.workspace/1", "label": "Synthetic fixture",
                 "resources": [{key: row[key] for key in ("key", "role", "path")} for row in self.resources]}
        self.files["forge.workspace.json"] = (json.dumps(index, sort_keys=True, indent=2) + "\n").encode()
        (root / "forge.workspace.json").write_bytes(self.files["forge.workspace.json"])
        self.policy = next(row for row in self.resources if row["key"] == "policy")
        self.mapping = {"manifest": json.loads(self.files["mapping.json"]),
                        "version": hashlib.sha256(self.files["mapping.json"]).hexdigest()}
        classes = ["applicable-mapped", "applicable-reviewed-no-relationship", "applicable-unmapped",
                   "not-applicable", "deferred", "under-review"]
        counts = [{"classification": name, "count": 1 if name in
                   {"applicable-reviewed-no-relationship", "under-review"} else 0} for name in classes]
        self.analysis = {"eligible_controls": 2, "classification_counts": counts}
        self.view = {"version": hashlib.sha256(self.files["applicability-report.json"]).hexdigest(),
                     "stale": False, "eligible_controls": 2, "classification_counts": copy.deepcopy(counts),
                     "input_fingerprints": [{"resource_id": row["resource_id"], "sha256": row["sha256"], "matches_current": True}
                                            for key in ("scope", "framework", "built-mapping")
                                            for row in self.resources if row["key"] == key]}
        self.queue = [{"item_id": "qi_" + "a" * 32, "reason_code": "no-reviewed-mapping",
                       "summary": "Synthetic omitted review", "resource_id": self.policy["resource_id"]},
                      {"item_id": "qi_" + "b" * 32, "reason_code": "reviewed-no-positive-relationship",
                       "summary": "Synthetic explicit review", "resource_id": self.policy["resource_id"]}]
        self.counts = {"total_open": 2, "by_reason": [{"reason_code": item["reason_code"], "count": 1}
                                                      for item in self.queue]}
        self.excerpt = {"excerpt_id": "ex_" + "a" * 32, "resource_id": self.policy["resource_id"],
                        "sha256": self.policy["sha256"], "start_line": 1, "end_line": 3,
                        "text": self.files["policy.md"].decode(), "truncated": False}
        self.entries = [{"entry_id": "prov_" + "a" * 32, "kind": "source-reference",
                         "label": "Synthetic source", "refs": [], "fingerprint": self.policy["sha256"],
                         "excerpt_refs": [self.excerpt["excerpt_id"]]},
                        {"entry_id": "prov_" + "b" * 32, "kind": "policy-subject",
                         "label": "Synthetic subject", "refs": [{"kind": "source-reference",
                         "id": "prov_" + "a" * 32}], "fingerprint": self.policy["sha256"]}]
        self.fault = None

    def report(self, resource=None, invalid=False):
        """Return an independent closed report whose error count and resource pointer reconcile exactly."""
        return {"state": "invalid" if invalid else "valid", "error_count": int(invalid), "warning_count": 0,
                "diagnostics": [{"code": "invalid-resource", "severity": "error", "message": "Synthetic invalid input",
                                 "resource_id": resource}] if invalid else []}

    def page(self, rows, query):
        """Serve one item per actual cursor request and require the caller's fixed paging size."""
        if query.get("page_size") != ["1"]:
            raise AssertionError("Synthetic paging size was not fixed to one")
        cursor = query.get("cursor", [None])[0]
        number = 0 if cursor is None else int(cursor.removeprefix("cursor-"))
        if number < 0 or number >= max(1, len(rows)):
            raise AssertionError("Synthetic cursor did not identify an existing page")
        return {"resource_version": "f" * 64, "page": {"items": copy.deepcopy(rows[number:number + 1]),
                "next_cursor": "cursor-" + str(number + 1) if number + 1 < len(rows) else None,
                "total_matching": len(rows)}}

    def response(self, event):
        """Answer observed documented requests with stateful private fixture facts and exact typed negatives."""
        url = urlsplit(event["path"])
        path, query, body, method = url.path, parse_qs(url.query), event["body"], event["method"]
        if method == "GET" and path == "/api/v1/session":
            result = {"session_id": "sess_" + "a" * 16, "mode": "machine", "read_only": self.read_only,
                      "api_major": 1, "contract_version": "1.2.0", "project_label": "Synthetic fixture"}
        elif method == "GET" and path == "/api/v1/project/config-status":
            result = {"present": False, "valid": False, "issues": []}
        elif method == "GET" and path == "/api/v1/resources":
            result = self.page(self.resources, query)
        elif method == "GET" and path.startswith("/api/v1/resources/"):
            resource = path.split("/")[4]
            if resource not in self.by_id:
                return 404, {"code": "not-found", "message": "Synthetic unknown resource"}
            result = self.report(resource) if path.endswith("/validation") else self.by_id[resource]
        elif method == "POST" and path == "/api/v1/validation/runs":
            if body == {"scope": "selected", "resource_ids": []}:
                return 400, {"code": "invalid-request", "message": "Synthetic empty selection"}
            if body["scope"] == "selected":
                if body["resource_ids"] != [self.policy["resource_id"]]:
                    raise AssertionError("Synthetic selection did not bind the policy identity")
                result = self.report(self.policy["resource_id"])
            elif body == {"scope": "all"}:
                result = self.report()
            else:
                raise AssertionError("Synthetic validation scope was not declared")
        elif method == "POST" and path == "/api/v1/mapping/checks":
            if body != {}:
                raise AssertionError("Synthetic mapping check expected the fixed empty body")
            result = self.report()
        elif method == "GET" and path == "/api/v1/mapping/draft":
            result = self.mapping
        elif method == "POST" and path == "/api/v1/mapping/draft/validation":
            if set(body) != {"manifest"}:
                raise AssertionError("Synthetic draft validation had an open request shape")
            maps = body["manifest"]["mapping"]["maps"]
            result = self.report(invalid=len({row["key"] for row in maps}) != len(maps))
        elif method == "GET" and path == "/api/v1/applicability/report":
            result = self.view
        elif method == "GET" and path == "/api/v1/review-queue/counts":
            result = self.counts
        elif method == "GET" and path == "/api/v1/review-queue/items":
            rows = [row for row in self.queue if "reason_code" not in query or row["reason_code"] == query["reason_code"][0]]
            result = self.page(rows, query)
        elif method == "GET" and path == "/api/v1/project/summary":
            result = {"review_counts": {"total_open": len(self.queue)}, "resource_counts": {"total": len(self.resources)}}
        elif method == "GET" and path == "/api/v1/provenance/entries":
            if query.get("anchor") != [self.policy["resource_id"]]:
                raise AssertionError("Synthetic provenance did not bind the policy anchor")
            rows = [row for row in self.entries if "kind" not in query or row["kind"] == query["kind"][0]]
            result = self.page(rows, query)
        elif method == "GET" and path.startswith("/api/v1/provenance/excerpts/"):
            if path.rsplit("/", 1)[1] != self.excerpt["excerpt_id"]:
                return 404, {"code": "not-found", "message": "Synthetic unknown excerpt"}
            result = self.excerpt
        else:
            raise AssertionError("Synthetic functional route was not declared")
        result = copy.deepcopy(result)
        if self.fault is not None:
            result = self.fault(event, result)
        return 200, result


class ExtensionControls(unittest.TestCase):
    """Use actual library parsing and RecordingWorkspace method/path/outcome accounting in every request control."""

    def setUp(self):
        """Reset and preserve global recorder state without constructing a native workspace process."""
        recorder = extension.RecordingWorkspace
        self.original_recording = {name: (hasattr(recorder, name), getattr(recorder, name, None))
                                   for name in ("routes", "outcomes", "request_count", "accounting_failed")}
        recorder.routes = extension.contract_routes(Path(extension.__file__).resolve().parents[1])
        recorder.outcomes, recorder.request_count, recorder.accounting_failed = {}, 0, False
        self.client = object.__new__(recorder)
        self.client._port, self.client._capability, self.client._next_request_at = 1, "a" * 64, 0.0
        self.client._api_major, self.client._api_prefix = 1, "/api/v1"
        self.now = 0.0
        self.clock = mock.patch.object(time, "monotonic", side_effect=lambda: self.now)
        self.sleep = mock.patch.object(time, "sleep")
        self.clock.start()
        self.sleep.start()
        self.addCleanup(self.clock.stop)
        self.addCleanup(self.sleep.stop)
        self.addCleanup(self.restore_recording)

    def restore_recording(self):
        """Restore every original recorder class attribute after each isolated synthetic control."""
        for name, (present, value) in self.original_recording.items():
            if present:
                setattr(extension.RecordingWorkspace, name, value)
            else:
                delattr(extension.RecordingWorkspace, name)

    def transport(self, *, script=None, handler=None):
        """Replace only the unchanged client library network constructor and return its private observations."""
        transport = SyntheticTransport(handler=handler, script=script)
        patch = mock.patch.object(client_library.http.client, "HTTPConnection", side_effect=transport.connection)
        patch.start()
        self.addCleanup(patch.stop)
        return transport

    def assert_accounting(self, expected):
        """Reconcile actual recorder counts and ensure no payload, path or capability entered its outcome cells."""
        extension.RecordingWorkspace.ensure_accounting()
        self.assertEqual(extension.RecordingWorkspace.outcomes, expected)
        self.assertEqual(extension.RecordingWorkspace.request_count,
                         sum(sum(counts.values()) for counts in expected.values()))
        self.assertFalse(extension.RecordingWorkspace.accounting_failed)
        for counts in expected.values():
            self.assertEqual(set(counts), {"succeeded", "rejected", "transport_failed"})

    def test_actual_recorder_classifies_target_routes_without_mocking_operation_names(self):
        """Real parameterized contract routes classify all proposed read/validation and retained-result requests."""
        resource = "res_" + "a" * 32
        operation = "op_" + "b" * 32
        preview = "prev_" + "c" * 32
        excerpt = "ex_" + "d" * 32
        rows = [
            ("GET", "/api/v1/session", "getSession"),
            ("GET", "/api/v1/project/config-status", "getProjectConfigStatus"),
            ("GET", "/api/v1/resources/" + resource, "getResource"),
            ("GET", "/api/v1/resources/" + resource + "/validation", "getResourceValidation"),
            ("POST", "/api/v1/validation/runs", "runValidation"),
            ("POST", "/api/v1/mapping/checks", "checkMapping"),
            ("POST", "/api/v1/mapping/draft/validation", "validateMappingDraft"),
            ("GET", "/api/v1/applicability/report", "getApplicabilityReport"),
            ("GET", "/api/v1/review-queue/counts", "getReviewQueueCounts"),
            ("GET", "/api/v1/review-queue/items?page_size=1", "listReviewQueueItems"),
            ("GET", "/api/v1/provenance/entries?anchor=" + resource + "&page_size=1", "listProvenanceEntries"),
            ("GET", "/api/v1/provenance/excerpts/" + excerpt, "getProvenanceExcerpt"),
            ("GET", "/api/v1/conversions/" + operation, "getConversion"),
            ("GET", "/api/v1/effects/previews/" + preview, "getEffectPreview"),
        ]
        transport = self.transport(script=[(method, path, 200, {"synthetic": True}) for method, path, _ in rows])
        for method, path, _ in rows:
            self.assertEqual(self.client.request(method, path, None if method == "GET" else {}), {"synthetic": True})
        expected = {name: {"succeeded": 1, "rejected": 0, "transport_failed": 0} for _, _, name in rows}
        self.assert_accounting(expected)
        self.assertEqual(len(extension.RecordingWorkspace.routes), 39)
        self.assertNotIn("cancelOperation", expected)
        self.assertNotIn("unlockSession", expected)
        self.assertEqual([(e["method"], e["path"]) for e in transport.events], [(m, p) for m, p, _ in rows])
        self.assertTrue(all(c.closed for c in transport.connections))
        self.assertNotIn("a" * 64, json.dumps(extension.RecordingWorkspace.outcomes))

    def test_actual_recorder_preserves_rejection_transport_and_unknown_route_failure(self):
        """Actual typed and transport failures keep separate outcomes; an unknown route never reaches the network."""
        path = "/api/v1/resources/res_" + "e" * 32
        transport = self.transport(script=[("GET", path, 404, {"code": "not-found", "message": "Synthetic unknown resource"}),
                                           ("GET", path, 200, OSError("Synthetic transport failure"))])
        with self.assertRaises(client_library.WorkspaceError) as caught:
            self.client.request("GET", path)
        self.assertEqual(caught.exception.payload["code"], "not-found")
        with self.assertRaises(OSError):
            self.client.request("GET", path)
        self.assert_accounting({"getResource": {"succeeded": 0, "rejected": 1, "transport_failed": 1}})
        with self.assertRaises(RuntimeError):
            self.client.request("GET", "/api/v1/not-a-declared-route")
        self.assertEqual(len(transport.events), 2)
        self.assertEqual(extension.RecordingWorkspace.request_count, 2)
        self.assertTrue(extension.RecordingWorkspace.accounting_failed)
        self.assertTrue(all(c.closed for c in transport.connections))

    def page_value(self, items, *, cursor=None, total=None, version=None):
        """Construct an independent bounded-page response for exact traversal failure controls."""
        return {"resource_version": "f" * 64 if version is None else version,
                "page": {"items": items, "next_cursor": cursor,
                         "total_matching": len(items) if total is None else total}}

    def validation_value(self, *, errors=0, warnings=0, information=0, resource=None):
        """Construct closed diagnostics independently of the helper's classification or counters."""
        diagnostics = []
        for severity, count in (("error", errors), ("warning", warnings), ("info", information)):
            diagnostics.extend({"code": "synthetic-diagnostic", "severity": severity, "message": "Synthetic diagnostic",
                                "resource_id": resource, "field": None} for _ in range(count))
        return {"state": "invalid" if errors else "valid", "error_count": errors,
                "warning_count": warnings, "diagnostics": diagnostics}

    def conversion_value(self):
        """Provide one same-session terminal conversion with an exact unconsumed preview and explicit synthetic products."""
        preview = {"preview_id": "prev_" + "a" * 32, "operation_type": "policy-conversion",
                   "target": {"path": "catalog.json", "status": "create"}, "base_sha256": None,
                   "target_version": "b" * 64, "exact_bytes_sha256": "c" * 64, "input_hashes": [],
                   "validation": self.validation_value(), "semantic_summary": "Synthetic conversion",
                   "diff_text": "Synthetic diff", "diff_truncated": False,
                   "receipt": {"token": "synthetic-private-receipt-token"}}
        operation = {"operation_id": "op_" + "d" * 32, "kind": "conversion", "state": "succeeded",
                     "result": {"operation_id": "op_" + "d" * 32,
                                "products": [{"kind": "oscal-catalog", "statement_count": 1}],
                                "validation": self.validation_value(), "preview": preview}}
        return operation

    def test_budget_expired_and_exact_deadline_never_attempt_or_delegate(self):
        """An expired or equal absolute clock fences actual recorder/network work before incrementing attempts."""
        transport = self.transport(script=[])
        for now in (10, 11):
            with self.subTest(now=now):
                self.now = now
                budget = extension.ExtensionBudget(deadline=10)
                with self.assertRaises(RuntimeError):
                    budget.request(self.client, "GET", "/api/v1/session")
                self.assertEqual(budget.attempts, 0)
                self.assertEqual(budget.deadline, 10)
        self.assertEqual(transport.events, [])
        self.assertEqual(transport.connections, [])
        self.assert_accounting({})

    def test_budget_late_success_fails_but_retains_actual_successful_request(self):
        """A genuine synthetic 200 returned after the deadline remains recorded but earns no helper success."""
        def late(event):
            """Move the clock at the actual transport boundary rather than bypassing the helper fence."""
            self.assertEqual((event["method"], event["path"]), ("GET", "/api/v1/session"))
            self.now = 10
            return 200, {"synthetic": True}
        transport = self.transport(handler=late)
        budget = extension.ExtensionBudget(deadline=10)
        with self.assertRaises(RuntimeError):
            budget.request(self.client, "GET", "/api/v1/session")
        self.assertEqual(budget.attempts, 1)
        self.assertEqual(budget.deadline, 10)
        self.assertEqual(len(transport.events), 1)
        self.assertTrue(transport.connections[0].closed)
        self.assert_accounting({"getSession": {"succeeded": 1, "rejected": 0, "transport_failed": 0}})

    def test_budget_exact200_requests_reconcile_and_201_never_delegates(self):
        """The extension's exact cap is distinct from the unchanged shared recorder limit and never permits attempt 201."""
        transport = self.transport(handler=lambda event: (200, {"synthetic": True}))
        budget = extension.ExtensionBudget(deadline=10)
        for _ in range(200):
            self.assertEqual(budget.request(self.client, "GET", "/api/v1/session"), {"synthetic": True})
        with self.assertRaises(RuntimeError):
            budget.request(self.client, "GET", "/api/v1/session")
        self.assertEqual(budget.attempts, 200)
        self.assertEqual(len(transport.events), 200)
        self.assertEqual(extension.MAX_RECORDED_REQUESTS, 10000)
        self.assert_accounting({"getSession": {"succeeded": 200, "rejected": 0, "transport_failed": 0}})

    def test_budget_refuses_bool_zero_and_over_cap_attempt_limits(self):
        """Malformed attempt limits cannot disable or enlarge the explicit extension request cap."""
        for limit in (True, False, 0, -1, 201, 200.0, "200", None):
            with self.subTest(limit=limit), self.assertRaises(ValueError):
                extension.ExtensionBudget(deadline=10, limit=limit)
        self.assert_accounting({})

    def test_proxy_commit_preserves_body_idempotency_and_confirmation(self):
        """The budget proxy delegates exact commit semantics through the real recorder instead of emulating effects."""
        operation = self.conversion_value()
        preview = operation["result"]["preview"]
        path = "/api/v1/effects/commits"
        transport = self.transport(script=[("POST", path, 200, {"synthetic": "commit"})])
        budget = extension.ExtensionBudget(deadline=10)
        proxy = extension.BudgetedClient(self.client, budget, False)
        for confirmed in (False, 1, "true", None):
            with self.subTest(confirmed=confirmed), self.assertRaises(ValueError):
                proxy.commit(preview, confirmed=confirmed, idempotency_key="synthetic-key")
        self.assertEqual(budget.attempts, 0)
        self.assertEqual(proxy.commit(preview, confirmed=True, idempotency_key="synthetic-key"), {"synthetic": "commit"})
        self.assertEqual(transport.events[0]["body"], {"receipt": preview["receipt"]["token"],
                         "observed_version": preview["target_version"], "confirmed": True})
        self.assertEqual(transport.events[0]["headers"]["Idempotency-Key"], "synthetic-key")
        self.assertEqual(budget.attempts, 1)
        self.assert_accounting({"commitEffectPreview": {"succeeded": 1, "rejected": 0, "transport_failed": 0}})

    def test_proxy_wait_polls_exact_accepted_id_and_stops_on_terminal_result(self):
        """Actual proxy polling stays in the accepted operation's session and consumes the same request budget."""
        operation = self.conversion_value()
        pending = {"operation_id": operation["operation_id"], "kind": "conversion", "state": "pending"}
        path = "/api/v1/operations/" + operation["operation_id"]
        transport = self.transport(script=[("GET", path, 200, {"operation_id": operation["operation_id"], "kind": "conversion", "state": "running"}),
                                           ("GET", path, 200, operation)])
        budget = extension.ExtensionBudget(deadline=10)
        proxy = extension.BudgetedClient(self.client, budget, False)
        self.assertEqual(proxy.wait(pending), operation)
        self.assertEqual(budget.attempts, 2)
        self.assertEqual([e["path"] for e in transport.events], [path, path])
        self.assert_accounting({"getOperation": {"succeeded": 2, "rejected": 0, "transport_failed": 0}})

    def test_proxy_wait_equal_deadline_abstains_before_poll(self):
        """The proxy's wait ceiling cannot renew the expired shared extension deadline or dispatch a poll."""
        self.now = 10
        transport = self.transport(script=[])
        budget = extension.ExtensionBudget(deadline=10)
        proxy = extension.BudgetedClient(self.client, budget, False)
        with self.assertRaises(TimeoutError):
            proxy.wait({"operation_id": "op_" + "a" * 32, "kind": "conversion", "state": "running"})
        self.assertEqual(transport.events, [])
        self.assertEqual(budget.attempts, 0)
        self.assert_accounting({})

    def test_pagination_two_pages_preserves_filters_version_cursor_and_unique_ids(self):
        """Two actual one-item page requests return a complete stable denominator and exact forwarded cursor."""
        items = [{"entry_id": "prov_" + "a" * 32}, {"entry_id": "prov_" + "b" * 32}]
        calls = []
        def pages(event):
            """Observe actual serialized query parameters while supplying independent two-page facts."""
            calls.append(parse_qs(urlsplit(event["path"]).query))
            self.assertEqual(event["method"], "GET")
            self.assertEqual(calls[-1]["anchor"], ["res_" + "c" * 32])
            self.assertEqual(calls[-1]["page_size"], ["1"])
            if len(calls) == 1:
                self.assertNotIn("cursor", calls[-1])
                return 200, self.page_value(items[:1], total=2, cursor="opaque +/cursor")
            self.assertEqual(calls[-1]["cursor"], ["opaque +/cursor"])
            return 200, self.page_value(items[1:], total=2)
        self.transport(handler=pages)
        budget = extension.ExtensionBudget(deadline=10)
        filters = {"anchor": "res_" + "c" * 32}
        self.assertEqual(extension.bounded_pages(budget, self.client, "/api/v1/provenance/entries", filters), items)
        self.assertEqual(filters, {"anchor": "res_" + "c" * 32})
        self.assertEqual(budget.attempts, 2)
        self.assert_accounting({"listProvenanceEntries": {"succeeded": 2, "rejected": 0, "transport_failed": 0}})

    def test_pagination_rejects_second_page_version_total_duplicate_and_partial_results(self):
        """Each stable-traversal violation rejects a complete-looking response after retaining both real requests."""
        first = self.page_value([{"item_id": "qi_" + "a" * 32}], cursor="next", total=2)
        faults = [self.page_value([{"item_id": "qi_" + "b" * 32}], total=2, version="e" * 64),
                  self.page_value([{"item_id": "qi_" + "b" * 32}], total=3),
                  self.page_value([{"item_id": "qi_" + "a" * 32}], total=2),
                  self.page_value([], total=2)]
        total_requests = 0
        for second in faults:
            with self.subTest(second=second):
                transport = self.transport(script=[("GET", "/api/v1/review-queue/items?page_size=1", 200, first),
                                ("GET", "/api/v1/review-queue/items?page_size=1&cursor=next", 200, second)])
                budget = extension.ExtensionBudget(deadline=10)
                with self.assertRaises(AssertionError):
                    extension.bounded_pages(budget, self.client, "/api/v1/review-queue/items")
                self.assertEqual(budget.attempts, 2)
                self.assertEqual(len(transport.events), 2)
                total_requests += 2
        self.assert_accounting({"listReviewQueueItems": {"succeeded": total_requests, "rejected": 0, "transport_failed": 0}})

    def test_pagination_cycles_bounds_and_unbounded_cursor_never_receive_partial_credit(self):
        """Nonadvancing cursors, an exhausted page ceiling and a cursor over the contract bound all fail closed."""
        requests = 0
        for second_cursor in ("next", "another"):
            with self.subTest(cursor=second_cursor):
                transport = self.transport(script=[("GET", "/api/v1/review-queue/items?page_size=1", 200,
                                      self.page_value([{"item_id": "qi_" + "a" * 32}], cursor="next", total=2)),
                                  ("GET", "/api/v1/review-queue/items?page_size=1&cursor=next", 200,
                                      self.page_value([{"item_id": "qi_" + "b" * 32}], cursor=second_cursor, total=2))])
                with self.assertRaises(AssertionError):
                    extension.bounded_pages(extension.ExtensionBudget(deadline=10), self.client,
                                            "/api/v1/review-queue/items", limit=2)
                self.assertEqual(len(transport.events), 2)
                requests += 2
        transport = self.transport(script=[("GET", "/api/v1/review-queue/items?page_size=1", 200,
                                            self.page_value([{"item_id": "qi_" + "c" * 32}], cursor="x" * 513, total=2))])
        with self.assertRaises(AssertionError):
            extension.bounded_pages(extension.ExtensionBudget(deadline=10), self.client, "/api/v1/review-queue/items")
        self.assertEqual(len(transport.events), 1)
        self.assert_accounting({"listReviewQueueItems": {"succeeded": requests + 1, "rejected": 0, "transport_failed": 0}})

    def test_pagination_rejects_open_shapes_bool_totals_and_caller_paging_overrides(self):
        """Malformed page envelopes and caller-controlled cursor/size cannot shrink or spoof the selected denominator."""
        requests = 0
        for mutation in (lambda value: value.update(extra=True),
                         lambda value: value["page"].update(extra=True),
                         lambda value: value["page"].update(total_matching=True),
                         lambda value: value["page"].update(items=[{"item_id": "one"}, {"item_id": "two"}]),
                         lambda value: value.update(resource_version="unknown")):
            value = self.page_value([], total=0)
            mutation(value)
            transport = self.transport(script=[("GET", "/api/v1/review-queue/items?page_size=1", 200, value)])
            with self.assertRaises(AssertionError):
                extension.bounded_pages(extension.ExtensionBudget(deadline=10), self.client, "/api/v1/review-queue/items")
            self.assertEqual(len(transport.events), 1)
            requests += 1
        transport = self.transport(script=[])
        for filters in ({"cursor": "hidden"}, {"page_size": "2"}):
            with self.subTest(filters=filters), self.assertRaises(AssertionError):
                extension.bounded_pages(extension.ExtensionBudget(deadline=10), self.client,
                                        "/api/v1/review-queue/items", filters)
        for limit in (True, 0, 17, 1.0):
            with self.subTest(limit=limit), self.assertRaises(AssertionError):
                extension.bounded_pages(extension.ExtensionBudget(deadline=10), self.client,
                                        "/api/v1/review-queue/items", limit=limit)
        self.assertEqual(transport.events, [])
        self.assert_accounting({"listReviewQueueItems": {"succeeded": requests, "rejected": 0, "transport_failed": 0}})

    def test_budget_constructor_requires_finite_exact_supported_deadline_and_duration(self):
        """NaN, infinity, bools and unsupported duration values cannot turn the absolute clock fence off."""
        for deadline in (True, False, float("nan"), float("inf"), -1, "10"):
            with self.subTest(deadline=deadline), self.assertRaises(ValueError):
                extension.ExtensionBudget(deadline=deadline)
        for seconds in (True, False, float("nan"), float("inf"), -1, 0, 301, "300"):
            with self.subTest(seconds=seconds), self.assertRaises(ValueError):
                extension.ExtensionBudget(seconds=seconds)
        budget = extension.ExtensionBudget(seconds=300)
        self.assertEqual(budget.deadline, 300)
        self.assertEqual(budget.attempts, 0)
        self.assert_accounting({})

    def test_validation_info_and_warning_are_valid_while_error_counts_define_invalid_state(self):
        """The exact contract severity enum permits informational diagnostics without fabricating error/warning counts."""
        resource = "res_" + "a" * 32
        for errors, warnings, information in ((0, 0, 0), (0, 1, 1), (1, 1, 1)):
            with self.subTest(errors=errors, warnings=warnings, information=information):
                value = self.validation_value(errors=errors, warnings=warnings, information=information, resource=resource)
                self.assertIs(extension.validate_report(value), value)
                self.assertEqual(value["state"], "invalid" if errors else "valid")
                self.assertEqual(len(value["diagnostics"]), errors + warnings + information)

    def test_validation_refuses_open_missing_fields_bool_counts_and_unreconciled_severity(self):
        """Open/missing report or diagnostic fields and every count/state mismatch fail rather than pass shape-only checks."""
        mutations = [lambda value: value.update(extra="synthetic private field"),
                     lambda value: value.pop("diagnostics"),
                     lambda value: value.update(error_count=True),
                     lambda value: value.update(warning_count=-1),
                     lambda value: value.update(state="valid"),
                     lambda value: value.update(error_count=0),
                     lambda value: value["diagnostics"][0].update(extra="synthetic private field"),
                     lambda value: value["diagnostics"][0].pop("code"),
                     lambda value: value["diagnostics"][0].pop("message"),
                     lambda value: value["diagnostics"][0].update(severity="fatal"),
                     lambda value: value["diagnostics"][0].update(resource_id="../foreign"),
                     lambda value: value["diagnostics"][0].update(field=True),
                     lambda value: value["diagnostics"][0].update(field="x" * 257),
                     lambda value: value["diagnostics"][0].update(message=""),
                     lambda value: value["diagnostics"][0].update(message="x" * 1001),
                     lambda value: value["diagnostics"][0].update(code="x"),
                     lambda value: value["diagnostics"][0].update(code="x" * 101),
                     lambda value: value.update(error_count=501, diagnostics=value["diagnostics"] * 501)]
        for number, mutate in enumerate(mutations):
            with self.subTest(mutation=number):
                value = self.validation_value(errors=1, resource="res_" + "a" * 32)
                mutate(value)
                with self.assertRaises(AssertionError):
                    extension.validate_report(value)

    def test_typed_negative_requires_exact_code_and_never_swallows_transport_or_accidental_success(self):
        """Negative conformance means one actual matching WorkspaceError; different rejections, transport faults and 200 fail."""
        path = "/api/v1/resources/res_" + "0" * 32
        transport = self.transport(script=[("GET", path, 404, {"code": "not-found", "message": "Synthetic absence"}),
                                           ("GET", path, 422, {"code": "validation-failed", "message": "Synthetic mismatch"}),
                                           ("GET", path, 200, OSError("Synthetic transport fault")),
                                           ("GET", path, 200, {"synthetic": True})])
        budget = extension.ExtensionBudget(deadline=10)
        self.assertIsNone(extension.expect_error(budget, self.client, "GET", path, None, "not-found"))
        with self.assertRaises(AssertionError):
            extension.expect_error(budget, self.client, "GET", path, None, "not-found")
        with self.assertRaises(OSError):
            extension.expect_error(budget, self.client, "GET", path, None, "not-found")
        with self.assertRaises(AssertionError):
            extension.expect_error(budget, self.client, "GET", path, None, "not-found")
        self.assertEqual(budget.attempts, 4)
        self.assertEqual(len(transport.events), 4)
        self.assert_accounting({"getResource": {"succeeded": 1, "rejected": 2, "transport_failed": 1}})

    def test_retained_conversion_gets_exact_result_and_unconsumed_preview_in_same_session(self):
        """Two actual documented requests bind conversion identity and complete result/preview equality before any commit."""
        operation = self.conversion_value()
        result = operation["result"]
        conversion_path = "/api/v1/conversions/" + operation["operation_id"]
        preview_path = "/api/v1/effects/previews/" + result["preview"]["preview_id"]
        transport = self.transport(script=[("GET", conversion_path, 200, result),
                                           ("GET", preview_path, 200, result["preview"])])
        budget = extension.ExtensionBudget(deadline=10)
        before = copy.deepcopy(operation)
        self.assertEqual(extension.retained_conversion(budget, self.client, operation),
                         (operation["operation_id"], result["preview"]["preview_id"]))
        self.assertEqual(operation, before)
        self.assertEqual(budget.attempts, 2)
        self.assertEqual([(e["method"], e["path"]) for e in transport.events],
                         [("GET", conversion_path), ("GET", preview_path)])
        self.assert_accounting({"getConversion": {"succeeded": 1, "rejected": 0, "transport_failed": 0},
                                "getEffectPreview": {"succeeded": 1, "rejected": 0, "transport_failed": 0}})

    def test_retained_conversion_refuses_nonterminal_foreign_result_and_preview_divergence(self):
        """Wrong source state, conversion identity, returned result and preview equality each reject before effect credit."""
        transport = self.transport(script=[])
        for mutation in (lambda value: value.update(state="running"), lambda value: value.update(kind="export")):
            operation = self.conversion_value()
            mutation(operation)
            with self.assertRaises(AssertionError):
                extension.retained_conversion(extension.ExtensionBudget(deadline=10), self.client, operation)
        self.assertEqual(transport.events, [])
        requests = 0
        for where in ("result", "preview"):
            operation = self.conversion_value()
            result = copy.deepcopy(operation["result"])
            conversion_path = "/api/v1/conversions/" + operation["operation_id"]
            preview_path = "/api/v1/effects/previews/" + result["preview"]["preview_id"]
            if where == "result":
                result["operation_id"] = "op_" + "e" * 32
                script = [("GET", conversion_path, 200, result)]
            else:
                preview = copy.deepcopy(result["preview"])
                preview["exact_bytes_sha256"] = "f" * 64
                script = [("GET", conversion_path, 200, result), ("GET", preview_path, 200, preview)]
            transport = self.transport(script=script)
            budget = extension.ExtensionBudget(deadline=10)
            with self.subTest(where=where), self.assertRaises(AssertionError):
                extension.retained_conversion(budget, self.client, operation)
            self.assertEqual(budget.attempts, len(script))
            self.assertEqual(len(transport.events), len(script))
            requests += len(script)
        self.assertEqual(requests, 3)
        self.assert_accounting({"getConversion": {"succeeded": 2, "rejected": 0, "transport_failed": 0},
                                "getEffectPreview": {"succeeded": 1, "rejected": 0, "transport_failed": 0}})

    def test_resource_metadata_binds_exact_captured_bytes_without_opening_server_paths(self):
        """Resource inspection compares local captured hashes/size/version and never opens a supplied metadata path."""
        raw = b"Synthetic policy bytes\n"
        files = {"policy.md": raw}
        digest = hashlib.sha256(raw).hexdigest()
        resource = {"resource_id": "res_" + "a" * 32, "key": "policy", "role": "policy-source", "path": "policy.md",
                    "sha256": digest, "version": digest, "size_bytes": len(raw), "validation_state": "valid", "stale": False}
        original = copy.deepcopy(resource)
        with mock.patch("builtins.open", side_effect=AssertionError("No server path may be opened")) as opened:
            self.assertIsNone(extension.inspect_resource(resource, files))
            for change in ({"sha256": "f" * 64}, {"version": "e" * 64}, {"size_bytes": True},
                           {"size_bytes": len(raw) + 1}, {"path": "../foreign"}, {"stale": 0}, {"extra": True}):
                value = dict(resource, **change)
                with self.subTest(change=change), self.assertRaises(AssertionError):
                    extension.inspect_resource(value, files)
            opened.assert_not_called()
        self.assertEqual(resource, original)
        self.assertEqual(files, {"policy.md": raw})

    def test_excerpt_whole_unicode_lines_hash_and_real_truncation_reconcile(self):
        """A real captured 101-line source yields exactly the first 100 whole UTF-8 lines and true truncation."""
        raw = "".join("Synthetic caf\u00e9 line " + str(number) + "\n" for number in range(101)).encode()
        resource = {"resource_id": "res_" + "a" * 32, "path": "policy.md", "sha256": hashlib.sha256(raw).hexdigest()}
        files, resources = {"policy.md": raw}, {resource["resource_id"]: resource}
        lines = raw.decode().splitlines(keepends=True)
        value = {"excerpt_id": "ex_" + "b" * 32, "resource_id": resource["resource_id"], "sha256": resource["sha256"],
                 "start_line": 1, "end_line": 100, "text": "".join(lines[:100]), "truncated": True}
        original = copy.deepcopy(value)
        self.assertIsNone(extension.inspect_excerpt(value, resources, files))
        for change in ({"start_line": True}, {"end_line": 102}, {"start_line": 0}, {"truncated": False},
                       {"truncated": 1}, {"text": value["text"] + "partial"}, {"sha256": "f" * 64}, {"extra": True}):
            with self.subTest(change=change), self.assertRaises(AssertionError):
                extension.inspect_excerpt(dict(value, **change), resources, files)
        self.assertEqual(value, original)
        self.assertEqual(files["policy.md"], raw)

    def test_fixture_capture_is_complete_flat_bounded_and_does_not_write(self):
        """Exact private byte capture rejects directories, links and oversized files instead of ignoring owned content."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "first.md").write_bytes(b"Synthetic first\n")
            (root / "second.json").write_bytes(b"{}\n")
            before = {path.name: path.read_bytes() for path in root.iterdir()}
            self.assertEqual(extension.fixture_bytes(root), before)
            self.assertEqual({path.name: path.read_bytes() for path in root.iterdir()}, before)
            nested = root / "nested"
            nested.mkdir()
            with self.assertRaises(AssertionError):
                extension.fixture_bytes(root)
            nested.rmdir()
            link = root / "link"
            link.symlink_to(root / "first.md")
            with self.assertRaises(AssertionError):
                extension.fixture_bytes(root)
            link.unlink()
            oversized = root / "too-large.md"
            oversized.write_bytes(b"x" * (1024 * 1024 + 1))
            with self.assertRaises(AssertionError):
                extension.fixture_bytes(root)
            oversized.unlink()
            self.assertEqual(extension.fixture_bytes(root), before)

    def test_fixture_stream_stops_at_seventeenth_entry_without_unbounded_retention(self):
        """A synthetic large directory iterator stops at cap plus one and reads only the 16 admitted flat files."""
        yielded = []
        def entries():
            """Expose 1000 fake flat entries while recording each item actually requested by the helper."""
            for number in range(1000):
                yielded.append(number)
                yield types.SimpleNamespace(name="fixture-" + str(number), path="/synthetic/" + str(number),
                                            stat=lambda follow_symlinks=False: types.SimpleNamespace(st_mode=stat.S_IFREG | 0o600))
        with mock.patch.object(extension.os, "scandir", return_value=contextlib.nullcontext(entries())), \
             mock.patch("builtins.open", side_effect=lambda path, mode: io.BytesIO(b"Synthetic bytes")) as opened:
            with self.assertRaises(AssertionError):
                extension.fixture_bytes(Path("/synthetic"))
        self.assertEqual(yielded, list(range(17)))
        self.assertEqual(opened.call_count, 16)

    def test_proxy_wait_refuses_changed_operation_identity_kind_and_unknown_terminal_state(self):
        """A poll response cannot transfer another operation or reinterpret an unsupported state as success."""
        operation = self.conversion_value()
        pending = {"operation_id": operation["operation_id"], "kind": "conversion", "state": "pending"}
        path = "/api/v1/operations/" + operation["operation_id"]
        total = 0
        for change in ({"operation_id": "op_" + "e" * 32}, {"kind": "export"}, {"state": "unknown"}):
            with self.subTest(change=change):
                transport = self.transport(script=[("GET", path, 200, dict(operation, **change))])
                budget = extension.ExtensionBudget(deadline=10)
                with self.assertRaises(AssertionError):
                    extension.BudgetedClient(self.client, budget, False).wait(pending)
                self.assertEqual(budget.attempts, 1)
                self.assertEqual([event["path"] for event in transport.events], [path])
                total += 1
        self.assert_accounting({"getOperation": {"succeeded": total, "rejected": 0, "transport_failed": 0}})

    def functional_fixture(self, root, read_only=False):
        """Attach independent stateful private responses to the real recorder without mocking helper checks."""
        fixture = FunctionalFixture(root, read_only)
        transport = self.transport(handler=fixture.response)
        return fixture, transport

    def test_functional_reads_both_session_modes_reconcile_twelve_target_families_and_preserve_files(self):
        """The full helper executes semantic reads against synthetic seven-resource state in both modes with real accounting."""
        budget = extension.ExtensionBudget(deadline=10)
        all_events = []
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture, transport = self.functional_fixture(root)
            before = {path.name: path.read_bytes() for path in root.iterdir()}
            mapping_before = copy.deepcopy(fixture.mapping)
            for read_only in (False, True):
                fixture.read_only = read_only
                value = extension.functional_reads(budget, self.client, root, 7, fixture.analysis, read_only=read_only)
                self.assertEqual(value, fixture.view)
                self.assertEqual({path.name: path.read_bytes() for path in root.iterdir()}, before)
                self.assertEqual(fixture.mapping, mapping_before)
            all_events.extend(transport.events)
            self.assertTrue(all(connection.closed for connection in transport.connections))
        expected = {
            "getSession": {"succeeded": 4, "rejected": 0, "transport_failed": 0},
            "getProjectConfigStatus": {"succeeded": 2, "rejected": 0, "transport_failed": 0},
            "listResources": {"succeeded": 14, "rejected": 0, "transport_failed": 0},
            "getResource": {"succeeded": 14, "rejected": 2, "transport_failed": 0},
            "getResourceValidation": {"succeeded": 14, "rejected": 0, "transport_failed": 0},
            "runValidation": {"succeeded": 4, "rejected": 2, "transport_failed": 0},
            "checkMapping": {"succeeded": 2, "rejected": 0, "transport_failed": 0},
            "getMappingDraft": {"succeeded": 2, "rejected": 0, "transport_failed": 0},
            "validateMappingDraft": {"succeeded": 4, "rejected": 0, "transport_failed": 0},
            "getApplicabilityReport": {"succeeded": 2, "rejected": 0, "transport_failed": 0},
            "getReviewQueueCounts": {"succeeded": 2, "rejected": 0, "transport_failed": 0},
            "listReviewQueueItems": {"succeeded": 8, "rejected": 0, "transport_failed": 0},
            "getProjectSummary": {"succeeded": 2, "rejected": 0, "transport_failed": 0},
            "listProvenanceEntries": {"succeeded": 6, "rejected": 0, "transport_failed": 0},
            "getProvenanceExcerpt": {"succeeded": 2, "rejected": 2, "transport_failed": 0},
        }
        self.assert_accounting(expected)
        self.assertEqual(budget.attempts, len(all_events))
        self.assertLess(budget.attempts, 200)
        targets = {"checkMapping", "getApplicabilityReport", "getProjectConfigStatus", "getProvenanceExcerpt", "getResource",
                   "getResourceValidation", "getReviewQueueCounts", "getSession", "listProvenanceEntries",
                   "listReviewQueueItems", "runValidation", "validateMappingDraft"}
        self.assertTrue(targets <= set(expected))
        self.assertNotIn("cancelOperation", expected)
        self.assertNotIn("unlockSession", expected)
        self.assertFalse(any(event["method"] == "PUT" or event["path"] == "/api/v1/effects/commits" for event in all_events))
        self.assertNotIn("a" * 64, json.dumps(extension.RecordingWorkspace.outcomes))

    def test_functional_reads_refuse_session_index_resource_and_file_fingerprint_divergence(self):
        """Each metadata mismatch against the owned index or captured bytes rejects rather than counting a route as conformance."""
        modes = ("mode", "read_only", "api_bool", "project_label", "resource_role", "resource_hash", "duplicate_key")
        for mode in modes:
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                fixture, transport = self.functional_fixture(root)
                def fault(event, value):
                    """Alter only the selected returned metadata; authoritative private fixture bytes remain exact."""
                    path = urlsplit(event["path"]).path
                    if path == "/api/v1/session":
                        if mode == "mode": value["mode"] = "browser"
                        elif mode == "read_only": value["read_only"] = True
                        elif mode == "api_bool": value["api_major"] = True
                        elif mode == "project_label": value["project_label"] = "Other project"
                    elif path == "/api/v1/resources" and value["page"]["items"]:
                        row = value["page"]["items"][0]
                        if mode == "resource_role": row["role"] = "policy-source"
                        elif mode == "resource_hash": row["sha256"] = "f" * 64
                        elif mode == "duplicate_key": row["key"] = "policy"
                    return value
                fixture.fault = fault
                before = {path.name: path.read_bytes() for path in root.iterdir()}
                with self.assertRaises(AssertionError):
                    extension.functional_reads(extension.ExtensionBudget(deadline=10), self.client, root,
                                               7, fixture.analysis, read_only=False)
                self.assertGreater(len(transport.events), 0)
                self.assertEqual({path.name: path.read_bytes() for path in root.iterdir()}, before)
                self.assertTrue(all(connection.closed for connection in transport.connections))
        extension.RecordingWorkspace.ensure_accounting()

    def test_functional_reads_refuse_report_queue_counts_filters_provenance_and_excerpt_drift(self):
        """Later semantic report/paging/line failures keep their actual requests while discarding any overall pass claim."""
        modes = ("report_version", "report_pin", "report_counts", "report_omitted", "report_duplicate",
                 "report_reordered", "report_policy_only", "queue_bool", "queue_duplicate_reason",
                 "queue_count", "queue_filter", "provenance_pin", "provenance_filter", "excerpt_text", "source_write")
        for mode in modes:
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                fixture, transport = self.functional_fixture(root)
                def fault(event, value):
                    """Introduce one independently selected server/content inconsistency at its authentic helper boundary."""
                    url = urlsplit(event["path"])
                    query = parse_qs(url.query)
                    if url.path == "/api/v1/applicability/report":
                        if mode == "report_version": value["version"] = "f" * 64
                        elif mode == "report_pin": value["input_fingerprints"][0]["sha256"] = "e" * 64
                        elif mode == "report_counts": value["classification_counts"][0]["count"] = 1
                        elif mode == "report_omitted": value["input_fingerprints"].pop()
                        elif mode == "report_duplicate": value["input_fingerprints"][2] = copy.deepcopy(value["input_fingerprints"][0])
                        elif mode == "report_reordered": value["input_fingerprints"].reverse()
                        elif mode == "report_policy_only": value["input_fingerprints"] = [{"resource_id": fixture.policy["resource_id"],
                                                      "sha256": fixture.policy["sha256"], "matches_current": True}]
                    elif url.path == "/api/v1/review-queue/counts":
                        if mode == "queue_bool": value["total_open"] = True
                        elif mode == "queue_duplicate_reason": value["by_reason"].append(copy.deepcopy(value["by_reason"][0]))
                        elif mode == "queue_count": value["total_open"] = 3
                    elif url.path == "/api/v1/review-queue/items" and "reason_code" in query and mode == "queue_filter":
                        value["page"]["items"][0]["summary"] = "Altered filtered row"
                    elif url.path == "/api/v1/provenance/entries" and value["page"]["items"]:
                        if mode == "provenance_pin": value["page"]["items"][0]["fingerprint"] = "d" * 64
                        elif mode == "provenance_filter" and "kind" in query:
                            value["page"]["items"][0]["label"] = "Altered filtered entry"
                    elif url.path == "/api/v1/provenance/excerpts/" + fixture.excerpt["excerpt_id"]:
                        if mode == "excerpt_text": value["text"] += "not a source line"
                        elif mode == "source_write": (root / "unexpected.txt").write_bytes(b"Synthetic unauthorized effect")
                    return value
                fixture.fault = fault
                before = {path.name: path.read_bytes() for path in root.iterdir()}
                with self.assertRaises(AssertionError):
                    extension.functional_reads(extension.ExtensionBudget(deadline=10), self.client, root,
                                               7, fixture.analysis, read_only=False)
                self.assertGreater(len(transport.events), 0)
                after = {path.name: path.read_bytes() for path in root.iterdir()}
                if mode == "source_write":
                    self.assertEqual(after.pop("unexpected.txt"), b"Synthetic unauthorized effect")
                self.assertEqual(after, before)
                self.assertTrue(all(connection.closed for connection in transport.connections))
        extension.RecordingWorkspace.ensure_accounting()

    def test_shutdown_requires_natural_zero_exit_and_actual_post_close_transport_refusal(self):
        """Cleanup facts come from the original owner and actual failed request, while forced/nonzero/live responses reject."""
        poll = mock.Mock(return_value=1)
        self.client.process = types.SimpleNamespace(poll=poll)
        transport = self.transport(script=[])
        with self.assertRaises(AssertionError):
            extension.assert_shutdown(self.client)
        self.assertEqual(transport.events, [])
        poll.return_value = 0
        transport = self.transport(script=[("GET", "/api/v1/project/summary", 200, OSError("Synthetic closed listener")),
                                           ("GET", "/api/v1/project/summary", 200, {"synthetic": "still live"})])
        self.assertIsNone(extension.assert_shutdown(self.client))
        with self.assertRaises(AssertionError):
            extension.assert_shutdown(self.client)
        self.assertEqual(len(transport.events), 2)
        self.assert_accounting({"getProjectSummary": {"succeeded": 1, "rejected": 0, "transport_failed": 1}})

    def test_typed_negative_propagates_unexpected_failure_and_preserves_recorder_failure_latch(self):
        """Unexpected library errors escape the exact-code helper and cannot be normalized into an accepted rejection."""
        path = "/api/v1/resources/res_" + "0" * 32
        fault = RuntimeError("Synthetic unexpected adapter fault")
        transport = self.transport(script=[("GET", path, 200, fault)])
        budget = extension.ExtensionBudget(deadline=10)
        with self.assertRaises(RuntimeError) as caught:
            extension.expect_error(budget, self.client, "GET", path, None, "not-found")
        self.assertIs(caught.exception, fault)
        self.assertEqual(budget.attempts, 1)
        self.assertEqual(len(transport.events), 1)
        self.assertEqual(extension.RecordingWorkspace.outcomes, {})
        self.assertEqual(extension.RecordingWorkspace.request_count, 1)
        self.assertTrue(extension.RecordingWorkspace.accounting_failed)
        with self.assertRaises(RuntimeError):
            extension.RecordingWorkspace.ensure_accounting()

    def test_queue_paging_primary_item_ids_allow_two_rows_for_one_resource(self):
        """Two distinct queue items sharing a resource pointer remain distinct by the route's primary item identity."""
        resource = "res_" + "a" * 32
        items = [{"item_id": "qi_" + "b" * 32, "resource_id": resource, "reason_code": "invalid-resource", "summary": "One"},
                 {"item_id": "qi_" + "c" * 32, "resource_id": resource, "reason_code": "stale-input", "summary": "Two"}]
        transport = self.transport(script=[("GET", "/api/v1/review-queue/items?page_size=1", 200,
                                            self.page_value(items[:1], cursor="next", total=2)),
                                           ("GET", "/api/v1/review-queue/items?page_size=1&cursor=next", 200,
                                            self.page_value(items[1:], total=2))])
        budget = extension.ExtensionBudget(deadline=10)
        self.assertEqual(extension.bounded_pages(budget, self.client, "/api/v1/review-queue/items"), items)
        self.assertEqual(budget.attempts, 2)
        self.assertEqual(len(transport.events), 2)
        self.assert_accounting({"listReviewQueueItems": {"succeeded": 2, "rejected": 0, "transport_failed": 0}})

    def test_queue_paging_duplicate_item_id_rejects_even_with_different_resource_pointers(self):
        """Changing a secondary resource pointer cannot conceal a duplicated queue item's primary identity."""
        items = [{"item_id": "qi_" + "b" * 32, "resource_id": "res_" + "a" * 32},
                 {"item_id": "qi_" + "b" * 32, "resource_id": "res_" + "c" * 32}]
        transport = self.transport(script=[("GET", "/api/v1/review-queue/items?page_size=1", 200,
                                            self.page_value(items[:1], cursor="next", total=2)),
                                           ("GET", "/api/v1/review-queue/items?page_size=1&cursor=next", 200,
                                            self.page_value(items[1:], total=2))])
        budget = extension.ExtensionBudget(deadline=10)
        with self.assertRaises(AssertionError):
            extension.bounded_pages(budget, self.client, "/api/v1/review-queue/items")
        self.assertEqual(budget.attempts, 2)
        self.assertEqual(len(transport.events), 2)
        self.assert_accounting({"listReviewQueueItems": {"succeeded": 2, "rejected": 0, "transport_failed": 0}})


    def test_pagination_route_primary_id_patterns_and_length_boundaries(self):
        """Each route rejects malformed primary IDs after its actual request and accepts both exact length boundaries."""
        routes = (("/api/v1/resources", "resource_id", "res", "listResources"),
                  ("/api/v1/review-queue/items", "item_id", "qi", "listReviewQueueItems"),
                  ("/api/v1/provenance/entries", "entry_id", "prov", "listProvenanceEntries"))
        expected = {}
        for route, field, prefix, operation in routes:
            invalid = ("", prefix + "_" + "a" * 11, prefix + "_" + "a" * 81,
                       prefix + "_" + "A" * 12, "wrong_" + "a" * 12,
                       prefix + "_" + "a" * 12 + "\n", None, True, 123)
            for identity in invalid:
                with self.subTest(route=route, identity=identity):
                    transport = self.transport(script=[("GET", route + "?page_size=1", 200,
                                                       self.page_value([{field: identity}], total=1))])
                    budget = extension.ExtensionBudget(deadline=10)
                    with self.assertRaises(AssertionError):
                        extension.bounded_pages(budget, self.client, route)
                    self.assertEqual(budget.attempts, 1)
                    self.assertEqual(len(transport.events), 1)
                    self.assertTrue(all(connection.closed for connection in transport.connections))
            for length in (12, 80):
                identity = prefix + "_" + "a" * length
                items = [{field: identity}]
                transport = self.transport(script=[("GET", route + "?page_size=1", 200,
                                                   self.page_value(items, total=1))])
                self.assertEqual(extension.bounded_pages(extension.ExtensionBudget(deadline=10),
                                                        self.client, route), items)
                self.assertEqual(len(transport.events), 1)
            expected[operation] = {"succeeded": len(invalid) + 2, "rejected": 0, "transport_failed": 0}
        self.assert_accounting(expected)

    def test_functional_reads_accept_derived_subject_fingerprint_outside_file_hashes(self):
        """A valid derived subject digest can differ from every raw resource digest in the complete read helper."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture, transport = self.functional_fixture(root)
            derived = "d" * 64
            self.assertNotIn(derived, {row["sha256"] for row in fixture.resources})
            self.assertEqual(fixture.entries[1]["kind"], "policy-subject")
            fixture.entries[1]["fingerprint"] = derived
            before = {path.name: path.read_bytes() for path in root.iterdir()}
            result = extension.functional_reads(extension.ExtensionBudget(deadline=10), self.client,
                                                root, 7, fixture.analysis, read_only=False)
            self.assertEqual(result, fixture.view)
            self.assertEqual({path.name: path.read_bytes() for path in root.iterdir()}, before)
            self.assertTrue(all(connection.closed for connection in transport.connections))
            extension.RecordingWorkspace.ensure_accounting()

    def test_functional_reads_reject_malformed_derived_and_wrong_raw_source_fingerprints(self):
        """Derived digests retain exact Sha256Hex validation while raw source digests remain correlated with captured files."""
        faults = [(1, value) for value in (None, True, 123, [], "", "D" * 64, "d" * 63, "d" * 65)]
        faults.append((0, "e" * 64))
        for entry, fingerprint in faults:
            with self.subTest(entry=entry, fingerprint=fingerprint), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                fixture, transport = self.functional_fixture(root)
                fixture.entries[entry]["fingerprint"] = fingerprint
                before = {path.name: path.read_bytes() for path in root.iterdir()}
                with self.assertRaises(AssertionError):
                    extension.functional_reads(extension.ExtensionBudget(deadline=10), self.client,
                                               root, 7, fixture.analysis, read_only=False)
                self.assertGreater(len(transport.events), 0)
                self.assertEqual({path.name: path.read_bytes() for path in root.iterdir()}, before)
                self.assertTrue(all(connection.closed for connection in transport.connections))
        extension.RecordingWorkspace.ensure_accounting()


if __name__ == "__main__":
    unittest.main()
