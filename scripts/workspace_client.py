#!/usr/bin/env python3
"""Maintained local API client; no browser bridge and no persisted capability.

Import Workspace for scripted workflows. Run this module for a read-only
summary: python3 scripts/workspace_client.py --forge ./target/debug/forge --project .
"""
import argparse
import http.client
import hashlib
import json
import queue
import os
import stat
import re
import subprocess
import threading
import time
from calendar import monthrange
from datetime import datetime
from pathlib import Path
from urllib.parse import quote, urlencode, urlsplit

MAX_RESPONSE = 4 * 1024 * 1024
# The unchanged1MiB request leaves147 bytes for the explicit source import envelope.
MAX_SOURCE_BUNDLE = 1_048_429
# Separate explicit2.4 transport limits; the inline147-byte envelope remains unchanged.
MAX_STAGED_SOURCE_BUNDLE = 10 * 1024 * 1024
STAGED_PART_BYTES = 32768
MAX_STAGED_PARTS = 320
SOURCE_LEGACY_ROLES = frozenset(("policy-source", "oscal-catalog-artifact", "oscal-component-artifact",
    "mapping-collection", "applicability-manifest", "applicability-report", "trace-report"))
SOURCE_ROLES = frozenset(("policy-source", "oscal-catalog-artifact", "oscal-component-artifact",
    "mapping-collection", "applicability-manifest", "applicability-report", "trace-report",
    "lifecycle-record", "lifecycle-source", "oscal-profile-artifact", "oscal-ssp-artifact",
    "framework-impact-manifest", "successor-map", "framework-impact-report", "framework-impact-dispositions"))


class SourceRestoreUncertain(RuntimeError):
    """Retain a nonauthorizing preknown lookup ID after an unverified commit reply."""
    def __init__(self, operation_id):
        """Expose only the stable ID; never persist a receipt or infer that no write occurred."""
        self.operation_id = operation_id
        super().__init__("The restore reply is unverified. Query the retained operation ID before any retry.")


class WorkspaceError(RuntimeError):
    """Expose the server's bounded typed error without retaining session credentials."""
    def __init__(self, payload):
        """Keep the received error envelope and its safe code/message display."""
        self.payload = payload
        super().__init__(f"{payload.get('code', 'request-failed')}: {payload.get('message', 'Request failed.')}")


