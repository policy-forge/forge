#!/usr/bin/env python3
"""Synthetic maintained-client conformance over the documented local API."""
import argparse
import base64
import json
import hashlib
import copy
import math
import re
import stat
import os
import time
from urllib.parse import urlencode
from pathlib import Path
import tempfile
import uuid
import sys

sys.dont_write_bytecode = True
from verify_workspace import MAX_CLIENT_REQUESTS, atomic_receipt, contract_routes
from workspace_client import Workspace, WorkspaceError

# Limit actual request attempts across all synchronous sessions; retain only
# allowlisted contract identifiers and three integer outcome categories.
MAX_RECORDED_REQUESTS = MAX_CLIENT_REQUESTS


class RecordingWorkspace(Workspace):
    """Record documented operation identifiers and bounded outcomes only."""

    # Shared only within one synchronous run; run() resets it before its sessions.
    request_count = 0
    accounting_failed = False

    def record(self, operation, outcome):
        """Increment one allowlisted outcome without retaining request or response data."""
        if outcome not in ("succeeded", "rejected", "transport_failed"):
            RecordingWorkspace.accounting_failed = True
            raise RuntimeError("The client used an unsupported outcome category")
        try:
            self.outcomes.setdefault(operation, {
                "succeeded": 0, "rejected": 0, "transport_failed": 0,
            })[outcome] += 1
        except Exception:
            RecordingWorkspace.accounting_failed = True
            raise

    def request(self, method, path, body=None, **kwargs):
        """Forward one documented request and retain only its operation outcome."""
        try:
            operation = next((name for verb, pattern, name in self.routes
                              if verb == method and pattern.fullmatch(path.split("?", 1)[0])), None)
        except Exception:
            RecordingWorkspace.accounting_failed = True
            raise RuntimeError("The client could not classify its request") from None
        if operation is None:
            RecordingWorkspace.accounting_failed = True
            raise RuntimeError("The client used an undocumented operation")
        if (type(RecordingWorkspace.request_count) is not int or
                not 0 <= RecordingWorkspace.request_count < MAX_RECORDED_REQUESTS):
            RecordingWorkspace.accounting_failed = True
            raise RuntimeError("The client exceeded its request accounting bound")
        RecordingWorkspace.request_count += 1
        try:
            result = super().request(method, path, body, **kwargs)
        except WorkspaceError:
            self.record(operation, "rejected")
            raise
        except OSError:
            self.record(operation, "transport_failed")
            raise
        except Exception:
            # close() may swallow RuntimeError/ValueError after the server has
            # already honoured shutdown; retain a failure latch without its text.
            RecordingWorkspace.accounting_failed = True
            raise
        self.record(operation, "succeeded")
        return result


    @classmethod
    def ensure_accounting(cls):
        """Reject sticky failures and require every attempted request to have one closed outcome."""
        if cls.accounting_failed:
            raise RuntimeError("The client could not account for every request")
        try:
            if type(cls.request_count) is not int or not 0 <= cls.request_count <= MAX_RECORDED_REQUESTS:
                raise ValueError("Invalid attempt count")
            declared = {name for _, _, name in cls.routes}
            total = 0
            for operation, counts in cls.outcomes.items():
                if operation not in declared or set(counts) != {"succeeded", "rejected", "transport_failed"}:
                    raise ValueError("Invalid outcome inventory")
                if any(type(count) is not int or count < 0 for count in counts.values()):
                    raise ValueError("Invalid outcome count")
                total += sum(counts.values())
            if total != cls.request_count:
                raise ValueError("Unreconciled attempt count")
        except Exception:
            cls.accounting_failed = True
            raise RuntimeError("The client could not account for every request") from None



# Existing client /2 check names stay closed. These extension bounds govern only
# this declared tiny synthetic fixture, not arbitrary project capacity.
EXTENSION_OPERATIONS = frozenset(("checkMapping", "getApplicabilityReport", "getConversion", "getEffectPreview",
    "getProjectConfigStatus", "getProvenanceExcerpt", "getResource", "getResourceValidation", "getReviewQueueCounts",
    "getSession", "listProvenanceEntries", "listReviewQueueItems", "runValidation", "validateMappingDraft"))
REVIEW_REASONS = ("invalid-resource", "stale-input", "external-conflict", "scope-decision-required",
    "deferred-scope-decision", "no-reviewed-mapping", "reviewed-no-positive-relationship")


class ExtensionBudget:
    """Fence this fixture's at most 200 calls and 300-second pass deadline; I/O still uses the unchanged 30-second client timeout."""

    def __init__(self, deadline=None, limit=200, seconds=300):
        """Keep one monotonic absolute deadline and exact bounded attempt count, never resetting it between phases."""
        if type(limit) is not int or not 1 <= limit <= 200:
            raise ValueError("Unsupported extension bound")
        if type(seconds) not in (int, float) or not math.isfinite(seconds) or not 0 < seconds <= 300:
            raise ValueError("Unsupported extension deadline")
        if deadline is not None and (type(deadline) not in (int, float) or not math.isfinite(deadline) or deadline < 0):
            raise ValueError("Unsupported extension deadline")
        self.deadline = time.monotonic() + seconds if deadline is None else deadline
        self.limit, self.attempts = limit, 0

    def check(self):
        """Reject equality or expiry before new work and before granting pass; this does not preempt an in-flight syscall."""
        if time.monotonic() >= self.deadline:
            raise RuntimeError("The extension exceeded its absolute deadline")

    def request(self, client, method, path, body=None, **kwargs):
        """Delegate actual tracked requests only after a budget fence; late returns cannot earn extension success."""
        self.check()
        if self.attempts >= self.limit:
            raise RuntimeError("The extension exceeded its request bound")
        self.attempts += 1
        try:
            return client.request(method, path, body, **kwargs)
        finally:
            self.check()


class BudgetedClient:
    """Use the real RecordingWorkspace with deadline-bound fixture requests, never fabricated observation labels."""

    def __init__(self, client, budget, read_only):
        """Retain only an owned active client, shared budget and its explicitly requested launch mode."""
        self.client, self.budget, self.read_only = client, budget, read_only

    def request(self, method, path, body=None, **kwargs):
        """Pass method/path/body through the existing recorder and unchanged transport limits."""
        return self.budget.request(self.client, method, path, body, **kwargs)

    def commit(self, preview, *, confirmed, idempotency_key):
        """Keep exact preview confirmation and the unchanged commit body/idempotency semantics under the same budget."""
        if confirmed is not True:
            raise ValueError("Exact preview confirmation is required")
        return self.request("POST", "/api/v1/effects/commits", {
            "receipt": preview["receipt"]["token"], "observed_version": preview["target_version"], "confirmed": True,
        }, idempotency_key=idempotency_key)

    def wait(self, operation, timeout=35):
        """Poll only the accepted session-local ID with the original wait ceiling and monotonic extension fence."""
        end = min(self.budget.deadline, time.monotonic() + timeout)
        operation_id, kind = operation["operation_id"], operation["kind"]
        while operation["state"] in ("pending", "running"):
            if time.monotonic() >= end:
                raise TimeoutError("Query the retained operation before retrying its effect")
            time.sleep(min(0.1, max(0, end - time.monotonic())))
            operation = self.request("GET", "/api/v1/operations/" + operation_id)
            assert operation["operation_id"] == operation_id and operation["kind"] == kind
        assert operation["state"] in ("succeeded", "failed", "cancelled")
        self.budget.check()
        return operation


def assert_closed(value, required, optional=()):
    """Check exact response member sets before semantic assertions; optional contract members remain optional."""
    assert type(value) is dict and set(required) <= set(value) <= set(required) | set(optional)


def fixture_bytes(root):
    """Capture the entire small owned flat fixture, rejecting links/special files, excessive entries and oversized bytes."""
    rows = {}
    with os.scandir(root) as entries:
        for entry in entries:
            assert len(rows) < 16 and stat.S_ISREG(entry.stat(follow_symlinks=False).st_mode)
            with open(entry.path, "rb") as stream:
                raw = stream.read(1024 * 1024 + 1)
            assert len(raw) <= 1024 * 1024
            rows[entry.name] = raw
    return rows


def bounded_pages(budget, client, path, filters=None, limit=16):
    """Traverse page_size=1 with fixed filters/version/denominator, rejecting cycles, duplicate rows and partial success."""
    assert type(limit) is int and 1 <= limit <= 16
    primary_id = {"/api/v1/resources": "resource_id", "/api/v1/review-queue/items": "item_id", "/api/v1/provenance/entries": "entry_id"}[path]
    identity_prefix = {"resource_id": "res", "item_id": "qi", "entry_id": "prov"}[primary_id]
    query = dict(filters or {})
    assert "cursor" not in query and "page_size" not in query
    query["page_size"] = "1"
    rows, seen, cursor_seen = [], set(), set()
    version, total = None, None
    for _page in range(limit):
        value = budget.request(client, "GET", path + "?" + urlencode(query))
        assert_closed(value, ("resource_version", "page"))
        page = value["page"]
        assert_closed(page, ("items", "next_cursor", "total_matching"))
        assert type(value["resource_version"]) is str and re.fullmatch(r"[0-9a-f]{64}", value["resource_version"])
        assert type(page["items"]) is list and len(page["items"]) <= 1
        assert type(page["total_matching"]) is int and 0 <= page["total_matching"] <= 16
        if version is None:
            version, total = value["resource_version"], page["total_matching"]
        assert value["resource_version"] == version and page["total_matching"] == total
        for item in page["items"]:
            identity = item[primary_id]
            assert type(identity) is str and re.fullmatch(identity_prefix + r"_[0-9a-z]{12,80}", identity) and identity not in seen
            seen.add(identity); rows.append(item)
        cursor = page["next_cursor"]
        if cursor is None:
            assert len(rows) == total
            return rows
        assert type(cursor) is str and 1 <= len(cursor) <= 512 and cursor not in cursor_seen and page["items"]
        cursor_seen.add(cursor); query["cursor"] = cursor
    raise AssertionError("The fixture exceeded its complete page bound")


def validate_report(value):
    """Reconcile exact typed severity counts with the closed validation response, never accepting empty invalid success."""
    assert_closed(value, ("state", "error_count", "warning_count", "diagnostics"))
    assert value["state"] in ("valid", "invalid") and type(value["diagnostics"]) is list and len(value["diagnostics"]) <= 500
    for count in (value["error_count"], value["warning_count"]):
        assert type(count) is int and 0 <= count <= 500
    for row in value["diagnostics"]:
        assert_closed(row, ("code", "severity", "message"), ("resource_id", "field"))
        assert type(row["code"]) is str and 3 <= len(row["code"]) <= 100
        assert row["severity"] in ("error", "warning", "info")
        assert type(row["message"]) is str and 1 <= len(row["message"]) <= 1000
        if row.get("resource_id") is not None:
            assert type(row["resource_id"]) is str and re.fullmatch(r"res_[0-9a-z]{12,80}", row["resource_id"])
        if row.get("field") is not None:
            assert type(row["field"]) is str and len(row["field"]) <= 256
    assert value["error_count"] == sum(row["severity"] == "error" for row in value["diagnostics"])
    assert value["warning_count"] == sum(row["severity"] == "warning" for row in value["diagnostics"])
    assert value["state"] == ("invalid" if value["error_count"] else "valid")
    return value


def expect_error(budget, client, method, path, body, code):
    """Require the actual typed rejection code; unexpected exceptions and accidental success revoke conformance."""
    try:
        budget.request(client, method, path, body)
    except WorkspaceError as error:
        assert error.payload["code"] == code
        return
    raise AssertionError("The API accepted the declared negative fixture")