class Workspace:
    """Own one loopback workspace process and explicitly negotiated API namespace."""
    def __init__(self, forge, project, read_only=True, *, api_major=1):
        """Launch one major; retain a matching numeric API2 descriptor and Session version."""
        if type(api_major) is not int or api_major not in (1, 2):
            raise ValueError("Select API major 1 or 2 explicitly")
        self._api_major = api_major
        self._contract_version = None
        self._api_prefix = "/api/v" + str(api_major)
        args = [str(Path(forge).resolve()), "workspace", "--project", str(Path(project).resolve()), "--machine-session"]
        if api_major == 2:
            args.extend(("--api-major", "2"))
        if read_only:
            args.append("--read-only")
        self.process = subprocess.Popen(args, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        self._capability = ""
        self._next_request_at = 0.0
        channel = queue.Queue(maxsize=1)
        def read_descriptor():
            """Read only the bounded first stdout line into the private bootstrap channel."""
            channel.put(self.process.stdout.readline(8193))
        threading.Thread(target=read_descriptor, daemon=True).start()
        try:
            raw = channel.get(timeout=20)
            if len(raw) > 8192:
                raise ValueError("Invalid machine descriptor")
            descriptor = json.loads(raw)
            url = urlsplit(descriptor["base_url"])
            if (url.scheme != "http" or url.hostname != "127.0.0.1" or not url.port or url.path not in ("", "/") or url.query or url.fragment or url.username or url.password
                or descriptor["mode"] != "machine" or descriptor["read_only"] is not read_only or str(descriptor["api_version"]).split(".")[0] != str(api_major)
                or not re.fullmatch(r"[0-9a-f]{64}", descriptor["capability"])):
                raise ValueError("Unsupported machine descriptor")
            if api_major == 2:
                required = {"base_url", "api_version", "api_major", "session_id", "capability", "mode", "read_only", "pid"}
                if (not required.issubset(descriptor) or type(descriptor["api_major"]) is not int or descriptor["api_major"] != 2
                    or not isinstance(descriptor["api_version"], str) or len(descriptor["api_version"]) > 64
                    or not re.fullmatch(r"2\.[0-9]+\.[0-9]+", descriptor["api_version"])
                    or type(descriptor["pid"]) is not int or descriptor["pid"] <= 0
                    or not isinstance(descriptor["session_id"], str) or not re.fullmatch(r"sess_[0-9a-z]{8,64}", descriptor["session_id"])):
                    raise ValueError("Unsupported machine descriptor")
            self._port = url.port
            self._capability = descriptor["capability"]
            if api_major == 2:
                session = self.request("GET", self.api_path("/session"))
                if (not isinstance(session, dict) or type(session.get("api_major")) is not int or session["api_major"] != 2
                    or session.get("contract_version") != descriptor["api_version"] or session.get("session_id") != descriptor["session_id"]
                    or session.get("mode") != "machine" or session.get("read_only") is not read_only):
                    raise ValueError("Unsupported selected API session")
                self._contract_version = session["contract_version"]
        except Exception:
            self.close()
            raise RuntimeError("The local workspace could not be started.") from None

    def api_path(self, path):
        """Build an explicitly selected route; never translate another public API-major path."""
        if (not isinstance(path, str) or not path.startswith("/") or path.startswith(("//", "/api/"))
            or "#" in path or any(ord(char) < 32 for char in path)):
            raise ValueError("Use a documented relative API operation path")
        return self._api_prefix + path

    def request(self, method, path, body=None, *, idempotency_key=None, raw=False, raw_json_body=None, expected_media_type=None):
        """Preserve negotiated transport; raw S6 import bytes and JSON downloads use narrow gates."""
        if not path.startswith(self._api_prefix + "/") or "#" in path or any(ord(char) < 32 for char in path):
            raise ValueError("Use a documented route for the selected API major")
        if raw_json_body is not None:
            metadata = self._contract_version in ("2.2.0", "2.3.0", "2.4.0") and path == self.api_path("/project/bundle-imports")
            source = self._contract_version in ("2.3.0", "2.4.0") and path == self.api_path("/project/source-bundle-imports")
            if (self._api_major != 2 or method != "POST" or not (metadata or source) or body is not None
                or type(raw_json_body) is not bytes or len(raw_json_body) > 1024 * 1024 or not idempotency_key):
                raise ValueError("Unsupported raw metadata import request")
        if expected_media_type is not None:
            metadata = self._contract_version in ("2.2.0", "2.3.0", "2.4.0") and re.fullmatch(
                re.escape(self._api_prefix) + r"/project/bundle-exports/op_[0-9a-z]{12,80}/download", path)
            source = self._contract_version in ("2.3.0", "2.4.0") and re.fullmatch(
                re.escape(self._api_prefix) + r"/project/source-bundle-exports/op_[0-9a-z]{12,80}/download", path)
            if (expected_media_type != "application/json" or not raw or method != "GET" or self._api_major != 2
                or not (metadata or source) or body is not None or idempotency_key):
                raise ValueError("Unsupported metadata download request")
        # This synchronous client spaces calls below the documented session rate.
        delay = self._next_request_at - time.monotonic()
        if delay > 0:
            time.sleep(delay)
        self._next_request_at = time.monotonic() + 0.06
        headers = {"Authorization": "Bearer " + self._capability, "Accept": "application/json"}
        encoded = None
        if method != "GET":
            headers["Content-Type"] = "application/json"
            encoded = raw_json_body if raw_json_body is not None else json.dumps({} if body is None else body, ensure_ascii=False, allow_nan=False).encode()
            if len(encoded) > 14 * 1024 * 1024:
                raise ValueError("Request exceeds the supported bound")
        if idempotency_key:
            headers["Idempotency-Key"] = idempotency_key
        connection = http.client.HTTPConnection("127.0.0.1", self._port, timeout=30)
        try:
            connection.request(method, path, body=encoded, headers=headers)
            response = connection.getresponse()
            response_bound = 1024 * 1024 if expected_media_type else MAX_RESPONSE
            payload = response.read(response_bound + 1)
            if len(payload) > response_bound:
                raise RuntimeError("Response exceeds the supported bound")
            if not 200 <= response.status < 300:
                raise WorkspaceError(json.loads(payload))
            if expected_media_type and response.getheader("Content-Type", "").split(";", 1)[0].strip().lower() != expected_media_type:
                raise RuntimeError("The committed metadata has an unsupported media type")
            return payload if raw else json.loads(payload)
        finally:
            connection.close()

    def bundle_preview(self):
        """Inspect the negotiated major's complete version-paired index fingerprints.

        Metadata labels, keys, paths and hashes can be sensitive. This returns
        no source bytes, exported file, receipt or approval. An absent index
        raises the server's typed not-found error.
        """
        return self.request("GET", self.api_path("/project/bundle-preview"))

    def verify_bundle(self, bundle):
        """Compare version-paired metadata only with current registered captures.

        The complete encoded request must fit the server's 1 MiB bound.
        On v2, a supplied bundle1 keeps its original index1 meaning even when
        current registrations use index2; extras and index equality are separate.
        Unregistered supplied paths are never opened. Matching bytes establish
        neither current domain validity nor import readiness; retries capture
        current state again and create no effect or replay receipt.
        """
        return self.request("POST", self.api_path("/project/bundle-verifications"), {"bundle": bundle})

    def _s3_resource(self, resource_id):
        """Accept only a registered opaque resource identifier for a selected S3 read."""
        if not isinstance(resource_id, str) or not re.fullmatch(r"res_[0-9a-z]{12,80}", resource_id):
            raise ValueError("Select an exact registered resource identifier")
        return quote(resource_id, safe="")

    def _s3_read(self, path, *, required_date=False, **query):
        """Issue one bounded API2-only GET; never fall back to API1 or create an effect.

        Query values are explicit, encoded once and checked before the existing
        paced transport. Dates are supplied by the caller; no clock default is
        introduced. Each cursor belongs to its unchanged capture/filter/date.
        The server retains whole-domain validation and count authority.
        """
        if self._api_major != 2 or self._contract_version not in ("2.1.0", "2.2.0", "2.3.0", "2.4.0"):
            raise ValueError("Lifecycle and impact reads require an explicitly negotiated API2 2.1.0, 2.2.0, 2.3.0 or 2.4.0 session")
        if required_date and query.get("as_of") is None:
            raise ValueError("An explicit as_of date is required")
        values = {}
        for name, value in query.items():
            if value is None:
                continue
            if name == "page_size":
                if type(value) is not int or not 1 <= value <= 200:
                    raise ValueError("Use an integer page_size from 1 to 200")
            elif not isinstance(value, str) or not value or len(value.encode("utf-8")) > (256 if name == "cursor" else 65536):
                raise ValueError("Use nonempty bounded query strings")
            if name == "as_of":
                if not re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}", value):
                    raise ValueError("Use an explicit YYYY-MM-DD date")
                try:
                    year, month, day = (int(part) for part in value.split("-"))
                    if not 1 <= day <= monthrange(year, month)[1]:
                        raise ValueError("Invalid day")
                except ValueError:
                    raise ValueError("Use an explicit calendar date") from None
            values[name] = value
        suffix = "?" + urlencode(values) if values else ""
        return self.request("GET", self.api_path(path) + suffix)

    def lifecycle_records(self, *, as_of=None, owner=None, state=None, page_size=50, cursor=None):
        """Read recorded inventory; derived status requires a caller-supplied date.

        Counts distinguish registered, matching and unavailable records. A next
        cursor must retain this exact date/filter tuple and page size; a conflict
        requires a new first-page read. Returned declarations are not approval.
        """
        return self._s3_read("/lifecycle/records", as_of=as_of, owner=owner, state=state,
                             page_size=page_size, cursor=cursor)

    def lifecycle_record(self, record_id, *, as_of):
        """Read one registered record's captured status at an explicit date.

        Redacted fingerprints, declared owners and blockers do not authenticate
        actors or authorize a transition. Impact references remain unresolved.
        """
        return self._s3_read("/lifecycle/records/" + self._s3_resource(record_id),
                             required_date=True, as_of=as_of)

    def lifecycle_history(self, record_id, *, page_size=50, cursor=None):
        """Read bounded recorded transition metadata without claiming current freshness."""
        return self._s3_read("/lifecycle/records/" + self._s3_resource(record_id) + "/history",
                             page_size=page_size, cursor=cursor)

    def lifecycle_queue(self, *, as_of, owner=None, page_size=50, cursor=None):
        """Read owner placements at an explicit date, preserving distinct record/group counts."""
        return self._s3_read("/lifecycle/queue", required_date=True, as_of=as_of, owner=owner,
                             page_size=page_size, cursor=cursor)

    def framework_impact_comparisons(self, *, page_size=50, cursor=None):
        """Read declared old/new comparison metadata; inventory is not a computed analysis."""
        return self._s3_read("/framework-impact/comparisons", page_size=page_size, cursor=cursor)

    def framework_impact_comparison(self, comparison_id):
        """Read the captured comparison's complete unfiltered summary and redacted provenance."""
        return self._s3_read("/framework-impact/comparisons/" + self._s3_resource(comparison_id))

    def framework_impact_changes(self, comparison_id, *, change_class=None, page_size=50, cursor=None):
        """Read bounded change rows; matching rows do not replace the complete change count."""
        return self._s3_read("/framework-impact/comparisons/" + self._s3_resource(comparison_id) + "/changes",
                             change_class=change_class, page_size=page_size, cursor=cursor)

    def framework_impact_findings(self, comparison_id, *, group=None, decision_state=None,
                                  policy_source=None, priority=None, owner=None, page_size=50, cursor=None):
        """Read exact AND-filtered findings within one old/new pair and capture context.

        The full summary and gates remain unfiltered; matched finding and emitted
        disposition counts have distinct scopes. No mutation or approval occurs.
        """
        return self._s3_read("/framework-impact/comparisons/" + self._s3_resource(comparison_id) + "/findings",
                             group=group, decision_state=decision_state, policy_source=policy_source,
                             priority=priority, owner=owner, page_size=page_size, cursor=cursor)

    def framework_impact_prior_dispositions(self, comparison_id, *, page_size=50, cursor=None):
        """Read prior-only disposition metadata; limited admission is not current finding coverage."""
        return self._s3_read("/framework-impact/comparisons/" + self._s3_resource(comparison_id) + "/prior-dispositions",
                             page_size=page_size, cursor=cursor)

    def _s6_supported(self):
        """Gate metadata effects to2.2/2.3/2.4 while preserving numeric-major bootstrap and older reads."""
        if self._api_major != 2 or self._contract_version not in ("2.2.0", "2.3.0", "2.4.0"):
            raise ValueError("Metadata bundle effects require API2 contract2.2.0,2.3.0 or2.4.0")

    def prepare_bundle_export(self, target_path, *, acknowledge_sensitive_metadata, idempotency_key):
        """Prepare an explicit metadata output; returned export work is not a confirmed write."""
        self._s6_supported()
        if acknowledge_sensitive_metadata is not True or not idempotency_key:
            raise ValueError("Sensitive metadata acknowledgment and an idempotency key are required")
        return self.request("POST", self.api_path("/project/bundle-exports"), {
            "target_path": target_path, "acknowledge_sensitive_metadata": True,
        }, idempotency_key=idempotency_key)

    def prepare_bundle_import(self, bundle_bytes, *, target_index_schema_version, acknowledge_index_replacement, idempotency_key):
        """Wrap original bundle bytes for a confirmed index-only replacement, never restoring sources."""
        self._s6_supported()
        if (type(target_index_schema_version) is not int or target_index_schema_version not in (1, 2)
            or acknowledge_index_replacement is not True or not idempotency_key or type(bundle_bytes) is not bytes):
            raise ValueError("Choose a numeric index schema and acknowledge exact index replacement")
        prefix = b'{"bundle":'
        suffix = (',"target_index_schema_version":' + str(target_index_schema_version)
                  + ',"acknowledge_index_replacement":true}').encode("ascii")
        if len(bundle_bytes) > 1024 * 1024 - len(prefix) - len(suffix):
            raise ValueError("Chosen bundle and its complete import wrapper must fit 1MiB")
        return self.request("POST", self.api_path("/project/bundle-imports"),
                            idempotency_key=idempotency_key, raw_json_body=prefix + bundle_bytes + suffix)

    def download_bundle_export(self, operation_id, *, expected_sha256):
        """Return exact committed JSON bytes after typed media, size and expected-hash checks."""
        self._s6_supported()
        if (not isinstance(operation_id, str) or not re.fullmatch(r"op_[0-9a-z]{12,80}", operation_id)
            or not isinstance(expected_sha256, str) or not re.fullmatch(r"[a-f0-9]{64}", expected_sha256)):
            raise ValueError("A prepared export identity and exact SHA256 are required")
        raw = self.request("GET", self.api_path("/project/bundle-exports/" + operation_id + "/download"),
                           raw=True, expected_media_type="application/json")
        if hashlib.sha256(raw).hexdigest() != expected_sha256:
            raise RuntimeError("The committed metadata bytes do not match the prepared hash")
        return raw

    def _source_supported(self):
        """Require explicit inline source API2.3/2.4; never translate older namespaces or feature gates."""
        if self._api_major != 2 or self._contract_version not in ("2.3.0", "2.4.0"):
            raise ValueError("Source bundle effects require API2 contract2.3.0 or2.4.0")

    @staticmethod
    def _source_closed(value, required, optional=()):
        """Validate exact object keys without trusting arbitrary mapping subclasses or extras."""
        if type(value) is not dict or not set(required).issubset(value) or set(value) - set(required) - set(optional):
            raise RuntimeError("The source restore response has an unsupported object shape")

    @staticmethod
    def _source_id(value, prefix):
        """Return one exact opaque lookup/preview identifier without normalization."""
        if type(value) is not str or not re.fullmatch(re.escape(prefix) + r"_[0-9a-z]{12,80}", value):
            raise ValueError("Use an exact source restore identifier")
        return value

    @staticmethod
    def _source_hash(value):
        """Require an exact lowercase SHA256 before comparing any restored or downloaded bytes."""
        if type(value) is not str or not re.fullmatch(r"[a-f0-9]{64}", value):
            raise RuntimeError("The source restore hash is unsupported")
        return value

    @staticmethod
    def _source_path(value):
        """Admit bounded project-relative lexical paths; server retains native confinement authority."""
        if (type(value) is not str or not 1 <= len(value) <= 512
            or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*(/[A-Za-z0-9][A-Za-z0-9._-]*)*", value)):
            raise RuntimeError("The source restore path is unsupported")
        for segment in value.split("/"):
            stem = segment.split(".", 1)[0].upper()
            if segment.endswith(".") or stem in ("CON", "PRN", "AUX", "NUL") or re.fullmatch(r"(?:COM|LPT)[1-9]", stem):
                raise RuntimeError("The source restore path is not portable")
        return value

    @staticmethod
    def _source_key(value):
        """Require a bounded header key before any new source effect dispatch."""
        if type(value) is not str or not 16 <= len(value) <= 255 or any(ord(char) < 33 or ord(char) > 126 for char in value):
            raise ValueError("Use a bounded printable source idempotency key")
        return value

    @staticmethod
    def _source_integer(value, maximum):
        """Reject booleans and oversized/negative integers in finite source DTO facts."""
        if type(value) is not int or not 0 <= value <= maximum:
            raise RuntimeError("The source restore count is unsupported")
        return value

    @classmethod
    def _source_validation(cls, value):
        """Inspect the complete bounded validation envelope without inventing native admission."""
        cls._source_closed(value, ("state", "error_count", "warning_count", "diagnostics"))
        if value["state"] not in ("valid", "invalid") or type(value["diagnostics"]) is not list or len(value["diagnostics"]) > 500:
            raise RuntimeError("The source restore validation is unsupported")
        cls._source_integer(value["error_count"], 2**63 - 1)
        cls._source_integer(value["warning_count"], 2**63 - 1)
        for row in value["diagnostics"]:
            cls._source_closed(row, ("code", "severity", "message"), ("resource_id", "field"))
            if (row["severity"] not in ("error", "warning", "info")
                or type(row["code"]) is not str or not 3 <= len(row["code"]) <= 100
                or type(row["message"]) is not str or not 1 <= len(row["message"]) <= 1000):
                raise RuntimeError("The source restore diagnostic is unsupported")
            if row.get("resource_id") is not None:
                cls._source_id(row["resource_id"], "res")
            if row.get("field") is not None and (type(row["field"]) is not str or len(row["field"]) > 256):
                raise RuntimeError("The source restore diagnostic pointer is unsupported")

    @classmethod
    def _source_file_fact(cls, value, *, prior):
        """Validate registered or explicitly unlabeled base facts without manufacturing identity."""
        names = ("key", "role", "sha256", "size") + (("resource_id",) if prior else ())
        cls._source_closed(value, names)
        if value["key"] is not None and (type(value["key"]) is not str or not re.fullmatch(r"[a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?", value["key"])):
            raise RuntimeError("The source restore registration key is unsupported")
        if value["role"] is not None and value["role"] not in SOURCE_ROLES:
            raise RuntimeError("The source restore role is unsupported")
        if (value["key"] is None) != (value["role"] is None):
            raise RuntimeError("The source restore registration facts disagree")
        if prior:
            if (value["key"] is None) != (value["resource_id"] is None):
                raise RuntimeError("The source restore prior identity disagrees")
            if value["resource_id"] is not None:
                cls._source_id(value["resource_id"], "res")
        cls._source_hash(value["sha256"])
        cls._source_integer(value["size"], 10 * 1024 * 1024)

    @classmethod
    def _source_preview(cls, preview):
        """Inspect every target/binding/directory and retain preknown ID before any commit."""
        names = ("preview_id", "operation_id", "operation_type", "snapshot_version", "observed_batch_version",
                 "exact_manifest_sha256", "targets", "input_bindings", "directories", "validation", "semantic_summary", "receipt")
        cls._source_closed(preview, names)
        cls._source_id(preview["preview_id"], "prev")
        cls._source_id(preview["operation_id"], "op")
        for name in ("snapshot_version", "observed_batch_version", "exact_manifest_sha256"):
            cls._source_hash(preview[name])
        if (preview["operation_type"] != "project-source-restore" or type(preview["semantic_summary"]) is not str
            or len(preview["semantic_summary"]) > 4000):
            raise RuntimeError("The source restore preview is unsupported")
        cls._source_validation(preview["validation"])
        cls._source_closed(preview["receipt"], ("token", "expires_at"))
        if (type(preview["receipt"]["token"]) is not str or not 16 <= len(preview["receipt"]["token"]) <= 512
            or type(preview["receipt"]["expires_at"]) is not str or not preview["receipt"]["expires_at"]):
            raise RuntimeError("The source restore receipt is unsupported")
        for field in ("targets", "input_bindings", "directories"):
            if type(preview[field]) is not list or len(preview[field]) > 100:
                raise RuntimeError("The complete source restore plan exceeds its bound")
        if not preview["targets"]:
            raise RuntimeError("The source restore has no complete index target")
        target_paths, diff_bytes = set(), 0
        targets_by_path = {}
        for row in preview["targets"]:
            cls._source_closed(row, ("path", "kind", "key", "role", "status", "base_sha256", "base_size", "target_version",
                                     "exact_bytes_sha256", "size", "diff_text", "diff_truncated", "binary"))
            path = cls._source_path(row["path"])
            if path in target_paths:
                raise RuntimeError("The source restore target is duplicated")
            target_paths.add(path)
            targets_by_path[path] = row
            cls._source_file_fact({"key":row["key"], "role":row["role"], "sha256":row["exact_bytes_sha256"], "size":row["size"]}, prior=False)
            if (row["kind"] not in ("index", "resource") or row["status"] not in ("create", "overwrite")
                or type(row["target_version"]) is not str or not 8 <= len(row["target_version"]) <= 128
                or type(row["diff_text"]) is not str or type(row["diff_truncated"]) is not bool or type(row["binary"]) is not bool):
                raise RuntimeError("The source restore target facts are unsupported")
            if row["status"] == "overwrite":
                cls._source_hash(row["base_sha256"])
                cls._source_integer(row["base_size"], 10 * 1024 * 1024)
            elif row["base_sha256"] is not None or row["base_size"] is not None:
                raise RuntimeError("The source restore create base is unsupported")
            if (path == "forge.workspace.json") != (row["kind"] == "index") or (row["kind"] == "index") != (row["key"] is None):
                raise RuntimeError("The source restore index identity is unsupported")
            diff_bytes += len(row["diff_text"].encode("utf-8"))
        if preview["targets"][-1]["path"] != "forge.workspace.json" or diff_bytes > 200000:
            raise RuntimeError("The complete source restore order or diff budget is unsupported")
        binding_paths = set()
        bindings_by_path = {}
        for row in preview["input_bindings"]:
            cls._source_closed(row, ("path", "kind", "current", "proposed"))
            path = cls._source_path(row["path"])
            if path in binding_paths or row["kind"] not in ("index", "resource") or (path == "forge.workspace.json") != (row["kind"] == "index"):
                raise RuntimeError("The source restore binding is unsupported")
            binding_paths.add(path)
            bindings_by_path[path] = row
            if row["current"] is None and row["proposed"] is None:
                raise RuntimeError("The source restore binding has no generation")
            for name in ("current", "proposed"):
                if row[name] is not None:
                    cls._source_file_fact(row[name], prior=name == "current")
        if not target_paths.issubset(binding_paths):
            raise RuntimeError("The complete source restore target bindings are missing")
        for path, target in targets_by_path.items():
            binding = bindings_by_path[path]
            proposed = binding["proposed"]
            if proposed != {"key":target["key"], "role":target["role"], "sha256":target["exact_bytes_sha256"], "size":target["size"]}:
                raise RuntimeError("The source restore target and proposed binding disagree")
            current = binding["current"]
            if ((target["status"] == "create" and current is not None)
                or (target["status"] == "overwrite" and (current is None or current["sha256"] != target["base_sha256"] or current["size"] != target["base_size"]))):
                raise RuntimeError("The source restore target and observed base disagree")
        directory_paths = []
        for row in preview["directories"]:
            cls._source_closed(row, ("path", "status", "nearest_existing_parent_version"))
            path = cls._source_path(row["path"])
            if (row["status"] != "create" or path in directory_paths or type(row["nearest_existing_parent_version"]) is not str
                or not 8 <= len(row["nearest_existing_parent_version"]) <= 128):
                raise RuntimeError("The source restore directory plan is unsupported")
            directory_paths.append(path)
        for position, path in enumerate(directory_paths):
            if any(path.startswith(parent + "/") for parent in directory_paths[position + 1:]):
                raise RuntimeError("The source restore directory plan is not parent-first")
        return preview

    @classmethod
    def _source_index(cls, value):
        """Inspect complete explicit index membership without interpreting it as domain authority."""
        cls._source_closed(value, ("schema_version", "label", "resources"))
        if (value["schema_version"] not in ("forge.workspace/1", "forge.workspace/2")
            or type(value["label"]) is not str or not 1 <= len(value["label"]) <= 200
            or any(ord(char) < 32 or 127 <= ord(char) <= 159 for char in value["label"])
            or type(value["resources"]) is not list or len(value["resources"]) > 1000):
            raise RuntimeError("The source restore complete index is unsupported")
        allowed = SOURCE_LEGACY_ROLES if value["schema_version"] == "forge.workspace/1" else SOURCE_ROLES
        keys, paths = set(), set()
        for row in value["resources"]:
            cls._source_closed(row, ("key", "role", "path"))
            cls._source_file_fact({"key":row["key"], "role":row["role"], "sha256":"0" * 64, "size":0}, prior=False)
            path = cls._source_path(row["path"])
            if row["key"] is None or row["role"] not in allowed or row["key"] in keys or path.lower() in paths or path.lower() == "forge.workspace.json":
                raise RuntimeError("The source restore index membership is unsupported")
            keys.add(row["key"])
            paths.add(path.lower())
        return value

    @staticmethod
    def _source_index_hash(value):
        """Hash the deterministic admitted index fields and LF, never the original uploaded bytes."""
        ordered = {"schema_version":value["schema_version"], "label":value["label"],
                   "resources":[{"key":row["key"], "role":row["role"], "path":row["path"]} for row in value["resources"]]}
        raw = (json.dumps(ordered, ensure_ascii=False, allow_nan=False, indent=2) + "\n").encode("utf-8")
        if len(raw) > 1024 * 1024:
            raise RuntimeError("The complete source restore index exceeds its bound")
        return hashlib.sha256(raw).hexdigest()

    @classmethod
    def _source_replacement(cls, replacement, preview, target_index_schema_version):
        """Reconcile full replacement membership and every ordered target/removal with complete bindings."""
        cls._source_closed(replacement, ("previous_index", "proposed_index", "supplied_index_sha256",
                           "proposed_index_sha256", "removed_resource_keys", "consumed_file_count"))
        cls._source_hash(replacement["supplied_index_sha256"])
        cls._source_hash(replacement["proposed_index_sha256"])
        cls._source_integer(replacement["consumed_file_count"], 100)
        previous = None if replacement["previous_index"] is None else cls._source_index(replacement["previous_index"])
        proposed = cls._source_index(replacement["proposed_index"])
        selected = "forge.workspace/" + str(target_index_schema_version)
        if proposed["schema_version"] != selected or (previous is not None and previous["schema_version"] == "forge.workspace/2" and selected == "forge.workspace/1"):
            raise RuntimeError("The source restore selected index version disagrees or would downgrade")
        if cls._source_index_hash(proposed) != replacement["proposed_index_sha256"]:
            raise RuntimeError("The source restore normalized proposed index hash disagrees")
        if len(proposed["resources"]) > 99 or replacement["consumed_file_count"] != len(preview["input_bindings"]):
            raise RuntimeError("The source restore whole replacement count is unsupported")
        current_rows = [] if previous is None else previous["resources"]
        incoming_keys = {row["key"] for row in proposed["resources"]}
        if replacement["removed_resource_keys"] != [row["key"] for row in current_rows if row["key"] not in incoming_keys]:
            raise RuntimeError("The source restore complete removed-key order disagrees")
        expected_paths = {"forge.workspace.json"} | {row["path"] for row in current_rows + proposed["resources"]}
        if expected_paths != {row["path"] for row in preview["input_bindings"]}:
            raise RuntimeError("The source restore complete replacement paths disagree")
        registered = {row["path"]:row for row in current_rows}
        proposed_paths = {row["path"] for row in proposed["resources"]} | {"forge.workspace.json"}
        for binding in preview["input_bindings"]:
            prior = binding["current"]
            registration = registered.get(binding["path"])
            if registration is not None:
                if prior is None or (prior["key"], prior["role"]) != (registration["key"], registration["role"]):
                    raise RuntimeError("The source restore prior registration facts disagree")
            elif prior is not None and (prior["key"] is not None or prior["role"] is not None or prior["resource_id"] is not None):
                raise RuntimeError("The source restore unregistered base is mislabeled")
            if (binding["proposed"] is not None) != (binding["path"] in proposed_paths):
                raise RuntimeError("The source restore proposed membership facts disagree")
        expected_targets = [(row["path"], "resource", row["key"], row["role"]) for row in proposed["resources"]]
        expected_targets.append(("forge.workspace.json", "index", None, None))
        if [(row["path"], row["kind"], row["key"], row["role"]) for row in preview["targets"]] != expected_targets:
            raise RuntimeError("The source restore complete target membership disagrees")
        if preview["targets"][-1]["exact_bytes_sha256"] != replacement["proposed_index_sha256"]:
            raise RuntimeError("The source restore index target hash disagrees")
        index_binding = next(row for row in preview["input_bindings"] if row["path"] == "forge.workspace.json")
        if (previous is None) != (index_binding["current"] is None):
            raise RuntimeError("The source restore previous-index absence disagrees")
        return replacement

    @classmethod
    def _source_result(cls, result, cleanup_state):
        """Validate complete committed targets separately from the journal's cleanup qualification."""
        cls._source_closed(result, ("write_committed", "exact_manifest_sha256", "committed_targets", "cleanup_state"))
        cls._source_hash(result["exact_manifest_sha256"])
        if (result["write_committed"] is not True or result["cleanup_state"] != cleanup_state
            or cleanup_state not in ("verified", "pending", "unverified")
            or type(result["committed_targets"]) is not list or not 1 <= len(result["committed_targets"]) <= 100):
            raise RuntimeError("The source restore committed targets are unsupported")
        paths = set()
        for row in result["committed_targets"]:
            cls._source_closed(row, ("path", "sha256", "size"))
            path = cls._source_path(row["path"])
            cls._source_hash(row["sha256"])
            cls._source_integer(row["size"], 10 * 1024 * 1024)
            if path in paths:
                raise RuntimeError("The source restore committed target is duplicated")
            paths.add(path)
        if result["committed_targets"][-1]["path"] != "forge.workspace.json":
            raise RuntimeError("The source restore committed index order is unsupported")

    @classmethod
    def _source_operation(cls, operation, operation_id):
        """Validate separate batch outcome/cleanup pairs without treating acknowledgment as terminal."""
        cls._source_closed(operation, ("operation_id", "kind", "state", "created_at", "updated_at", "cancel_requested",
                                       "write_outcome", "progress", "result", "error", "cleanup_state"))
        if (operation["operation_id"] != cls._source_id(operation_id, "op") or operation["kind"] != "bundle-restore"
            or operation["state"] not in ("pending", "running", "succeeded", "failed", "cancelled", "recovery-required")
            or operation["write_outcome"] not in ("unmeasured", "none", "committed", "unknown")
            or operation["cleanup_state"] not in ("unmeasured", "verified", "pending", "unverified")
            or type(operation["cancel_requested"]) is not bool
            or any(type(operation[name]) is not str or not operation[name] for name in ("created_at", "updated_at"))):
            raise RuntimeError("The source restore operation is unsupported")
        if operation["progress"] is not None:
            cls._source_closed(operation["progress"], ("completed_files", "total_files"))
            completed = cls._source_integer(operation["progress"]["completed_files"], 100)
            total = cls._source_integer(operation["progress"]["total_files"], 100)
            if completed > total or operation["state"] not in ("pending", "running") or operation["cancel_requested"]:
                raise RuntimeError("The source restore progress is unsupported")
        if operation["state"] in ("pending", "running"):
            if operation["write_outcome"] != "unmeasured" or operation["result"] is not None or operation["error"] is not None:
                raise RuntimeError("The source restore pending outcome is unsupported")
        elif operation["state"] == "succeeded":
            if operation["write_outcome"] != "committed" or operation["error"] is not None:
                raise RuntimeError("The source restore committed outcome is unsupported")
            cls._source_result(operation["result"], operation["cleanup_state"])
        else:
            if operation["state"] in ("failed", "cancelled") and (operation["write_outcome"] != "none" or operation["cleanup_state"] != "verified"):
                raise RuntimeError("The source restore rollback outcome is unsupported")
            if operation["state"] == "recovery-required" and operation["write_outcome"] == "unmeasured":
                raise RuntimeError("The source restore recovery outcome is unsupported")
            if operation["result"] is not None:
                if operation["state"] != "recovery-required" or operation["write_outcome"] != "committed":
                    raise RuntimeError("The source restore terminal result is unsupported")
                cls._source_result(operation["result"], operation["cleanup_state"])
            if operation["state"] == "cancelled" and operation["error"] is None:
                return operation
            if type(operation["error"]) is not dict:
                raise RuntimeError("The source restore terminal error is unsupported")
            cls._source_closed(operation["error"], ("code", "message", "retryable"), ("correlation_id", "field", "resource", "resource_version"))
            if (type(operation["error"]["code"]) is not str or not operation["error"]["code"]
                or type(operation["error"]["message"]) is not str or not 1 <= len(operation["error"]["message"]) <= 500
                or type(operation["error"]["retryable"]) is not bool):
                raise RuntimeError("The source restore error is unsupported")
        return operation

    def prepare_source_bundle_export(self, target_path, *, acknowledge_sensitive_metadata, acknowledge_source_content, idempotency_key):
        """Prepare opt-in exact source output; completion is only a normal single-file preview.

        Labels, registration paths and source bytes may be sensitive. Source-only
        planning reserves the output even absent, alongside index/current paths;
        a distinct output plus present index permits at most98 resources.
        """
        self._source_supported()
        self._source_path(target_path)
        self._source_key(idempotency_key)
        if acknowledge_sensitive_metadata is not True or acknowledge_source_content is not True or not idempotency_key:
            raise ValueError("Both source sensitivity acknowledgments and an idempotency key are required")
        return self.request("POST", self.api_path("/project/source-bundle-exports"), {
            "target_path":target_path, "acknowledge_sensitive_metadata":True, "acknowledge_source_content":True,
        }, idempotency_key=idempotency_key)

    def prepare_source_bundle_import(self, bundle_bytes, *, target_index_schema_version, acknowledge_index_replacement,
                                     acknowledge_source_content, acknowledge_replace_files, idempotency_key):
        """Wrap exact original local bytes with147-byte explicit envelope; never collapse duplicates."""
        self._source_supported()
        self._source_key(idempotency_key)
        if (type(bundle_bytes) is not bytes or type(target_index_schema_version) is not int or target_index_schema_version not in (1, 2)
            or acknowledge_index_replacement is not True or acknowledge_source_content is not True
            or acknowledge_replace_files is not True or not idempotency_key or len(bundle_bytes) > MAX_SOURCE_BUNDLE):
            raise ValueError("Select bounded source bytes, an index version and every explicit acknowledgment")
        suffix = (',"target_index_schema_version":' + str(target_index_schema_version)
                  + ',"acknowledge_index_replacement":true,"acknowledge_source_content":true,"acknowledge_replace_files":true}').encode("ascii")
        raw = b'{"bundle":' + bundle_bytes + suffix
        if len(raw) > 1024 * 1024:
            raise ValueError("The complete exact source import envelope exceeds1MiB")
        response = self.request("POST", self.api_path("/project/source-bundle-imports"),
                                raw_json_body=raw, idempotency_key=idempotency_key)
        self._source_closed(response, ("validation", "preview", "replacement"))
        self._source_validation(response["validation"])
        preview = self._source_preview(response["preview"])
        if response["validation"] != preview["validation"]:
            raise RuntimeError("The source restore validation envelopes disagree")
        self._source_replacement(response["replacement"], preview, target_index_schema_version)
        return response

    def prepare_source_bundle_import_file(self, path, **acknowledgments):
        """Read one selected regular file with a fixed cap and stable observed instance; never print its path."""
        self._source_supported()
        descriptor = None
        try:
            flags = os.O_RDONLY | getattr(os, "O_NONBLOCK", 0) | getattr(os, "O_CLOEXEC", 0)
            descriptor = os.open(path, flags)
            before = os.fstat(descriptor)
            if not stat.S_ISREG(before.st_mode) or before.st_size > MAX_SOURCE_BUNDLE:
                raise ValueError("Select a bounded regular source bundle")
            stream = os.fdopen(descriptor, "rb")
            descriptor = None
            with stream:
                raw = stream.read(MAX_SOURCE_BUNDLE + 1)
                after = os.fstat(stream.fileno())
                identity = ("st_dev", "st_ino", "st_size", "st_mtime_ns")
                if (len(raw) > MAX_SOURCE_BUNDLE or len(raw) != before.st_size
                    or any(getattr(before, key) != getattr(after, key) for key in identity)):
                    raise ValueError("The selected source bundle changed or exceeds its bound")
        except OSError:
            raise ValueError("The selected source bundle could not be read") from None
        finally:
            if descriptor is not None:
                try:
                    os.close(descriptor)
                except OSError:
                    raise ValueError("The selected source bundle cleanup could not be verified") from None
        return self.prepare_source_bundle_import(raw, **acknowledgments)

    def download_source_bundle_export(self, operation_id, *, expected_sha256):
        """Return exact committed private-family source JSON only after media/hash/reimport cap checks."""
        self._source_supported()
        self._source_id(operation_id, "op")
        self._source_hash(expected_sha256)
        raw = self.request("GET", self.api_path("/project/source-bundle-exports/" + operation_id + "/download"),
                           raw=True, expected_media_type="application/json")
        if len(raw) > MAX_SOURCE_BUNDLE or hashlib.sha256(raw).hexdigest() != expected_sha256:
            raise RuntimeError("The exact source export bytes do not match the prepared artifact")
        return raw

    def commit_source_restore(self, preview, *, confirmed, idempotency_key):
        """Send one complete confirmed batch; uncertain replies retain preknown lookup and never resend."""
        self._source_supported()
        self._source_preview(preview)
        self._source_key(idempotency_key)
        if confirmed is not True or not idempotency_key or preview["validation"]["state"] != "valid" or preview["validation"]["error_count"] != 0:
            raise ValueError("Complete valid source restore confirmation and an idempotency key are required")
        operation_id = preview["operation_id"]
        try:
            operation = self.request("POST", self.api_path("/project/bundle-restores/" + preview["preview_id"] + "/commit"), {
                "receipt":preview["receipt"]["token"], "observed_batch_version":preview["observed_batch_version"],
                "acknowledge_exact_restore":True,
            }, idempotency_key=idempotency_key)
            return self._source_operation(operation, operation_id)
        except WorkspaceError:
            raise
        except (OSError, http.client.HTTPException, RuntimeError, ValueError):
            raise SourceRestoreUncertain(operation_id) from None

    def source_restore_status(self, operation_id):
        """Query preknown ID in this fresh same-root session;404 does not establish no-write or authorize resend."""
        self._source_supported()
        self._source_id(operation_id, "op")
        value = self.request("GET", self.api_path("/project/bundle-restores/" + operation_id))
        return self._source_operation(value, operation_id)

    def cancel_source_restore(self, operation_id):
        """Request cancellation only; running acknowledgment is not verified rollback or terminal proof."""
        self._source_supported()
        self._source_id(operation_id, "op")
        value = self.request("POST", self.api_path("/project/bundle-restores/" + operation_id + "/cancel"), {})
        return self._source_operation(value, operation_id)

    def wait_source_restore(self, operation_id, *, timeout=65):
        """Poll only known status with one caller budget; never restart or reconfirm a batch on timeout."""
        self._source_supported()
        self._source_id(operation_id, "op")
        if type(timeout) not in (int, float) or isinstance(timeout, bool) or not 0 < timeout <= 120:
            raise ValueError("Choose a finite restore polling timeout up to120seconds")
        deadline = time.monotonic() + timeout
        while True:
            if time.monotonic() >= deadline:
                raise SourceRestoreUncertain(operation_id)
            operation = self.source_restore_status(operation_id)
            if operation["state"] not in ("pending", "running"):
                return operation
            time.sleep(min(0.1, max(0, deadline - time.monotonic())))


    def _staged_supported(self):
        """Require exact negotiated2.4; numeric-major bootstrap alone grants no staged feature."""
        if self._api_major != 2 or self._contract_version != "2.4.0":
            raise ValueError("Staged source transfer requires API2 contract2.4.0")

    @staticmethod
    def _staged_pairs(pairs):
        """Reject decoded and escaped duplicate object keys without displaying supplied names."""
        value = {}
        for key, child in pairs:
            if key in value:
                raise ValueError("The staged artifact contains duplicate object keys")
            value[key] = child
        return value

    @staticmethod
    def _staged_constant(_value):
        """Reject nonfinite JSON tokens without retaining their spelling in a public exception."""
        raise ValueError("The staged artifact contains unsupported JSON scalars")

    @staticmethod
    def _staged_time(value):
        """Check a bounded offset-bearing timestamp, without treating expiry text as a local authority."""
        if (type(value) is not str or len(value) > 40
            or not re.fullmatch(r"\d{4}-\d{2}-\d{2}[Tt]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:[Zz]|[+-]\d{2}:\d{2})", value)):
            raise RuntimeError("The staged timestamp is unsupported")
        try:
            parsed = datetime.fromisoformat(value.replace("t", "T").replace("z", "+00:00").replace("Z", "+00:00"))
            if parsed.tzinfo is None:
                raise ValueError
        except ValueError:
            raise RuntimeError("The staged timestamp is unsupported") from None
        return value

    @classmethod
    def _staged_identity(cls, value):
        """Check the closed Bundle4 tag/profile and exact logical artifact/part declaration."""
        if (value["schema_version"] != "forge.workspace-index-bundle/4"
            or value["profile"] != "index-and-source-hex-staged"):
            raise RuntimeError("The staged artifact profile is unsupported")
        cls._source_hash(value["artifact_sha256"])
        size = cls._source_integer(value["artifact_size_bytes"], MAX_STAGED_SOURCE_BUNDLE)
        count = cls._source_integer(value["chunk_count"], MAX_STAGED_PARTS)
        if (size == 0 or type(value["chunk_size_bytes"]) is not int or value["chunk_size_bytes"] != STAGED_PART_BYTES
            or count != (size + STAGED_PART_BYTES - 1) // STAGED_PART_BYTES):
            raise RuntimeError("The staged artifact size and part count disagree")
        return value

    @classmethod
    def _staged_bundle(cls, raw):
        """Inspect exact bounded raw Bundle4 bytes without re-encoding them or approving native closure.

        Original-byte SHA remains separate from the normalized index hash. These
        structural/hash checks establish neither native domain admission nor
        current freshness or a literal decoder heap bound.
        """
        if type(raw) is not bytes or not 1 <= len(raw) <= MAX_STAGED_SOURCE_BUNDLE or raw.startswith(b"\xef\xbb\xbf"):
            raise ValueError("Select bounded exact staged source bytes without a BOM")
        try:
            value = json.loads(raw.decode("utf-8"), object_pairs_hook=cls._staged_pairs, parse_constant=cls._staged_constant)
        except (ValueError, UnicodeError, RecursionError):
            raise ValueError("The staged artifact is not one strict UTF8 JSON document") from None
        cls._source_closed(value, ("schema_version", "profile", "source_content_included", "index", "index_sha256", "pins", "contents"))
        if (value["schema_version"] != "forge.workspace-index-bundle/4" or value["profile"] != "index-and-source-hex-staged"
            or value["source_content_included"] is not True):
            raise ValueError("Select the explicit staged source profile")
        index = cls._source_index(value["index"])
        if len(index["resources"]) > 99:
            raise ValueError("The staged artifact exceeds complete index membership")
        cls._source_hash(value["index_sha256"])
        if cls._source_index_hash(index) != value["index_sha256"]:
            raise RuntimeError("The staged normalized index hash disagrees")
        if (type(value["pins"]) is not list or type(value["contents"]) is not list
            or len(value["pins"]) != len(index["resources"]) or len(value["contents"]) != len(index["resources"])):
            raise RuntimeError("The staged index pin and content membership disagree")
        for resource, pin, content in zip(index["resources"], value["pins"], value["contents"]):
            cls._source_closed(pin, ("key", "sha256", "size"))
            cls._source_closed(content, ("key", "encoding", "chunks"))
            cls._source_hash(pin["sha256"])
            size = cls._source_integer(pin["size"], MAX_STAGED_SOURCE_BUNDLE)
            if (pin["key"] != resource["key"] or content["key"] != resource["key"] or content["encoding"] != "hex"
                or type(content["chunks"]) is not list or len(content["chunks"]) > MAX_STAGED_PARTS):
                raise RuntimeError("The staged authorial pin and content order disagree")
            hasher, decoded = hashlib.sha256(), 0
            for ordinal, text in enumerate(content["chunks"]):
                part = cls._staged_hex(text)
                if ordinal + 1 < len(content["chunks"]) and len(part) != STAGED_PART_BYTES:
                    raise RuntimeError("The staged nonfinal content part is incomplete")
                hasher.update(part); decoded += len(part)
            if decoded != size or hasher.hexdigest() != pin["sha256"]:
                raise RuntimeError("The staged content size or hash disagrees")
        return raw

    @staticmethod
    def _staged_hex(value):
        """Decode only finite canonical lowercase-even hex; reject empty or oversized parts first."""
        if (type(value) is not str or not 2 <= len(value) <= STAGED_PART_BYTES * 2
            or not re.fullmatch(r"(?:[a-f0-9]{2})+", value)):
            raise RuntimeError("The staged part encoding is unsupported")
        return bytes.fromhex(value)


    @classmethod
    def _staged_received(cls, count, received, size, parts, *, acknowledged_ordinal=None):
        """Require a byte total from actual full-part/final-remainder membership, not arbitrary counters."""
        cls._source_integer(count, parts)
        cls._source_integer(received, size)
        last = size - (parts - 1) * STAGED_PART_BYTES
        possible = {0} if count == 0 else {count * STAGED_PART_BYTES, (count - 1) * STAGED_PART_BYTES + last}
        if acknowledged_ordinal is not None:
            if count == 0:
                raise RuntimeError("The staged acknowledgment contains no observed part")
            if acknowledged_ordinal == parts - 1:
                possible = {(count - 1) * STAGED_PART_BYTES + last}
            elif count == 1:
                possible = {STAGED_PART_BYTES}
        if received not in possible or (count == parts and received != size):
            raise RuntimeError("The staged received total cannot represent exact retained parts")

    @classmethod
    def _staged_stage(cls, value, *, stage_id=None, expected=None):
        """Validate complete closed transport counters without claiming native readiness or renewal."""
        cls._source_closed(value, ("stage_id", "schema_version", "profile", "artifact_sha256", "artifact_size_bytes",
                                 "chunk_size_bytes", "chunk_count", "received_chunk_count", "received_bytes", "state", "expires_at"))
        cls._staged_identity(value)
        cls._source_id(value["stage_id"], "bst")
        cls._staged_time(value["expires_at"])
        count = cls._source_integer(value["received_chunk_count"], value["chunk_count"])
        received = cls._source_integer(value["received_bytes"], value["artifact_size_bytes"])
        cls._staged_received(count, received, value["artifact_size_bytes"], value["chunk_count"])
        if (value["state"] not in ("receiving", "ready", "preparing", "prepared", "discarded")
            or (value["state"] == "receiving" and count == value["chunk_count"])
            or (value["state"] in ("ready", "preparing", "prepared")
                and (count != value["chunk_count"] or received != value["artifact_size_bytes"]))):
            raise RuntimeError("The staged transport counters disagree")
        if stage_id is not None and value["stage_id"] != stage_id:
            raise RuntimeError("The staged response identifier disagrees")
        if expected is not None and any(value[key] != expected[key] for key in expected):
            raise RuntimeError("The staged artifact declaration or original lifetime changed")
        return value

    @classmethod
    def _staged_ack(cls, value, stage, ordinal, raw):
        """Bind original acknowledgment to exact stage/ordinal/part; old replay counters may remain old."""
        cls._source_closed(value, ("stage_id", "chunk_ordinal", "sha256", "size_bytes", "received_chunk_count", "received_bytes", "expires_at"))
        cls._source_hash(value["sha256"])
        cls._source_integer(value["chunk_ordinal"], MAX_STAGED_PARTS - 1)
        count = cls._source_integer(value["received_chunk_count"], stage["chunk_count"])
        received = cls._source_integer(value["received_bytes"], stage["artifact_size_bytes"])
        size = cls._source_integer(value["size_bytes"], STAGED_PART_BYTES)
        cls._staged_time(value["expires_at"])
        cls._staged_received(count, received, stage["artifact_size_bytes"], stage["chunk_count"], acknowledged_ordinal=ordinal)
        if (value["stage_id"] != stage["stage_id"] or value["chunk_ordinal"] != ordinal or size != len(raw)
            or value["sha256"] != hashlib.sha256(raw).hexdigest() or received < size
            or value["expires_at"] != stage["expires_at"]):
            raise RuntimeError("The staged acknowledgment disagrees with exact sent bytes or lifetime")
        return value


    @staticmethod
    def _staged_read_file(path):
        """Read one selected regular file with10MiB cap and stable observed instance; never display its path."""
        descriptor = None
        try:
            flags = os.O_RDONLY | getattr(os, "O_NONBLOCK", 0) | getattr(os, "O_CLOEXEC", 0)
            descriptor = os.open(path, flags)
            before = os.fstat(descriptor)
            if not stat.S_ISREG(before.st_mode) or not 1 <= before.st_size <= MAX_STAGED_SOURCE_BUNDLE:
                raise ValueError("Select a bounded regular staged artifact")
            stream = os.fdopen(descriptor, "rb")
            descriptor = None
            with stream:
                raw = stream.read(MAX_STAGED_SOURCE_BUNDLE + 1)
                after = os.fstat(stream.fileno())
                identity = ("st_dev", "st_ino", "st_size", "st_mtime_ns")
                if (len(raw) != before.st_size or len(raw) > MAX_STAGED_SOURCE_BUNDLE
                    or any(getattr(before, key) != getattr(after, key) for key in identity)):
                    raise ValueError("The selected staged artifact changed or exceeds its bound")
        except OSError:
            raise ValueError("The selected staged artifact could not be read") from None
        finally:
            if descriptor is not None:
                try:
                    os.close(descriptor)
                except OSError:
                    raise ValueError("The selected staged artifact cleanup could not be verified") from None
        return raw

    def create_source_transfer_stage_file(self, path, **acknowledgments):
        """Admit stable bounded selected-file bytes before creating any session-local transport record."""
        self._staged_supported()
        return self.create_source_transfer_stage(self._staged_read_file(path), **acknowledgments)

    def upload_source_transfer_stage_file(self, stage, path, *, timeout=60):
        """Re-read a bounded selected file and match its original full hash before any part dispatch."""
        self._staged_supported()
        return self.upload_source_transfer_stage(stage, self._staged_read_file(path), timeout=timeout)

    def create_source_transfer_stage(self, bundle_bytes, *, acknowledge_sensitive_metadata, acknowledge_source_content, idempotency_key):
        """Create one acknowledged declaration from exact strict bytes; no implicit upload or confirmation."""
        self._staged_supported()
        self._source_key(idempotency_key)
        if acknowledge_sensitive_metadata is not True or acknowledge_source_content is not True:
            raise ValueError("Both staged source sensitivity acknowledgments are required")
        raw = self._staged_bundle(bundle_bytes)
        declaration = {"schema_version":"forge.workspace-index-bundle/4", "profile":"index-and-source-hex-staged",
                       "artifact_sha256":hashlib.sha256(raw).hexdigest(), "artifact_size_bytes":len(raw),
                       "chunk_size_bytes":STAGED_PART_BYTES, "chunk_count":(len(raw) + STAGED_PART_BYTES - 1) // STAGED_PART_BYTES}
        value = self.request("POST", self.api_path("/project/source-transfer-stages"),
                             {**declaration, "acknowledge_sensitive_metadata":True, "acknowledge_source_content":True},
                             idempotency_key=idempotency_key)
        return self._staged_stage(value, expected=declaration)

    def put_source_transfer_chunk(self, stage, chunk_ordinal, chunk_bytes):
        """Send one exact part once; an explicit identical retry replays without renewing lifetime."""
        self._staged_supported()
        self._staged_stage(stage)
        ordinal = self._source_integer(chunk_ordinal, stage["chunk_count"] - 1)
        size = min(STAGED_PART_BYTES, stage["artifact_size_bytes"] - ordinal * STAGED_PART_BYTES)
        if type(chunk_bytes) is not bytes or len(chunk_bytes) != size:
            raise ValueError("Send the exact declared staged part length")
        value = self.request("PUT", self.api_path("/project/source-transfer-stages/" + stage["stage_id"] + "/chunks/" + str(ordinal)),
                             {"sha256":hashlib.sha256(chunk_bytes).hexdigest(), "size_bytes":size, "hex":chunk_bytes.hex()})
        return self._staged_ack(value, stage, ordinal, chunk_bytes)

    def source_transfer_status(self, stage_id):
        """Read current-session transport only;404 grants neither no-write proof nor renewed authority."""
        self._staged_supported()
        self._source_id(stage_id, "bst")
        return self._staged_stage(self.request("GET", self.api_path("/project/source-transfer-stages/" + stage_id)), stage_id=stage_id)

    def upload_source_transfer_stage(self, stage, bundle_bytes, *, timeout=60):
        """Send each exact ordinal once and return actual status; never retry a failed response automatically.

        Explicit re-invocation may replay identical parts. The one caller deadline
        is checked between paced network calls; it does not preempt a syscall.
        """
        self._staged_supported()
        self._staged_stage(stage)
        raw = self._staged_bundle(bundle_bytes)
        if len(raw) != stage["artifact_size_bytes"] or hashlib.sha256(raw).hexdigest() != stage["artifact_sha256"]:
            raise ValueError("The original staged artifact changed")
        if type(timeout) not in (int, float) or not 0 < timeout <= 600:
            raise ValueError("Choose a finite staged upload budget up to600seconds")
        deadline = time.monotonic() + timeout
        for ordinal in range(stage["chunk_count"]):
            if time.monotonic() >= deadline:
                raise TimeoutError("Read stage status before explicitly recovering the upload")
            self.put_source_transfer_chunk(stage, ordinal, raw[ordinal * STAGED_PART_BYTES:(ordinal + 1) * STAGED_PART_BYTES])
        if time.monotonic() >= deadline:
            raise TimeoutError("Read stage status before explicitly recovering the upload")
        value = self.source_transfer_status(stage["stage_id"])
        expected = {key:stage[key] for key in ("schema_version", "profile", "artifact_sha256", "artifact_size_bytes", "chunk_size_bytes", "chunk_count", "expires_at")}
        return self._staged_stage(value, expected=expected)

    def discard_source_transfer_stage(self, stage_id):
        """Retire only an unconfirmed stage once; never infer accepted restore cancellation or rollback."""
        self._staged_supported()
        self._source_id(stage_id, "bst")
        value = self.request("DELETE", self.api_path("/project/source-transfer-stages/" + stage_id), {})
        self._source_closed(value, ("stage_id", "discarded"))
        if value["stage_id"] != stage_id or value["discarded"] is not True:
            raise RuntimeError("The stage retirement response disagrees")
        return value

    def prepare_staged_source_restore(self, stage_id, *, target_index_schema_version, acknowledge_index_replacement,
                                      acknowledge_source_content, acknowledge_replace_files, idempotency_key):
        """Read the complete native preview and preknown ID; existing explicit confirmation remains separate."""
        self._staged_supported()
        self._source_id(stage_id, "bst")
        self._source_key(idempotency_key)
        if (type(target_index_schema_version) is not int or target_index_schema_version not in (1, 2)
            or acknowledge_index_replacement is not True or acknowledge_source_content is not True or acknowledge_replace_files is not True):
            raise ValueError("Select an index version and every staged restore acknowledgment")
        response = self.request("POST", self.api_path("/project/source-transfer-stages/" + stage_id + "/preview"),
                                {"target_index_schema_version":target_index_schema_version, "acknowledge_index_replacement":True,
                                 "acknowledge_source_content":True, "acknowledge_replace_files":True}, idempotency_key=idempotency_key)
        self._source_closed(response, ("validation", "preview", "replacement"))
        self._source_validation(response["validation"])
        preview = self._source_preview(response["preview"])
        if response["validation"] != preview["validation"]:
            raise RuntimeError("The staged preview validation envelopes disagree")
        self._source_replacement(response["replacement"], preview, target_index_schema_version)
        return response

    def prepare_source_stream_export(self, target_path, *, acknowledge_sensitive_metadata, acknowledge_source_content, idempotency_key):
        """Prepare an acknowledged staged-profile export through the unchanged Operation/preview/commit workflow."""
        self._staged_supported()
        self._source_path(target_path)
        self._source_key(idempotency_key)
        if acknowledge_sensitive_metadata is not True or acknowledge_source_content is not True:
            raise ValueError("Both staged source sensitivity acknowledgments are required")
        return self.request("POST", self.api_path("/project/source-stream-exports"),
                            {"target_path":target_path, "acknowledge_sensitive_metadata":True, "acknowledge_source_content":True},
                            idempotency_key=idempotency_key)

    def source_stream_manifest(self, operation_id, *, expected_sha256):
        """Read one exact committed private family manifest; preparation alone cannot earn download authority."""
        self._staged_supported()
        self._source_id(operation_id, "op")
        self._source_hash(expected_sha256)
        value = self.request("GET", self.api_path("/project/source-stream-exports/" + operation_id + "/manifest"))
        self._source_closed(value, ("operation_id", "schema_version", "profile", "source_content_included", "artifact_sha256",
                                   "artifact_size_bytes", "chunk_size_bytes", "chunk_count"))
        self._staged_identity(value)
        if value["operation_id"] != operation_id or value["artifact_sha256"] != expected_sha256 or value["source_content_included"] is not True:
            raise RuntimeError("The committed staged manifest identity disagrees")
        return value

    def source_stream_chunk(self, manifest, chunk_ordinal):
        """Read one exact ordinal with manifest/full/part identity checks; never a supplied path or range."""
        self._staged_supported()
        self._source_closed(manifest, ("operation_id", "schema_version", "profile", "source_content_included", "artifact_sha256",
                                      "artifact_size_bytes", "chunk_size_bytes", "chunk_count"))
        self._staged_identity(manifest)
        self._source_id(manifest["operation_id"], "op")
        if manifest["source_content_included"] is not True:
            raise ValueError("Select an explicit source manifest")
        ordinal = self._source_integer(chunk_ordinal, manifest["chunk_count"] - 1)
        value = self.request("GET", self.api_path("/project/source-stream-exports/" + manifest["operation_id"] + "/chunks/" + str(ordinal)))
        self._source_closed(value, ("operation_id", "artifact_sha256", "chunk_ordinal", "sha256", "size_bytes", "hex"))
        self._source_hash(value["sha256"])
        self._source_integer(value["chunk_ordinal"], MAX_STAGED_PARTS - 1)
        size = self._source_integer(value["size_bytes"], STAGED_PART_BYTES)
        part = self._staged_hex(value["hex"])
        expected_size = min(STAGED_PART_BYTES, manifest["artifact_size_bytes"] - ordinal * STAGED_PART_BYTES)
        if (value["operation_id"] != manifest["operation_id"] or value["artifact_sha256"] != manifest["artifact_sha256"]
            or value["chunk_ordinal"] != ordinal or size != expected_size or len(part) != size
            or hashlib.sha256(part).hexdigest() != value["sha256"]):
            raise RuntimeError("The committed staged part identity or exact bytes disagree")
        return value

    def download_source_stream_export(self, operation_id, *, expected_sha256, timeout=60):
        """Reconstruct exact committed Bundle4 bytes and full hash; no local publication or import confirmation."""
        self._staged_supported()
        if type(timeout) not in (int, float) or not 0 < timeout <= 600:
            raise ValueError("Choose a finite staged download budget up to600seconds")
        deadline = time.monotonic() + timeout
        manifest = self.source_stream_manifest(operation_id, expected_sha256=expected_sha256)
        result = bytearray()
        for ordinal in range(manifest["chunk_count"]):
            if time.monotonic() >= deadline:
                raise TimeoutError("The staged download budget expired without a complete artifact")
            value = self.source_stream_chunk(manifest, ordinal)
            result.extend(self._staged_hex(value["hex"]))
        if time.monotonic() >= deadline or len(result) != manifest["artifact_size_bytes"] or hashlib.sha256(result).hexdigest() != expected_sha256:
            raise RuntimeError("The complete committed staged artifact is unverified")
        return self._staged_bundle(bytes(result))

    def wait(self, operation, timeout=35):
        """Poll retained IDs in the same negotiated session without restarting an effect."""
        deadline = time.monotonic() + timeout
        while operation["state"] in ("pending", "running"):
            if time.monotonic() >= deadline:
                raise TimeoutError("Query the retained operation before retrying its effect")
            time.sleep(0.1)
            operation = self.request("GET", self.api_path("/operations/" + operation["operation_id"]))
        return operation

    def commit(self, preview, *, confirmed, idempotency_key):
        """Confirm the exact session-owned preview on the selected major, with its original key."""
        if confirmed is not True:
            raise ValueError("Exact preview confirmation is required")
        return self.request("POST", self.api_path("/effects/commits"), {
            "receipt": preview["receipt"]["token"], "observed_version": preview["target_version"], "confirmed": True,
        }, idempotency_key=idempotency_key)

    def close(self):
        """Request selected-major shutdown, then retain the existing owned process cleanup."""
        if getattr(self, "_capability", "") and self.process.poll() is None:
            try:
                self.request("POST", self.api_path("/session/shutdown"), {})
            except (OSError, RuntimeError, ValueError):
                pass
        self._capability = ""
        if self.process.poll() is None:
            try:
                self.process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait()
        if self.process.stdout:
            self.process.stdout.close()

    def __enter__(self):
        """Return this already-negotiated owned session to a context manager."""
        return self

    def __exit__(self, *_):
        """Close the owned session even when the context body fails."""
        self.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--forge", required=True)
    parser.add_argument("--project", required=True)
    parser.add_argument("--api-major", type=int, choices=(1, 2), default=1)
    arguments = parser.parse_args()
    with Workspace(arguments.forge, arguments.project, api_major=arguments.api_major) as client:
        print(json.dumps(client.request("GET", client.api_path("/project/summary")), indent=2))