def retained_conversion(budget, client, operation):
    """Inspect the actual accepted conversion and unconsumed exact preview in its creating writable session."""
    assert operation["state"] == "succeeded" and operation["kind"] == "conversion"
    result = budget.request(client, "GET", "/api/v1/conversions/" + operation["operation_id"])
    assert result == operation["result"] and result["operation_id"] == operation["operation_id"]
    validate_report(result["validation"])
    preview = budget.request(client, "GET", "/api/v1/effects/previews/" + result["preview"]["preview_id"])
    assert preview == result["preview"] and preview["operation_type"] == "policy-conversion"
    assert preview["validation"]["state"] == "valid" and re.fullmatch(r"[0-9a-f]{64}", preview["exact_bytes_sha256"])
    return operation["operation_id"], preview["preview_id"]


def inspect_resource(resource, files):
    """Bind returned registered metadata to already captured owned file bytes, never opening a supplied response path."""
    assert_closed(resource, ("resource_id", "key", "role", "path", "sha256", "size_bytes", "validation_state", "stale", "version"), ("media_type",))
    assert re.fullmatch(r"res_[0-9a-z]{12,80}", resource["resource_id"])
    assert resource["path"] in files and type(resource["size_bytes"]) is int
    raw = files[resource["path"]]
    assert resource["sha256"] == resource["version"] == hashlib.sha256(raw).hexdigest()
    assert resource["size_bytes"] == len(raw) and type(resource["stale"]) is bool
    assert resource["validation_state"] in ("valid", "invalid", "stale", "not-validated")


def inspect_excerpt(value, resources, files):
    """Reconcile whole UTF-8 source lines and current resource SHA with the real excerpt identifier, without publishing text."""
    assert_closed(value, ("excerpt_id", "resource_id", "sha256", "start_line", "end_line", "text", "truncated"))
    resource = resources[value["resource_id"]]
    assert type(value["start_line"]) is int and type(value["end_line"]) is int
    lines = files[resource["path"]].decode("utf-8").splitlines(keepends=True)
    assert 1 <= value["start_line"] <= value["end_line"] <= len(lines)
    assert type(value["text"]) is str and value["text"] == "".join(lines[value["start_line"] - 1:value["end_line"]])
    assert len(value["text"].encode()) <= 20000 and value["sha256"] == resource["sha256"]
    assert type(value["truncated"]) is bool and value["truncated"] == (value["end_line"] < len(lines))


def functional_reads(budget, client, root, expected_count, expected_analysis, *, read_only, invalid_key=None):
    """Assert twelve actual read/compute routes against a complete fixed fixture in both session modes, without effects."""
    before = fixture_bytes(root)
    session = budget.request(client, "GET", "/api/v1/session")
    assert_closed(session, ("session_id", "mode", "read_only", "api_major", "contract_version", "project_label"), ("launched_at",))
    assert re.fullmatch(r"sess_[0-9a-z]{8,64}", session["session_id"])
    assert session["mode"] == "machine" and type(session["read_only"]) is bool and session["read_only"] is read_only
    assert type(session["api_major"]) is int and session["api_major"] == 1 and session["contract_version"] == "1.2.0"
    index = json.loads(before["forge.workspace.json"])
    assert session["project_label"] == index["label"]
    assert budget.request(client, "GET", "/api/v1/session") == session
    assert budget.request(client, "GET", "/api/v1/project/config-status") == {"present": False, "valid": False, "issues": []}
    resources = bounded_pages(budget, client, "/api/v1/resources")
    assert len(resources) == expected_count and [row["key"] for row in resources] == sorted(row["key"] for row in resources)
    by_id = {row["resource_id"]: row for row in resources}; by_key = {row["key"]: row for row in resources}
    assert len(by_id) == len(by_key) == expected_count == len(index["resources"])
    registered = {row["key"]: row for row in index["resources"]}
    assert set(by_key) == set(registered)
    validations = []
    for row in resources:
        inspect_resource(row, before)
        assert {key: row[key] for key in ("key", "role", "path")} == registered[row["key"]]
        assert budget.request(client, "GET", "/api/v1/resources/" + row["resource_id"]) == row
        report = validate_report(budget.request(client, "GET", "/api/v1/resources/" + row["resource_id"] + "/validation"))
        assert (report["state"] == "invalid") is (row["validation_state"] in ("invalid", "stale"))
        validations.append(report)
    selected = by_key[invalid_key or "policy"]
    selection = validate_report(budget.request(client, "POST", "/api/v1/validation/runs", {"scope": "selected", "resource_ids": [selected["resource_id"]]}))
    assert selection == validations[resources.index(selected)]
    if invalid_key:
        assert selection["state"] == "invalid" and selection["error_count"] >= 1
        assert all(row["resource_id"] == selected["resource_id"] for row in selection["diagnostics"])
    all_report = validate_report(budget.request(client, "POST", "/api/v1/validation/runs", {"scope": "all"}))
    assert all_report["error_count"] == sum(row["error_count"] for row in validations)
    assert all_report["warning_count"] == sum(row["warning_count"] for row in validations)
    assert sorted(json.dumps(row, sort_keys=True) for row in all_report["diagnostics"]) == sorted(json.dumps(issue, sort_keys=True) for row in validations for issue in row["diagnostics"])
    unknown = "res_" + "0" * 32
    assert unknown not in by_id
    expect_error(budget, client, "GET", "/api/v1/resources/" + unknown, None, "not-found")
    expect_error(budget, client, "POST", "/api/v1/validation/runs", {"scope": "selected", "resource_ids": []}, "invalid-request")
    assert validate_report(budget.request(client, "POST", "/api/v1/mapping/checks", {}))["state"] == "valid"
    mapping = budget.request(client, "GET", "/api/v1/mapping/draft")
    assert validate_report(budget.request(client, "POST", "/api/v1/mapping/draft/validation", {"manifest": mapping["manifest"]}))["state"] == "valid"
    invalid = copy.deepcopy(mapping["manifest"]); invalid["mapping"]["maps"].append(copy.deepcopy(invalid["mapping"]["maps"][0]))
    invalid_report = validate_report(budget.request(client, "POST", "/api/v1/mapping/draft/validation", {"manifest": invalid}))
    assert invalid_report["state"] == "invalid" and invalid_report["error_count"] >= 1
    view = budget.request(client, "GET", "/api/v1/applicability/report")
    assert_closed(view, ("version", "stale", "input_fingerprints", "classification_counts", "eligible_controls"))
    assert view["version"] == hashlib.sha256(before["applicability-report.json"]).hexdigest() and view["stale"] is False
    assert view["eligible_controls"] == expected_analysis["eligible_controls"] == 2
    assert view["classification_counts"] == expected_analysis["classification_counts"]
    assert sum(row["count"] for row in view["classification_counts"]) == view["eligible_controls"]
    assert view["input_fingerprints"] == [
        {"resource_id": by_key[key]["resource_id"], "sha256": by_key[key]["sha256"], "matches_current": True}
        for key in ("scope", "framework", "built-mapping")
    ]
    counts = budget.request(client, "GET", "/api/v1/review-queue/counts")
    assert_closed(counts, ("total_open", "by_reason"))
    assert type(counts["total_open"]) is int and 0 <= counts["total_open"] <= 16
    assert type(counts["by_reason"]) is list and len(counts["by_reason"]) <= 7
    for row in counts["by_reason"]:
        assert_closed(row, ("reason_code", "count"))
        assert row["reason_code"] in REVIEW_REASONS and type(row["count"]) is int and 0 <= row["count"] <= 16
    assert len({row["reason_code"] for row in counts["by_reason"]}) == len(counts["by_reason"])
    queue = bounded_pages(budget, client, "/api/v1/review-queue/items")
    assert len(queue) >= 2 and len(queue) == counts["total_open"] == sum(row["count"] for row in counts["by_reason"])
    summary = budget.request(client, "GET", "/api/v1/project/summary")
    assert summary["review_counts"]["total_open"] == len(queue) and summary["resource_counts"]["total"] == expected_count
    assert queue == sorted(queue, key=lambda row: (REVIEW_REASONS.index(row["reason_code"]), row["item_id"]))
    for reason in sorted({row["reason_code"] for row in queue}):
        assert bounded_pages(budget, client, "/api/v1/review-queue/items", {"reason_code": reason}) == [row for row in queue if row["reason_code"] == reason]
    assert {row["reason_code"]: row["count"] for row in counts["by_reason"]} == {reason: sum(row["reason_code"] == reason for row in queue) for reason in {row["reason_code"] for row in queue}}
    policy = by_key["policy"]
    entries = bounded_pages(budget, client, "/api/v1/provenance/entries", {"anchor": policy["resource_id"]})
    assert len(entries) >= 2 and [row["entry_id"] for row in entries] == sorted(row["entry_id"] for row in entries)
    assert bounded_pages(budget, client, "/api/v1/provenance/entries", {"anchor": policy["resource_id"], "kind": "source-reference"}) == [row for row in entries if row["kind"] == "source-reference"]
    for row in entries:
        if "fingerprint" in row:
            assert type(row["fingerprint"]) is str and re.fullmatch(r"[0-9a-f]{64}", row["fingerprint"])
            if row["kind"] == "source-reference":
                assert row["fingerprint"] in {item["sha256"] for item in resources}
    excerpts = sorted({reference for row in entries for reference in row.get("excerpt_refs", [])})
    assert excerpts and len(excerpts) <= 16
    for reference in excerpts:
        excerpt = budget.request(client, "GET", "/api/v1/provenance/excerpts/" + reference)
        assert excerpt["excerpt_id"] == reference
        inspect_excerpt(excerpt, by_id, before)
    expect_error(budget, client, "GET", "/api/v1/provenance/excerpts/ex_" + "0" * 32, None, "not-found")
    assert fixture_bytes(root) == before
    return view


def assert_shutdown(client):
    """Require the exact owned session's natural exit and actual post-close refusal, without publishing process state."""
    assert client.process.poll() == 0
    try:
        client.request("GET", "/api/v1/project/summary")
    except OSError:
        return
    raise AssertionError("The extension workspace answered after shutdown")


def register_extension(client, file, role, key):
    """Register only a real existing fixture through its reviewed index preview and explicit confirmed commit."""
    response = client.request("POST", "/api/v1/resources/register", {"path": file, "role": role, "key": key}, idempotency_key=str(uuid.uuid4()))
    client.commit(response["preview"], confirmed=True, idempotency_key=str(uuid.uuid4()))


def seed_extension_fixture(client, root):
    """Reuse published maintained-client producer workflows in a separate two-control fixture; no authored report or imported authority."""
    source=b"# Synthetic policy\n\n## Access\n\n- Operators must review supplied clauses.\n"
    source=source.replace(b"Synthetic policy", "Synthetic policy Ω".encode()) + ("Additional synthetic context Ω.\n" * 110).encode()
    prepared=client.request("POST","/api/v1/resources/upload",{"role":"policy-source","target_path":"policy.md","filename":"policy.md","content_base64":base64.b64encode(source).decode()},idempotency_key=str(uuid.uuid4()))
    preview=prepared["preview"]
    assert not (root/"policy.md").exists()
    key=str(uuid.uuid4())
    first=client.commit(preview,confirmed=True,idempotency_key=key)
    second=client.commit(preview,confirmed=True,idempotency_key=key)
    assert first==second and (root/"policy.md").read_bytes()==source
    def register(file,role,key):
        """Prepare and explicitly commit one documented resource registration."""
        response=client.request("POST","/api/v1/resources/register",{"path":file,"role":role,"key":key},idempotency_key=str(uuid.uuid4()))
        client.commit(response["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
    register("policy.md","policy-source","policy")
    resources=client.request("GET","/api/v1/resources")["page"]["items"]
    operation=client.wait(client.request("POST","/api/v1/conversions",{"source_resource_id":resources[0]["resource_id"],"output_kind":"oscal-catalog","target_path":"catalog.json"},idempotency_key=str(uuid.uuid4())))
    assert operation["state"]=="succeeded"
    client.commit(operation["result"]["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
    register("catalog.json","oscal-catalog-artifact","catalog")
    resources=client.request("GET","/api/v1/resources")["page"]["items"]
    catalog=next(item for item in resources if item["key"]=="catalog")
    framework={"catalog":{"uuid":"22222222-2222-4222-8222-222222222222","metadata":{"title":"Synthetic framework","last-modified":"2026-09-10T00:00:00Z","version":"1","oscal-version":"1.2.3"},"controls":[{"id":"framework-a","title":"Synthetic control"},{"id":"framework-b","title":"Second synthetic control"}]}}
    upload=client.request("POST","/api/v1/resources/upload",{"role":"oscal-catalog-artifact","target_path":"framework.json","filename":"framework.json","content_base64":base64.b64encode(json.dumps(framework).encode()).decode()},idempotency_key=str(uuid.uuid4()))
    client.commit(upload["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
    register("framework.json","oscal-catalog-artifact","framework")
    resources=client.request("GET","/api/v1/resources")["page"]["items"]
    framework=next(item for item in resources if item["key"]=="framework")
    subjects=client.request("GET","/api/v1/mapping/subjects?side=policy&resource_id="+catalog["resource_id"])["page"]["items"]
    subject=next(item for item in subjects if item["statement_count"]==0)
    mapping={"source_resource_id":catalog["resource_id"],"target_resource_id":framework["resource_id"],"target_path":"mapping.json","scope":"control-only","maps":[{"key":"reviewed-none","relationship":"no-relationship","sources":[{"type":"control","id_ref":subject["label"]}],"targets":[{"type":"control","id_ref":"framework-a"}],"reviewer_key":"reviewer","reviewed_at":"2026-09-10T00:00:00Z","rationale":"Explicit synthetic review"}],"review":{"collection":{"key":"mapping","title":"Synthetic mappings","version":"1","last_modified":"2026-09-10T00:00:00Z"},"reviewers":[{"key":"reviewer","type":"person","name":"Synthetic reviewer"}],"provenance":{"method":"human","matching_rationale":"semantic","status":"draft","mapping_description":"Explicit synthetic review","reviewer_keys":["reviewer"],"reviewed_at":"2026-09-10T00:00:00Z"}}}
    initial_map=client.request("POST","/api/v1/mapping/initializations",mapping,idempotency_key=str(uuid.uuid4()))
    client.commit(initial_map["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
    register("mapping.json","mapping-collection","mapping")
    map_draft=client.request("GET","/api/v1/mapping/draft")
    map_draft["manifest"]["mapping"]["maps"][0]["rationale"]="Updated explicit rationale"
    edit=client.request("PUT","/api/v1/mapping/draft",{"manifest":map_draft["manifest"],"observed_version":map_draft["version"]},idempotency_key=str(uuid.uuid4()))
    client.commit(edit["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
    built=client.wait(client.request("POST","/api/v1/mapping/builds",{},idempotency_key=str(uuid.uuid4())))
    assert built["state"]=="succeeded" and built["result"]["positive_relationship_count"]==0
    client.commit(built["result"]["report_preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
    register("mapping-collection.json","mapping-collection","built-mapping")
    initial=client.request("POST","/api/v1/applicability/initializations",{"framework_resource_id":framework["resource_id"],"target_path":"scope.json"},idempotency_key=str(uuid.uuid4()))
    client.commit(initial["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
    register("scope.json","applicability-manifest","scope")
    controls=client.request("GET","/api/v1/applicability/controls")["page"]["items"]
    assert controls and all(item["decision_state"]=="under-review" for item in controls)
    draft=client.request("GET","/api/v1/applicability/draft")
    manifest=draft["manifest"]
    manifest["mapping_collections"]=["mapping-collection.json"]
    manifest["reviewers"]=[{"key":"reviewer","type":"person","name":"Synthetic reviewer"}]
    manifest["decisions"]=[{"control_id":controls[0]["control_id"],"state":"applicable","reviewer_key":"reviewer","reviewed_at":"2026-09-10T00:00:00Z","rationale":"Explicit synthetic review"}]
    assert client.request("POST","/api/v1/applicability/draft/validation",{"manifest":manifest})["state"]=="valid"
    edit=client.request("PUT","/api/v1/applicability/draft",{"manifest":manifest,"observed_version":draft["version"]},idempotency_key=str(uuid.uuid4()))
    client.commit(edit["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
    analysis=client.wait(client.request("POST","/api/v1/applicability/analyses",{},idempotency_key=str(uuid.uuid4())))
    assert analysis["state"]=="succeeded"
    client.commit(analysis["result"]["report_preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
    register_extension(client, "applicability-report.json", "applicability-report", "scope-report")
    return analysis["result"]


def config_case(forge, root, budget, expected):
    """Read actual configuration status from a fresh read-only session while its stopped-phase fixture bytes stay fixed."""
    before = fixture_bytes(root)
    budget.check()
    with RecordingWorkspace(forge, root, read_only=True) as actual:
        assert budget.request(actual, "GET", "/api/v1/project/config-status") == expected
        assert fixture_bytes(root) == before
    assert_shutdown(actual)
    budget.check()


def run_functional_extension(forge, budget):
    """Exercise twelve compute/read families and targeted invalid/stale cases using owned stopped-session fixtures, preserving old check names."""
    start_attempts = RecordingWorkspace.request_count
    with tempfile.TemporaryDirectory(prefix="forge-client-extension-") as directory:
        root = Path(directory)
        budget.check()
        with RecordingWorkspace(forge, root, read_only=False) as actual:
            expected = seed_extension_fixture(BudgetedClient(actual, budget, False), root)
            functional_reads(budget, actual, root, 7, expected, read_only=False)
        assert_shutdown(actual)
        budget.check()
        with RecordingWorkspace(forge, root, read_only=True) as actual:
            functional_reads(budget, actual, root, 7, expected, read_only=True)
        assert_shutdown(actual)
        # Register an ordinary valid extra source first; invalid bytes are a
        # deliberate owned-fixture mutation only after its session has stopped.
        budget.check()
        with RecordingWorkspace(forge, root, read_only=False) as actual:
            client = BudgetedClient(actual, budget, False)
            raw = b"# Extra synthetic policy\n\n## Boundary\n\n- Review the supplied input.\n"
            uploaded = client.request("POST", "/api/v1/resources/upload", {"role": "policy-source", "target_path": "invalid-policy.md", "filename": "invalid-policy.md", "content_base64": base64.b64encode(raw).decode()}, idempotency_key=str(uuid.uuid4()))
            client.commit(uploaded["preview"], confirmed=True, idempotency_key=str(uuid.uuid4()))
            register_extension(client, "invalid-policy.md", "policy-source", "invalid-policy")
        assert_shutdown(actual)
        (root / "invalid-policy.md").write_bytes(b"\xff\xfe invalid synthetic policy\n")
        before = fixture_bytes(root)
        budget.check()
        with RecordingWorkspace(forge, root, read_only=True) as actual:
            resources = bounded_pages(budget, actual, "/api/v1/resources")
            assert len(resources) == 8
            item = next(row for row in resources if row["key"] == "invalid-policy")
            assert item["validation_state"] == "invalid"
            inspect_resource(budget.request(actual, "GET", "/api/v1/resources/" + item["resource_id"]), before)
            report = validate_report(budget.request(actual, "GET", "/api/v1/resources/" + item["resource_id"] + "/validation"))
            assert report["state"] == "invalid" and report["error_count"] == 1
            assert report["diagnostics"][0]["resource_id"] == item["resource_id"]
            selected = validate_report(budget.request(actual, "POST", "/api/v1/validation/runs", {"scope": "selected", "resource_ids": [item["resource_id"]]}))
            assert selected == report
            aggregate = validate_report(budget.request(actual, "POST", "/api/v1/validation/runs", {"scope": "all"}))
            assert aggregate["state"] == "invalid" and aggregate["error_count"] >= 1
            queue = bounded_pages(budget, actual, "/api/v1/review-queue/items", {"resource_id": item["resource_id"]})
            assert len(queue) == 1 and queue[0]["reason_code"] == "invalid-resource"
            assert fixture_bytes(root) == before
        assert_shutdown(actual)
        # These fixed minimal TOML cases are owned data, not producer output;
        # they are written before startup and queried using the project root.
        (root / ".forge.toml").write_bytes(b"schema-version = 1\n")
        config_case(forge, root, budget, {"present": True, "valid": True, "issues": []})
        (root / ".forge.toml").write_bytes(b"schema-version = 2\n")
        config_case(forge, root, budget, {"present": True, "valid": False, "issues": ["The project configuration is invalid."]})
        (root / ".forge.toml").unlink()
        # Alter one valid recorded input without regenerating the committed
        # report; it remains a historical report with an explicit stale match.
        scope_before = (root / "scope.json").read_bytes()
        manifest = json.loads(scope_before)
        manifest["decisions"][0]["rationale"] = "Changed synthetic rationale after report generation"
        (root / "scope.json").write_bytes((json.dumps(manifest, ensure_ascii=False, indent=2) + "\n").encode())
        before = fixture_bytes(root)
        budget.check()
        with RecordingWorkspace(forge, root, read_only=True) as actual:
            resources = bounded_pages(budget, actual, "/api/v1/resources")
            scope = next(row for row in resources if row["key"] == "scope")
            view = budget.request(actual, "GET", "/api/v1/applicability/report")
            assert view["stale"] is True and view["version"] == hashlib.sha256(before["applicability-report.json"]).hexdigest()
            assert view["input_fingerprints"] == [{"resource_id": scope["resource_id"], "sha256": hashlib.sha256(scope_before).hexdigest(), "matches_current": False}]
            assert view["classification_counts"] == expected["classification_counts"] and view["eligible_controls"] == 2
            assert fixture_bytes(root) == before
        assert_shutdown(actual)
    budget.check()
    # Actual recorder deltas include all seven extra shutdown calls and the
    # seven post-close transport probes. Cleanup is bounded and never omitted.
    assert 0 < RecordingWorkspace.request_count - start_attempts <= 200


def run(forge):
    """Run all existing workflows and publish only complete assertion-group accounting."""
    if not __debug__:
        raise RuntimeError("Conformance assertions require Python without optimization")
    checks = []
    RecordingWorkspace.routes = contract_routes(Path(__file__).resolve().parents[1])
    RecordingWorkspace.outcomes = {}
    RecordingWorkspace.request_count = 0
    RecordingWorkspace.accounting_failed = False

    with tempfile.TemporaryDirectory(prefix="forge-client-") as directory:
        root=Path(directory)
        with RecordingWorkspace(forge,root,read_only=False) as client:
            source=b"# Synthetic policy\n\n## Access\n\n- Operators must review supplied clauses.\n"
            prepared=client.request("POST","/api/v1/resources/upload",{"role":"policy-source","target_path":"policy.md","filename":"policy.md","content_base64":base64.b64encode(source).decode()},idempotency_key=str(uuid.uuid4()))
            preview=prepared["preview"]
            assert not (root/"policy.md").exists()
            checks.append("upload_requires_confirmation")
            key=str(uuid.uuid4())
            first=client.commit(preview,confirmed=True,idempotency_key=key)
            second=client.commit(preview,confirmed=True,idempotency_key=key)
            assert first==second and (root/"policy.md").read_bytes()==source
            checks.append("idempotent_commit_replays_exact_bytes")
            def register(file,role,key):
                """Prepare and explicitly commit one documented resource registration."""
                response=client.request("POST","/api/v1/resources/register",{"path":file,"role":role,"key":key},idempotency_key=str(uuid.uuid4()))
                client.commit(response["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
            register("policy.md","policy-source","policy")
            resources=client.request("GET","/api/v1/resources")["page"]["items"]
            operation=client.wait(client.request("POST","/api/v1/conversions",{"source_resource_id":resources[0]["resource_id"],"output_kind":"oscal-catalog","target_path":"catalog.json"},idempotency_key=str(uuid.uuid4())))
            assert operation["state"]=="succeeded"
            extension_budget = ExtensionBudget()
            retained_ids = retained_conversion(extension_budget, client, operation)
            checks.append("conversion_succeeds")
            client.commit(operation["result"]["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
            register("catalog.json","oscal-catalog-artifact","catalog")
            resources=client.request("GET","/api/v1/resources")["page"]["items"]
            catalog=next(item for item in resources if item["key"]=="catalog")
            framework={"catalog":{"uuid":"22222222-2222-4222-8222-222222222222","metadata":{"title":"Synthetic framework","last-modified":"2026-09-10T00:00:00Z","version":"1","oscal-version":"1.2.3"},"controls":[{"id":"framework-a","title":"Synthetic control"}]}}
            upload=client.request("POST","/api/v1/resources/upload",{"role":"oscal-catalog-artifact","target_path":"framework.json","filename":"framework.json","content_base64":base64.b64encode(json.dumps(framework).encode()).decode()},idempotency_key=str(uuid.uuid4()))
            client.commit(upload["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
            register("framework.json","oscal-catalog-artifact","framework")
            resources=client.request("GET","/api/v1/resources")["page"]["items"]
            framework=next(item for item in resources if item["key"]=="framework")
            subjects=client.request("GET","/api/v1/mapping/subjects?side=policy&resource_id="+catalog["resource_id"])["page"]["items"]
            subject=next(item for item in subjects if item["statement_count"]==0)
            mapping={"source_resource_id":catalog["resource_id"],"target_resource_id":framework["resource_id"],"target_path":"mapping.json","scope":"control-only","maps":[{"key":"reviewed-none","relationship":"no-relationship","sources":[{"type":"control","id_ref":subject["label"]}],"targets":[{"type":"control","id_ref":"framework-a"}],"reviewer_key":"reviewer","reviewed_at":"2026-09-10T00:00:00Z","rationale":"Explicit synthetic review"}],"review":{"collection":{"key":"mapping","title":"Synthetic mappings","version":"1","last_modified":"2026-09-10T00:00:00Z"},"reviewers":[{"key":"reviewer","type":"person","name":"Synthetic reviewer"}],"provenance":{"method":"human","matching_rationale":"semantic","status":"draft","mapping_description":"Explicit synthetic review","reviewer_keys":["reviewer"],"reviewed_at":"2026-09-10T00:00:00Z"}}}
            initial_map=client.request("POST","/api/v1/mapping/initializations",mapping,idempotency_key=str(uuid.uuid4()))
            client.commit(initial_map["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
            register("mapping.json","mapping-collection","mapping")
            map_draft=client.request("GET","/api/v1/mapping/draft")
            map_draft["manifest"]["mapping"]["maps"][0]["rationale"]="Updated explicit rationale"
            edit=client.request("PUT","/api/v1/mapping/draft",{"manifest":map_draft["manifest"],"observed_version":map_draft["version"]},idempotency_key=str(uuid.uuid4()))
            client.commit(edit["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
            built=client.wait(client.request("POST","/api/v1/mapping/builds",{},idempotency_key=str(uuid.uuid4())))
            assert built["state"]=="succeeded" and built["result"]["positive_relationship_count"]==0
            checks.append("explicit_no_relationship_is_preserved")
            client.commit(built["result"]["report_preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
            register("mapping-collection.json","mapping-collection","built-mapping")
            initial=client.request("POST","/api/v1/applicability/initializations",{"framework_resource_id":framework["resource_id"],"target_path":"scope.json"},idempotency_key=str(uuid.uuid4()))
            client.commit(initial["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
            register("scope.json","applicability-manifest","scope")
            controls=client.request("GET","/api/v1/applicability/controls")["page"]["items"]
            assert controls and all(item["decision_state"]=="under-review" for item in controls)
            checks.append("omitted_scope_decisions_remain_under_review")
            draft=client.request("GET","/api/v1/applicability/draft")
            manifest=draft["manifest"]
            manifest["mapping_collections"]=["mapping-collection.json"]
            manifest["reviewers"]=[{"key":"reviewer","type":"person","name":"Synthetic reviewer"}]
            manifest["decisions"]=[{"control_id":controls[0]["control_id"],"state":"applicable","reviewer_key":"reviewer","reviewed_at":"2026-09-10T00:00:00Z","rationale":"Explicit synthetic review"}]
            assert client.request("POST","/api/v1/applicability/draft/validation",{"manifest":manifest})["state"]=="valid"
            checks.append("explicit_scope_decision_validates")
            edit=client.request("PUT","/api/v1/applicability/draft",{"manifest":manifest,"observed_version":draft["version"]},idempotency_key=str(uuid.uuid4()))
            client.commit(edit["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
            analysis=client.wait(client.request("POST","/api/v1/applicability/analyses",{},idempotency_key=str(uuid.uuid4())))
            assert analysis["state"]=="succeeded"
            checks.append("committed_scope_analysis_succeeds")
            client.commit(analysis["result"]["report_preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
            export=client.wait(client.request("POST","/api/v1/exports",{"report_kind":"trace","target_path":"report.html"},idempotency_key=str(uuid.uuid4())))
            client.commit(export["result"]["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
            downloaded=client.request("GET","/api/v1/exports/"+export["operation_id"]+"/download",raw=True)
            assert downloaded==(root/"report.html").read_bytes() and b"Synthetic reviewer" not in downloaded
            checks.append("export_matches_committed_redacted_bytes")
        writer=client
        with RecordingWorkspace(forge,root,read_only=True) as client:
            expect_error(extension_budget, client, "GET", "/api/v1/conversions/" + retained_ids[0], None, "not-found")
            expect_error(extension_budget, client, "GET", "/api/v1/effects/previews/" + retained_ids[1], None, "not-found")
            assert client.request("GET","/api/v1/project/summary")["resource_counts"]["total"]==6
            checks.append("read_only_summary_reconciles_resources")
            before_bundle_queries={path.name:path.read_bytes() for path in root.iterdir() if path.is_file()}
            bundle_preview=client.bundle_preview()
            bundle=bundle_preview["bundle"]
            assert bundle_preview["source_index_present"] is True
            assert bundle_preview["source_content_included"] is False
            assert bundle["schema_version"]=="forge.workspace-index-bundle/1"
            assert bundle["content_profile"]=="index-and-hashes"
            assert len(bundle["pins"])==len(bundle["index"]["resources"])==6
            checks.append("metadata_bundle_preview_preserves_complete_denominator")
            normalized_index={"schema_version":bundle["index"]["schema_version"],"label":bundle["index"]["label"],"resources":[{"key":item["key"],"role":item["role"],"path":item["path"]} for item in bundle["index"]["resources"]]}
            normalized_bytes=(json.dumps(normalized_index,ensure_ascii=False,indent=2)+"\n").encode()
            assert hashlib.sha256(normalized_bytes).hexdigest()==bundle["index_sha256"]
            for registration,pin in zip(bundle["index"]["resources"],bundle["pins"]):
                resource_bytes=(root/registration["path"]).read_bytes()
                assert pin["key"]==registration["key"]
                assert pin["sha256"]==hashlib.sha256(resource_bytes).hexdigest()
                assert pin["size_bytes"]==len(resource_bytes)
            checks.append("metadata_bundle_fingerprints_match_exact_bytes")
            verified=client.verify_bundle(bundle)
            assert verified["scope"]=="registered-fingerprints-only" and verified["state"]=="matched"
            assert verified["expected_resources"]==verified["matched_resources"]==6
            assert verified["unregistered_resources"]==verified["mismatched_resources"]==0
            assert verified["current_resources"]==6 and verified["current_only_resources"]==0
            assert verified["expected_index_matches_current"] is True
            assert [item["key"] for item in verified["items"]]==[item["key"] for item in bundle["index"]["resources"]]
            assert verified["source_content_included"] is False
            checks.append("registered_bundle_comparison_reconciles_expected_current")
            assert {path.name:path.read_bytes() for path in root.iterdir() if path.is_file()}==before_bundle_queries
            checks.append("metadata_bundle_queries_preserve_workspace_files")
            try:
                client.request("POST","/api/v1/resources/register",{"path":"report.html","role":"trace-report","key":"report"},idempotency_key=str(uuid.uuid4()))
            except WorkspaceError as error:
                assert error.payload["code"]=="read-only-session"
                checks.append("read_only_mutation_is_rejected")
            else:
                raise AssertionError("Read-only session accepted a write")
        reader=client
        # Workspace.close() swallows shutdown failures, so prove each session really
        # ended: a clean exit code means the shutdown route was honoured rather than
        # the close() timeout killing the process, and nothing may still answer.
        for session in (writer,reader):
            assert session.process.poll()==0, f"Workspace did not shut down cleanly: exit {session.process.poll()}"
            try:
                session.request("GET","/api/v1/project/summary")
            except OSError:
                checks.append("clean_shutdown_" + ("writable" if session is writer else "read_only"))
                continue
            except WorkspaceError:
                raise AssertionError("The workspace answered after close()") from None
            raise AssertionError("The workspace answered after close()")

    run_functional_extension(forge, extension_budget)
    RecordingWorkspace.ensure_accounting()
    assert EXTENSION_OPERATIONS <= set(RecordingWorkspace.outcomes)
    assert all(RecordingWorkspace.outcomes[name]["succeeded"] > 0 for name in EXTENSION_OPERATIONS)
    assert {name for _, _, name in RecordingWorkspace.routes} - set(RecordingWorkspace.outcomes) == {"cancelOperation", "unlockSession"}
    observed = sorted(RecordingWorkspace.outcomes)
    declared = sorted(name for _, _, name in RecordingWorkspace.routes)
    return {
        "schema_version": "forge.workspace-client-verification/2",
        "status": "passed",
        "checks": checks,
        "check_count": len(checks),
        "declared_operation_count": len(declared),
        "observed_operations": observed,
        "unobserved_operations": sorted(set(declared) - set(observed)),
        "operation_outcomes": RecordingWorkspace.outcomes,
    }


def main():
    """Run conformance and atomically retain an optional redacted receipt without replacement."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--forge", required=True)
    parser.add_argument("--receipt", type=Path)
    args = parser.parse_args()
    # Existing destinations, including dangling symlinks, are preserved before
    # executing the workflow. Atomic hard-link publication also closes the race.
    if args.receipt and (args.receipt.exists() or args.receipt.is_symlink()):
        print("Maintained headless client receipt destination already exists.", file=sys.stderr)
        return 1
    try:
        receipt = run(args.forge)
    except Exception:
        if args.receipt:
            try:
                atomic_receipt(args.receipt, {
                    "schema_version": "forge.workspace-client-verification/2",
                    "status": "failed", "checks": [], "check_count": 0,
                    "failure": "client-conformance-failed",
                })
            except Exception:
                # Publication failures never expose native paths or exception
                # payloads. Exit status remains failed without a success receipt.
                pass
        print("Maintained headless client conformance failed.", file=sys.stderr)
        return 1
    if args.receipt:
        try:
            atomic_receipt(args.receipt, receipt)
        except Exception:
            print("Maintained headless client receipt publication failed.", file=sys.stderr)
            return 1
    print(f"Maintained headless client conformance passed ({receipt['check_count']} explicit check groups).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
